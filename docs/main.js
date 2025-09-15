import init, { make_key, make_id_card, encrypt_message, decrypt_message, verify_message } from './pkg/gxt_wasm.js';

const $ = (s) => document.querySelector(s);
const maskKey = (key) => {
  if (!key || typeof key !== 'string') return '–';
  const vis = Math.max(1, Math.ceil(key.length * 0.01));
  return key.slice(0, vis) + '…' + '•'.repeat(Math.max(0, key.length - vis));
};

let currentKey = '';
let currentId = '';

async function boot() {
  await init();
  bindTabs();
  bindCrypt();
  bindDecrypt();
}

function bindTabs() {
  const tabs = document.querySelectorAll('.tab');
  tabs.forEach((t) => t.addEventListener('click', () => {
    tabs.forEach((x) => x.classList.remove('active'));
    t.classList.add('active');
    const isCrypt = t.dataset.tab === 'crypt';
    $('#panel-crypt').hidden = !isCrypt;
    $('#panel-decrypt').hidden = isCrypt;
  }));
}

function readFileAsText(file) {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onerror = () => reject(reader.error);
    reader.onload = () => resolve(String(reader.result || ''));
    reader.readAsText(file);
  });
}

function setKey(key) {
  currentKey = (key || '').trim();
  $('#maskedKey').textContent = maskKey(currentKey);
  $('#maskedKey2').textContent = maskKey(currentKey);
}

function ensureKey() {
  if (!currentKey) {
    // Falls kein Key hochgeladen wurde, generieren (nur Demo)
    currentKey = make_key();
    $('#maskedKey').textContent = maskKey(currentKey);
    $('#maskedKey2').textContent = maskKey(currentKey);
  }
}

function bindCrypt() {
  $('#keyFile').addEventListener('change', async (e) => {
    const f = e.target.files?.[0];
    if (!f) return;
    const text = await readFileAsText(f);
    setKey(text);
  });

  $('#msgFile').addEventListener('change', async (e) => {
    const f = e.target.files?.[0];
    if (!f) return;
    const text = await readFileAsText(f);
    $('#msgInput').value = text;
  });

  $('#clearMsg').addEventListener('click', () => $('#msgInput').value = '');

  $('#btnMakeId').addEventListener('click', () => {
    try {
      ensureKey();
      const meta = { createdAt: new Date().toISOString() };
      currentId = make_id_card(currentKey, meta);
      $('#outputCrypt').value = currentId;
    } catch (err) {
      $('#outputCrypt').value = 'Fehler (ID): ' + (err?.message || String(err));
    }
  });

  $('#btnVerify').addEventListener('click', () => {
    try {
      const src = ($('#msgInput').value || '').trim();
      if (!src) { $('#outputCrypt').value = 'Keine Nachricht.'; return; }
      const v = verify_message(src);
      $('#outputCrypt').value = JSON.stringify(v, null, 2);
    } catch (err) {
      $('#outputCrypt').value = 'Fehler (Verify): ' + (err?.message || String(err));
    }
  });

  $('#btnEncrypt').addEventListener('click', () => {
    try {
      ensureKey();
      if (!currentId) currentId = make_id_card(currentKey, { createdAt: new Date().toISOString() });
      const raw = ($('#msgInput').value || '').trim();
      const payload = raw ? tryParseJson(raw) ?? { text: raw } : { text: '' };
      const token = encrypt_message(currentKey, currentId, payload, null);
      $('#outputCrypt').value = token;
    } catch (err) {
      $('#outputCrypt').value = 'Fehler (Encrypt): ' + (err?.message || String(err));
    }
  });
}

function bindDecrypt() {
  $('#cipherFile').addEventListener('change', async (e) => {
    const f = e.target.files?.[0];
    if (!f) return;
    const text = await readFileAsText(f);
    $('#cipherInput').value = text;
  });
  $('#clearCipher').addEventListener('click', () => $('#cipherInput').value = '');

  $('#btnDecrypt').addEventListener('click', () => {
    try {
      ensureKey();
      const token = ($('#cipherInput').value || '').trim();
      if (!token) { appendChat('Keine verschlüsselte Nachricht.'); return; }
      const env = decrypt_message(token, currentKey);
      appendChat(JSON.stringify(env.payload, null, 2));
    } catch (err) {
      appendChat('Fehler (Decrypt): ' + (err?.message || String(err)));
    }
  });
}

function appendChat(line) {
  const box = $('#chatBox');
  box.value += (box.value ? '\n' : '') + line;
  box.scrollTop = box.scrollHeight;
}

function tryParseJson(text) {
  try { return JSON.parse(text); } catch { return null; }
}

boot();
