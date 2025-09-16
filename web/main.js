import init, {
  make_key,
  make_id_card,
  encrypt_message,
  decrypt_message,
  verify_message,
} from "./pkg/gxt_wasm.js";

const kinds = ["k", "i", "m"];
const $ = (s) => document.querySelector(s);
const stripPrefix = (key) => (kinds.some((k) => { key?.startsWith(`gx${k}:`) }) ? key.slice(4) : key);
const maskKey = (key) => {
  if (!key || typeof key !== "string") return "–";
  const k = stripPrefix(key);
  const vis = Math.max(1, Math.ceil(k.length * 0.01));
  return k.slice(0, vis) + "…" + "•".repeat(Math.max(0, k.length - vis));
};

let currentKey = "";
let currentId = "";
let idCards = []; // { id: string, label: string, value: string }

async function boot() {
  await init();
  bindTabs();
  bindIdPanel();
  bindVerify();
  bindCrypt();
  bindDecrypt();
  renderIdList();
  refreshIdSelect();
  // Initial sichtbar: nur ID-Panel
  $("#panel-id").hidden = false;
  $("#panel-verify").hidden = true;
  $("#panel-decrypt").hidden = true;
  $("#panel-crypt").hidden = true;
}

function bindTabs() {
  const tabs = document.querySelectorAll(".tab");
  tabs.forEach((t) =>
    t.addEventListener("click", () => {
      if (t.classList.contains("disabled")) return;
      tabs.forEach((x) => x.classList.remove("active"));
      t.classList.add("active");
      const tab = t.dataset.tab;
      $("#panel-id").hidden = tab !== "id";
      $("#panel-verify").hidden = tab !== "verify";
      $("#panel-decrypt").hidden = tab !== "decrypt";
      $("#panel-crypt").hidden = tab !== "crypt";
    })
  );
}

function readFileAsText(file) {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onerror = () => reject(reader.error);
    reader.onload = () => resolve(String(reader.result || ""));
    reader.readAsText(file);
  });
}

function setKey(key) {
  currentKey = (key || "").trim();
  $("#maskedKey").textContent = maskKey(currentKey);
  updateTabAvailability();
}

function ensureKey() {
  if (!currentKey) {
    currentKey = make_key();
    $("#maskedKey").textContent = maskKey(currentKey);
    // Nutzer fragen und automatisch downloaden
    try {
      if (confirm("Download New Key?")) {
        const blob = new Blob([currentKey], {
          type: "text/plain;charset=utf-8",
        });
        const a = document.createElement("a");
        a.href = URL.createObjectURL(blob);
        a.download = "my.gxk";
        document.body.appendChild(a);
        a.click();
        setTimeout(() => {
          URL.revokeObjectURL(a.href);
          a.remove();
        }, 0);
      }
    } catch {}
    updateTabAvailability();
  }
}

function bindIdPanel() {
  $("#keyFile").addEventListener("change", async (e) => {
    const f = e.target.files?.[0];
    if (!f) return;
    const text = await readFileAsText(f);
    setKey(text);
  });

  // Generate .key button
  $("#btnGenerateKey").addEventListener("click", () => {
    // Force generate now; ensureKey() only generates if empty
    currentKey = "";
    ensureKey();
  });

  $("#idFile").addEventListener("change", async (e) => {
    const f = e.target.files?.[0];
    if (!f) return;
    const text = await readFileAsText(f);
    const tokens = extractTokens(text);
    // Dateiname ohne Endung als Label verwenden
    const base = (f.name || "").replace(/\.gx(k|m|i)$/i, "");
    let added = 0;
    for (const tok of tokens) {
      try {
        await addIdCard(tok, base || "Import");
        added++;
      } catch {}
    }
    if (!added) alert("No valid ID card found.");
    renderIdList();
    refreshIdSelect();
  });

  $("#btnMakeId").addEventListener("click", async () => {
    try {
      ensureKey();
      // Meta aus JSON-Textarea strikt validieren
      const metaText = ($("#idMeta").value || "").trim();
      const labelInput = ($("#idLabel").value || "").trim();
      let meta = {};
      if (metaText) {
        try {
          meta = JSON.parse(metaText);
        } catch (err) {
          $("#outputId").value =
            "Error (Meta JSON): " + (err?.message || String(err));
          return;
        }
      }
      const idTok = make_id_card(currentKey, meta);
      currentId = idTok;
      $("#outputId").value = idTok;
      await addIdCard(idTok, labelInput || meta?.name || "ID");
      renderIdList();
      refreshIdSelect();
    } catch (err) {
      $("#outputId").value = "Error (ID): " + (err?.message || String(err));
    }
  });

  $("#btnCopyLastId").addEventListener("click", async () => {
    const val = currentId || idCards[idCards.length - 1]?.value;
    if (!val) return;
    try {
      await navigator.clipboard.writeText(val);
    } catch {}
  });

  // Backup Export/Import
  $("#btnExportBackup").addEventListener("click", () => {
    // Export nur, wenn mindestens Key oder eine ID vorhanden
    if (!(currentKey || idCards.length)) {
      alert("Nothing to export yet. Add a key or an ID card.");
      return;
    }
    const data = {
      key: currentKey || null,
      ids: idCards.map((c) => ({ id: c.id, label: c.label, value: c.value })),
    };
    const blob = new Blob([JSON.stringify(data, null, 2)], {
      type: "application/json;charset=utf-8",
    });
    const a = document.createElement("a");
    a.href = URL.createObjectURL(blob);
    a.download = "gxt-backup.json";
    document.body.appendChild(a);
    a.click();
    setTimeout(() => {
      URL.revokeObjectURL(a.href);
      a.remove();
    }, 0);
  });
  $("#btnImportBackup").addEventListener("click", () =>
    $("#backupFile").click()
  );
  $("#backupFile").addEventListener("change", async (e) => {
    const f = e.target.files?.[0];
    if (!f) return;
    const text = await readFileAsText(f);
    try {
      const data = JSON.parse(text);
      if (data?.key) setKey(String(data.key));
      if (Array.isArray(data?.ids)) {
        for (const item of data.ids) {
          try {
            await addIdCard(String(item.value), String(item.label || "ID"));
          } catch {}
        }
      }
      renderIdList();
      refreshIdSelect();
    } catch (err) {
      alert("Invalid Backup: " + (err?.message || String(err)));
    }
    e.target.value = "";
  });
}

function extractTokens(text) {
  const tokens = [];
  for (const part of String(text).split(/\s+/)) {
    const good_prefix = kinds.some((k) => {
      return part.startsWith(`gx${k}:`);
    });
    if (good_prefix) tokens.push(part);
  }
  return tokens;
}

async function addIdCard(value, label) {
  const env = verify_message(value);
  if (env.kind !== "id" && env.kind !== "Id") throw new Error("Invalid Payload Kind");
  const short = String(env.id || "").slice(0, 8);
  const name = label || (env?.payload?.name ?? "ID");
  // Duplikate vermeiden
  if (!idCards.find((x) => x.id === env.id)) {
    idCards.push({ id: env.id, label: `${name} (${short})`, value });
  }
}

function renderIdList() {
  const list = $("#idList");
  list.innerHTML = "";
  $("#idCount").textContent = String(idCards.length);
  for (const card of idCards) {
    const row = document.createElement("div");
    row.style.display = "flex";
    row.style.gap = "8px";
    row.style.alignItems = "center";

    const selectBtn = document.createElement("button");
    selectBtn.className = "btn secondary";
    selectBtn.textContent = card.label;
    selectBtn.style.flex = "1 1 auto";
    selectBtn.addEventListener("click", async () => {
      try {
        await navigator.clipboard.writeText(card.value);
      } catch {}
      currentId = card.value;
      refreshIdSelect();
    });
    // Rechte Maustaste: Kontextmenü öffnen
    selectBtn.addEventListener("contextmenu", (ev) => {
      ev.preventDefault();
      openContextMenu(ev.clientX, ev.clientY, card);
    });

    row.appendChild(selectBtn);
    list.appendChild(row);
  }
}

function deleteIdCard(id) {
  idCards = idCards.filter((x) => x.id !== id);
  // Falls die aktuell gewählte ID gelöscht wurde, Auswahl zurücksetzen
  try {
    if (currentId && verifyId(currentId) === id) currentId = "";
  } catch {}
  renderIdList();
  refreshIdSelect();
}

// Kontextmenü für ID-Cards
let contextMenuEl = null;
function openContextMenu(x, y, card) {
  closeContextMenu();
  const menu = document.createElement("div");
  menu.className = "context-menu";
  menu.style.left = `${x}px`;
  menu.style.top = `${y}px`;

  const exportBtn = document.createElement("button");
  exportBtn.textContent = "Export (.gxi)";
  exportBtn.addEventListener("click", async () => {
    await exportIdCard(card);
    closeContextMenu();
  });

  const copyBtn = document.createElement("button");
  copyBtn.textContent = "Copy to clipboard";
  copyBtn.addEventListener("click", async () => {
    try {
      await navigator.clipboard.writeText(card.value);
    } catch {}
    closeContextMenu();
  });

  const delBtn = document.createElement("button");
  delBtn.textContent = "Delete";
  delBtn.addEventListener("click", () => {
    if (confirm("Do you really want to delete this ID card??")) deleteIdCard(card.id);
    closeContextMenu();
  });

  menu.appendChild(exportBtn);
  menu.appendChild(copyBtn);
  menu.appendChild(delBtn);
  document.body.appendChild(menu);
  contextMenuEl = menu;

  // Schließen bei Klick außerhalb oder Escape
  const onDown = (e) => {
    if (contextMenuEl && !contextMenuEl.contains(e.target)) closeContextMenu();
  };
  const onKey = (e) => {
    if (e.key === "Escape") closeContextMenu();
  };
  setTimeout(() => {
    document.addEventListener("mousedown", onDown, { once: true });
    document.addEventListener("keydown", onKey, { once: true });
  }, 0);
}
function closeContextMenu() {
  if (contextMenuEl) {
    contextMenuEl.remove();
    contextMenuEl = null;
  }
}
async function exportIdCard(card) {
  const blob = new Blob([card.value], { type: "text/plain;charset=utf-8" });
  const a = document.createElement("a");
  a.href = URL.createObjectURL(blob);
  // Dateiname aus Label ableiten (ohne Klammerzusatz)
  const base = (card.label || "id")
    .replace(/\s*\([^)]*\)\s*$/, "")
    .replace(/\s+/g, "_");
  a.download = `${base}.gxi`;
  document.body.appendChild(a);
  a.click();
  setTimeout(() => {
    URL.revokeObjectURL(a.href);
    a.remove();
  }, 0);
}

// Import aus Textfeld in der Seitenleiste
$("#btnImportIdText")?.addEventListener("click", async () => {
  const text = ($("#idImportInput")?.value || "").trim();
  if (!text) return;
  const tokens = extractTokens(text);
  let added = 0,
    failed = 0;
  for (const tok of tokens) {
    try {
      await addIdCard(tok, "Import");
      added++;
    } catch {
      failed++;
    }
  }
  renderIdList();
  refreshIdSelect();
  if (!added) alert("No valid ID card found in text.");
});

function refreshIdSelect() {
  const sel = $("#idSelect");
  if (!sel) return;
  sel.innerHTML = "";
  for (const card of idCards) {
    const opt = document.createElement("option");
    opt.value = card.id;
    opt.textContent = card.label;
    sel.appendChild(opt);
    if (currentId && verifyId(card.value) === verifyId(currentId))
      sel.value = card.id;
  }
}

function updateTabAvailability() {
  const hasKey = Boolean(currentKey);
  const tabs = [$("#tab-decrypt"), $("#tab-crypt")];
  for (const t of tabs) {
    if (!t) continue;
    if (hasKey) {
      t.classList.remove("disabled");
      t.removeAttribute("aria-disabled");
    } else {
      t.classList.add("disabled");
      t.setAttribute("aria-disabled", "true");
      // Falls ein gesperrter Tab aktiv ist, zurück auf ID
      if (t.classList.contains("active")) {
        $("#tab-id").click();
      }
    }
  }
}

function verifyId(tok) {
  try {
    return verify_message(tok).id;
  } catch {
    return "";
  }
}

function bindCrypt() {
  $("#msgFile").addEventListener("change", async (e) => {
    const f = e.target.files?.[0];
    if (!f) return;
    const text = await readFileAsText(f);
    $("#msgInput").value = text;
  });

  $("#clearMsg").addEventListener("click", () => ($("#msgInput").value = ""));

  $("#btnEncrypt").addEventListener("click", () => {
    try {
      ensureKey();
      const select = $("#idSelect");
      const chosen = idCards.find((x) => x.id === select.value);
      const idToUse = chosen?.value || currentId || "";
      if (!idToUse) {
        $("#outputCrypt").value =
          "Please import an ID card first.";
        return;
      }
      const raw = ($("#msgInput").value || "").trim();
      const payload = raw ? tryParseJson(raw) ?? { text: raw } : { text: "" };
      const token = encrypt_message(currentKey, idToUse, payload, null);
      $("#outputCrypt").value = token;
    } catch (err) {
      $("#outputCrypt").value =
        "Error (Encrypt): " + (err?.message || String(err));
    }
  });
}

function bindVerify() {
  $("#tokenFile").addEventListener("change", async (e) => {
    const f = e.target.files?.[0];
    if (!f) return;
    const text = await readFileAsText(f);
    $("#tokenInput").value = text;
  });
  $("#clearToken").addEventListener(
    "click",
    () => ($("#tokenInput").value = "")
  );

  $("#btnVerify").addEventListener("click", () => {
    try {
      const token = ($("#tokenInput").value || "").trim();
      if (!token) {
        $("#verifyBox").value = "Not a valid token";
        return;
      }
      let env = verify_message(token);
      env.payload = JSON.parse(env.payload);
      $("#verifyBox").value = JSON.stringify(env, null, 2);
    } catch (err) {
      $("#verifyBox").value = "Error (Verify): " + (err?.message || String(err));
    }
  });
}

function bindDecrypt() {
  $("#cipherFile").addEventListener("change", async (e) => {
    const f = e.target.files?.[0];
    if (!f) return;
    const text = await readFileAsText(f);
    $("#cipherInput").value = text;
  });
  $("#clearCipher").addEventListener(
    "click",
    () => ($("#cipherInput").value = "")
  );

  $("#btnDecrypt").addEventListener("click", () => {
    try {
      ensureKey();
      const token = ($("#cipherInput").value || "").trim();
      if (!token) {
        setChat("Not an encrypted message");
        return;
      }
      const env = decrypt_message(token, currentKey);
      setChat(JSON.stringify(JSON.parse(env.payload), null, 2));
    } catch (err) {
      setChat("Error (Decrypt): " + (err?.message || String(err)));
    }
  });
}

function setChat(line) {
  console.log(line);
  const box = $("#chatBox");
  box.value = line;
  box.scrollTop = box.scrollHeight;
}

function tryParseJson(text) {
  try {
    return JSON.parse(text);
  } catch {
    return null;
  }
}

boot();
