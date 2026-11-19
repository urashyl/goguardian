use std::env;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

const HEALTH_JSON: &str = r#"{"status":"ok","service":"goguardian-docs","backend":"rust"}"#;
const META_JSON: &str = r#"{"extension_version":"4.1.246","manifest_version":3,"content_scripts":22,"microservices":10,"ai_adapters":7,"pusher_channel_types":6}"#;

#[derive(Debug, PartialEq, Eq)]
enum Route {
    ApiHealth,
    ApiMeta,
    StaticFile(String),
    NotFound,
}

fn resolve_route(raw_target: &str) -> Route {
    let raw_path = raw_target.split('?').next().unwrap_or(raw_target);
    let trimmed_end = raw_path.trim_end_matches('/');
    let path = if trimmed_end.is_empty() { "/" } else { trimmed_end };
    match path {
        "/api/health" => Route::ApiHealth,
        "/api/meta" => Route::ApiMeta,
        "/" | "/index.html" => Route::StaticFile("index.html".to_string()),
        "/telemetry" | "/telemetry.html" => Route::StaticFile("telemetry.html".to_string()),
        "/ai-and-docs" | "/ai-and-docs.html" => Route::StaticFile("ai-and-docs.html".to_string()),
        "/filtering" | "/filtering.html" => Route::StaticFile("filtering.html".to_string()),
        "/teacher-control" | "/teacher-control.html" => {
            Route::StaticFile("teacher-control.html".to_string())
        }
        "/scripts" | "/scripts.html" => Route::StaticFile("scripts.html".to_string()),
        other => {
            let trimmed = other.trim_start_matches('/');
            if trimmed.is_empty() || !is_safe_relative_path(trimmed) {
                Route::NotFound
            } else {
                Route::StaticFile(trimmed.to_string())
            }
        }
    }
}

fn is_safe_relative_path(rel: &str) -> bool {
    if rel.contains('\\') || rel.contains('\0') {
        return false;
    }
    let p = Path::new(rel);
    if p.is_absolute() {
        return false;
    }
    for comp in p.components() {
        match comp {
            Component::Normal(_) => {}
            _ => return false,
        }
    }
    true
}

fn content_type_for(filename: &str) -> &'static str {
    if filename.ends_with(".html") {
        "text/html; charset=utf-8"
    } else if filename.ends_with(".css") {
        "text/css; charset=utf-8"
    } else if filename.ends_with(".js") {
        "application/javascript; charset=utf-8"
    } else if filename.ends_with(".json") {
        "application/json; charset=utf-8"
    } else if filename.ends_with(".svg") {
        "image/svg+xml"
    } else if filename.ends_with(".txt") {
        "text/plain; charset=utf-8"
    } else {
        "application/octet-stream"
    }
}

fn find_static_dir() -> PathBuf {
    let cwd_static = PathBuf::from("static");
    if cwd_static.is_dir() {
        return fs::canonicalize(cwd_static).unwrap_or_else(|_| PathBuf::from("static"));
    }
    if let Ok(exe) = env::current_exe() {
        for ancestor in exe.ancestors().skip(1) {
            let candidate = ancestor.join("static");
            if candidate.is_dir() {
                return fs::canonicalize(candidate).unwrap_or(ancestor.join("static"));
            }
        }
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("static")
}

fn read_static_asset(static_dir: &Path, rel: &str) -> Option<(Vec<u8>, &'static str)> {
    if !is_safe_relative_path(rel) {
        return None;
    }
    let joined = static_dir.join(rel);
    let canonical_base = fs::canonicalize(static_dir).ok()?;
    let canonical_target = fs::canonicalize(&joined).ok()?;
    if !canonical_target.starts_with(&canonical_base) || !canonical_target.is_file() {
        return None;
    }
    let bytes = fs::read(&canonical_target).ok()?;
    Some((bytes, content_type_for(rel)))
}

fn write_response(
    stream: &mut TcpStream,
    status: &str,
    content_type: &str,
    body: &[u8],
    head_only: bool,
) {
    let header = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nX-Content-Type-Options: nosniff\r\nConnection: close\r\n\r\n",
        body.len()
    );
    let _ = stream.write_all(header.as_bytes());
    if !head_only {
        let _ = stream.write_all(body);
    }
    let _ = stream.flush();
}

fn handle_client(mut stream: TcpStream, static_dir: &Path) {
    let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
    let mut reader = match stream.try_clone() {
        Ok(cloned) => BufReader::new(cloned),
        Err(_) => return,
    };

    let mut request_line = String::new();
    if reader.read_line(&mut request_line).unwrap_or(0) == 0 {
        return;
    }

    // Drain remaining headers until empty line
    let mut header_line = String::new();
    loop {
        header_line.clear();
        match reader.read_line(&mut header_line) {
            Ok(0) | Err(_) => break,
            Ok(_) => {
                if header_line == "\r\n" || header_line == "\n" {
                    break;
                }
            }
        }
    }

    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or("");
    let target = parts.next().unwrap_or("/");

    if method != "GET" && method != "HEAD" {
        write_response(
            &mut stream,
            "405 Method Not Allowed",
            "application/json; charset=utf-8",
            br#"{"error":"Method not allowed; read-only documentation server"}"#,
            false,
        );
        return;
    }
    let head_only = method == "HEAD";

    match resolve_route(target) {
        Route::ApiHealth => write_response(
            &mut stream,
            "200 OK",
            "application/json; charset=utf-8",
            HEALTH_JSON.as_bytes(),
            head_only,
        ),
        Route::ApiMeta => write_response(
            &mut stream,
            "200 OK",
            "application/json; charset=utf-8",
            META_JSON.as_bytes(),
            head_only,
        ),
        Route::StaticFile(rel) => match read_static_asset(static_dir, &rel) {
            Some((bytes, ctype)) => {
                write_response(&mut stream, "200 OK", ctype, &bytes, head_only);
            }
            None => {
                write_response(
                    &mut stream,
                    "404 Not Found",
                    "text/plain; charset=utf-8",
                    b"404 Not Found\n",
                    head_only,
                );
            }
        },
        Route::NotFound => {
            write_response(
                &mut stream,
                "404 Not Found",
                "text/plain; charset=utf-8",
                b"404 Not Found\n",
                head_only,
            );
        }
    }
}

fn parse_bind_addr() -> String {
    let mut host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let mut port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--host" => {
                if let Some(val) = args.next() {
                    host = val;
                }
            }
            "--port" => {
                if let Some(val) = args.next() {
                    port = val;
                }
            }
            _ => {}
        }
    }
    format!("{host}:{port}")
}

fn main() {
    let addr = parse_bind_addr();
    let static_dir = Arc::new(find_static_dir());
    let listener = TcpListener::bind(&addr).unwrap_or_else(|e| {
        eprintln!("Failed to bind to {addr}: {e}");
        std::process::exit(1);
    });

    println!(
        "goguardian-docs Rust server listening at http://{addr} (static: {})",
        static_dir.display()
    );

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let dir = Arc::clone(&static_dir);
                thread::spawn(move || handle_client(stream, &dir));
            }
            Err(err) => {
                eprintln!("Accept error: {err}");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_clean_page_routes() {
        assert_eq!(resolve_route("/"), Route::StaticFile("index.html".into()));
        assert_eq!(
            resolve_route("/telemetry"),
            Route::StaticFile("telemetry.html".into())
        );
        assert_eq!(
            resolve_route("/ai-and-docs?section=1"),
            Route::StaticFile("ai-and-docs.html".into())
        );
        assert_eq!(
            resolve_route("/filtering"),
            Route::StaticFile("filtering.html".into())
        );
        assert_eq!(
            resolve_route("/teacher-control"),
            Route::StaticFile("teacher-control.html".into())
        );
        assert_eq!(
            resolve_route("/scripts"),
            Route::StaticFile("scripts.html".into())
        );
    }

    #[test]
    fn maps_trailing_slash_routes() {
        assert_eq!(
            resolve_route("/telemetry/"),
            Route::StaticFile("telemetry.html".into())
        );
        assert_eq!(
            resolve_route("/ai-and-docs/"),
            Route::StaticFile("ai-and-docs.html".into())
        );
        assert_eq!(
            resolve_route("/filtering/"),
            Route::StaticFile("filtering.html".into())
        );
        assert_eq!(
            resolve_route("/teacher-control/"),
            Route::StaticFile("teacher-control.html".into())
        );
        assert_eq!(
            resolve_route("/scripts/"),
            Route::StaticFile("scripts.html".into())
        );
        assert_eq!(resolve_route("/api/health/"), Route::ApiHealth);
        assert_eq!(resolve_route("///"), Route::StaticFile("index.html".into()));
    }

    #[test]
    fn maps_api_routes() {
        assert_eq!(resolve_route("/api/health"), Route::ApiHealth);
        assert_eq!(resolve_route("/api/meta?fmt=json"), Route::ApiMeta);
    }

    #[test]
    fn rejects_path_traversal() {
        assert_eq!(resolve_route("/../Cargo.toml"), Route::NotFound);
        assert_eq!(resolve_route("/static/../../etc/passwd"), Route::NotFound);
        assert!(!is_safe_relative_path("../Cargo.toml"));
        assert!(!is_safe_relative_path("a\\b"));
        assert!(is_safe_relative_path("styles.css"));
    }

    #[test]
    fn detects_content_types() {
        assert_eq!(content_type_for("index.html"), "text/html; charset=utf-8");
        assert_eq!(content_type_for("styles.css"), "text/css; charset=utf-8");
        assert_eq!(
            content_type_for("app.js"),
            "application/javascript; charset=utf-8"
        );
        assert_eq!(
            content_type_for("data.json"),
            "application/json; charset=utf-8"
        );
    }
}
