// 900Notes Web Clipper — Background Service Worker
// Handles context menu creation and command shortcuts

const DEFAULT_PORT = 1420;

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
  const { port = DEFAULT_PORT } = await chrome.storage.local.get('port');

  const response = await fetch(`http://127.0.0.1:${port}/api/clip`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data),
  });

  if (!response.ok) {
    throw new Error(`Server returned ${response.status}`);
  }

  return response.json();
}
