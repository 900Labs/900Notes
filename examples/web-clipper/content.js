// 900Notes Web Clipper — Content Script
// Extracts page content and sends to background for saving

chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
  if (message.action === 'clip') {
    const data = extractPageContent(message.selection);
    chrome.runtime.sendMessage({ action: 'saveTo900Notes', data }, (response) => {
      if (response && response.success) {
        showNotification('Clipped to 900Notes!');
      } else {
        showNotification('Failed to clip: ' + (response?.error || 'Unknown error'));
      }
    });
  }
});

function extractPageContent(selection) {
  const title = document.title || 'Untitled';
  const sourceUrl = window.location.href;

  let body;
  if (selection) {
    body = selection;
  } else {
    // Extract main content area
    const article = document.querySelector('article') || document.querySelector('main') || document.body;
    body = article ? article.innerText.trim() : '';
  }

  return {
    title,
    sourceUrl,
    body,
    tags: ['web'],
    useInbox: true,
  };
}

function showNotification(message) {
  const notification = document.createElement('div');
  notification.textContent = message;
  notification.style.cssText = `
    position: fixed;
    top: 20px;
    right: 20px;
    padding: 12px 20px;
    background: #2563eb;
    color: white;
    border-radius: 8px;
    font-family: -apple-system, system-ui, sans-serif;
    font-size: 14px;
    z-index: 999999;
    box-shadow: 0 4px 12px rgba(0,0,0,0.15);
    transition: opacity 0.3s;
  `;
  document.body.appendChild(notification);
  setTimeout(() => {
    notification.style.opacity = '0';
    setTimeout(() => notification.remove(), 300);
  }, 3000);
}
