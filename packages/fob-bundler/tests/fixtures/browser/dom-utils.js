/**
 * Browser DOM utilities
 */

export function updateUI(elementId, content) {
  const element = document.getElementById(elementId);
  if (element) {
    element.textContent = content;
  }
}

export function attachEventListeners() {
  const button = document.querySelector('.action-button');
  if (button) {
    button.addEventListener('click', handleClick);
  }
}

function handleClick(event) {
  event.preventDefault();

  // Use localStorage
  const clickCount = parseInt(localStorage.getItem('clickCount') || '0', 10);
  localStorage.setItem('clickCount', String(clickCount + 1));

  // Update UI
  updateUI('counter', `Clicks: ${clickCount + 1}`);
}

export function getStoredData(key) {
  return localStorage.getItem(key);
}

export function setStoredData(key, value) {
  localStorage.setItem(key, value);
}
