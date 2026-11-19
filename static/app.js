(() => {
  "use strict";

  // Highlight active navigation link
  const currentPath =
    window.location.pathname.replace(/\.html$/, "").replace(/\/+$/, "") || "/";
  document.querySelectorAll(".nav a").forEach((link) => {
    const href = link.getAttribute("href");
    if (href === currentPath || (currentPath === "/index" && href === "/")) {
      link.classList.add("active");
    }
  });

  // Simple / Advanced reading mode controller
  const modeStorageKey = "gg-docs-mode";
  const modeButtons = document.querySelectorAll("[data-mode]");
  const modeHint = document.getElementById("mode-hint");

  function setMode(mode) {
    const isAdvanced = mode === "advanced";
    document.body.classList.toggle("mode-advanced", isAdvanced);
    try {
      localStorage.setItem(modeStorageKey, isAdvanced ? "advanced" : "simple");
    } catch {
      // Ignore storage failures in restricted contexts
    }
    modeButtons.forEach((btn) => {
      const active = btn.dataset.mode === (isAdvanced ? "advanced" : "simple");
      btn.classList.toggle("active", active);
      btn.setAttribute("aria-pressed", active ? "true" : "false");
    });
    if (modeHint) {
      modeHint.textContent = isAdvanced
        ? "Advanced view active: showing API endpoints, manifest permissions, scripts, and forensic protocols."
        : "Simple view active: plain-English summary for parents and students. Switch to Advanced for code-level proof.";
    }
  }

  let initialMode = "simple";
  try {
    const params = new URLSearchParams(window.location.search);
    const urlMode = params.get("mode");
    const savedMode = localStorage.getItem(modeStorageKey);
    if (urlMode === "advanced" || urlMode === "simple") {
      initialMode = urlMode;
    } else if (savedMode === "advanced" || savedMode === "simple") {
      initialMode = savedMode;
    }
  } catch {
    // Ignore URL/storage read failures
  }
  setMode(initialMode);

  modeButtons.forEach((btn) => {
    btn.addEventListener("click", () => {
      setMode(btn.dataset.mode === "advanced" ? "advanced" : "simple");
    });
  });

  document.querySelectorAll("[data-enable-advanced]").forEach((btn) => {
    btn.addEventListener("click", () => {
      setMode("advanced");
    });
  });

  // Ping Rust backend health endpoint
  const statusEl = document.getElementById("rust-status");
  if (statusEl) {
    fetch("/api/health")
      .then((r) => (r.ok ? r.json() : Promise.reject(r.status)))
      .then((data) => {
        statusEl.textContent = `Rust backend: ${data.status} (${data.service})`;
        statusEl.className = "badge green";
      })
      .catch(() => {
        statusEl.textContent = "Rust backend: offline (static view)";
        statusEl.className = "badge";
      });
  }

  // Scripts page: search + world filter
  const scriptSearch = document.getElementById("script-search");
  const scriptRows = document.querySelectorAll("#scripts-table tbody tr");
  const worldBtns = document.querySelectorAll("[data-world-filter]");
  if (scriptSearch && scriptRows.length) {
    let activeWorld = "ALL";
    const applyFilter = () => {
      const q = scriptSearch.value.trim().toLowerCase();
      scriptRows.forEach((row) => {
        const world = row.dataset.world || "";
        const text = row.textContent.toLowerCase();
        const matchesWorld = activeWorld === "ALL" || world === activeWorld;
        const matchesText = !q || text.includes(q);
        row.style.display = matchesWorld && matchesText ? "" : "none";
      });
    };
    scriptSearch.addEventListener("input", applyFilter);
    worldBtns.forEach((btn) => {
      btn.addEventListener("click", () => {
        worldBtns.forEach((b) => b.classList.remove("active"));
        btn.classList.add("active");
        activeWorld = btn.dataset.worldFilter;
        applyFilter();
      });
    });
  }

  // Filtering page: SafeTranslate URL unwrapper
  const stInput = document.getElementById("safetranslate-input");
  const stBtn = document.getElementById("safetranslate-btn");
  const stOut = document.getElementById("safetranslate-out");

  function unwrapSafeTranslate(raw) {
    let u;
    try {
      u = new URL(raw.includes("://") ? raw : "https://" + raw);
    } catch {
      return { error: "Invalid URL format" };
    }
    const host = u.hostname.toLowerCase();
    if (host === "translate.google.com") {
      const target = u.searchParams.get("u") || "";
      return {
        matched: Boolean(target),
        mode: "translate.google.com?u=",
        unwrapped: target || "(missing u query param)",
      };
    }
    if (host.endsWith(".translate.goog")) {
      let sub = host.slice(0, -15);
      if (!sub) return { error: "Empty subdomain before .translate.goog" };
      const enc = (u.searchParams.get("_x_tr_enc") || "").split(",");
      const hp = u.searchParams.get("_x_tr_hp");
      if (hp) sub = hp + sub;
      if (enc.includes("1") && sub.startsWith("1-")) sub = sub.slice(2);
      let puny = false;
      if (enc.includes("0") && sub.startsWith("0-")) {
        puny = true;
        sub = sub.slice(2);
      }
      let decodedHost = sub.replace(/\b-\b/g, ".").replace(/--/g, "-");
      if (puny) decodedHost = "xn--" + decodedHost;
      const cleanParams = new URLSearchParams();
      for (const [k, v] of u.searchParams) {
        if (!k.startsWith("_x_tr_")) cleanParams.append(k, v);
      }
      const qs = cleanParams.toString();
      return {
        matched: true,
        mode: "*.translate.goog",
        unwrapped: `${u.protocol}//${decodedHost}${u.pathname}${qs ? "?" + qs : ""}${u.hash}`,
      };
    }
    return { matched: false, mode: "direct", unwrapped: u.toString() };
  }

  if (stInput && stBtn && stOut) {
    const runDecode = () => {
      const res = unwrapSafeTranslate(stInput.value.trim());
      stOut.textContent = JSON.stringify(res, null, 2);
    };
    stBtn.addEventListener("click", runDecode);
    document.querySelectorAll("[data-preset-url]").forEach((btn) => {
      btn.addEventListener("click", () => {
        stInput.value = btn.dataset.presetUrl;
        runDecode();
      });
    });
    runDecode();
  }
})();
