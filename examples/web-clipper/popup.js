// 900Notes Web Clipper: Popup Script

const portInput = document.getElementById('port');
const tokenInput = document.getElementById('token');
const saveBtn = document.getElementById('save');
const clipBtn = document.getElementById('clip');
const statusEl = document.getElementById('status');

// Load saved settings
chrome.storage.local.get(['port', 'clipperToken'], (data) => {
  portInput.value = data.port || '17690';
  tokenInput.value = data.clipperToken || '';
});

saveBtn.addEventListener('click', () => {
  const port = parseInt(portInput.value, 10);
  const clipperToken = tokenInput.value.trim();
  chrome.storage.local.set({ port, clipperToken });
  showStatus('Settings saved', 'ok');
});

clipBtn.addEventListener('click', () => {
  chrome.tabs.query({ active: true, currentWindow: true }, (tabs) => {
    if (tabs[0]) {
      chrome.tabs.sendMessage(tabs[0].id, { action: 'clip' }, (response) => {
        if (chrome.runtime.lastError) {
          showStatus('Error: ' + chrome.runtime.lastError.message, 'err');
        }
      });
      showStatus('Clipping...', 'ok');
      setTimeout(() => window.close(), 1000);
    }
  });
});

function showStatus(msg, cls) {
  statusEl.textContent = msg;
  statusEl.className = 'status ' + cls;
  setTimeout(() => {
    statusEl.textContent = '';
    statusEl.className = 'status';
  }, 3000);
}
