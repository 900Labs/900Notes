// 900Notes Web Clipper: Background Service Worker
// Handles context menu creation and command shortcuts

const DEFAULT_PORT = 17690;

chrome.runtime.onInstalled.addListener(() => {
  chrome.contextMenus.create({
    id: 'clip-to-900notes',
    title: 'Clip to 900Notes',
    contexts: ['page', 'selection'],
  });
});

chrome.contextMenus.onClicked.addListener((info, tab) => {
  if (info.menuItemId === 'clip-to-900notes') {
    chrome.tabs.sendMessage(tab.id, { action: 'clip', selection: info.selectionText });
  }
});

chrome.commands.onCommand.addListener((command) => {
  if (command === 'clip-page') {
    chrome.tabs.query({ active: true, currentWindow: true }, (tabs) => {
      if (tabs[0]) {
        chrome.tabs.sendMessage(tabs[0].id, { action: 'clip' });
      }
    });
  }
});

chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
  if (message.action === 'saveTo900Notes') {
    saveTo900Notes(message.data)
      .then((result) => sendResponse({ success: true, result }))
      .catch((error) => sendResponse({ success: false, error: error.message }));
    return true;
  }
});

async function saveTo900Notes(data) {
  const { port = DEFAULT_PORT, clipperToken = '' } = await chrome.storage.local.get([
    'port',
    'clipperToken',
  ]);
  const token = clipperToken.trim();
  if (!token) {
    throw new Error('900Notes clipper token is required');
  }

  const response = await fetch(`http://127.0.0.1:${port}/api/clip`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'X-900Notes-Clipper-Token': token,
    },
    body: JSON.stringify(data),
  });

  if (!response.ok) {
    const error = await response.json().catch(() => null);
    throw new Error(error?.error || `Server returned ${response.status}`);
  }

  return response.json();
}
