// Settings window entry point
import { invoke } from '@tauri-apps/api/core';

// Settings state
interface AppSettings {
  window_mode: string;
  window_width: number;
  window_height: number;
  custom_width: number | null;
  custom_height: number | null;
}

interface TraySettings {
  minimize_to_tray: boolean;
  start_minimized: boolean;
}

// Standard window resolutions
const STANDARD_RESOLUTIONS = [
  { label: '800x600', width: 800, height: 600 },
  { label: '1024x768', width: 1024, height: 768 },
  { label: '1280x720', width: 1280, height: 720 },
  { label: '1366x768', width: 1366, height: 768 },
  { label: '1600x900', width: 1600, height: 900 },
  { label: '1920x1080', width: 1920, height: 1080 },
];

let currentSettings: AppSettings = {
  window_mode: 'windowed',
  window_width: 1200,
  window_height: 800,
  custom_width: null,
  custom_height: null,
};

let currentTraySettings: TraySettings = {
  minimize_to_tray: false,
  start_minimized: false,
};

// Initialize the settings window
async function init() {
  try {
    // Load settings
    currentSettings = await invoke<AppSettings>('get_settings');
    currentTraySettings = await invoke<TraySettings>('tray_get_state');
    console.log('Settings loaded:', currentSettings);
    console.log('Tray settings loaded:', currentTraySettings);

    // Render the UI
    renderSettings();

    // Close window on Escape key
    document.addEventListener('keydown', (e) => {
      if (e.key === 'Escape') {
        closeWindow();
      }
    });

    console.log('Settings window initialized');
  } catch (error) {
    console.error('Failed to initialize settings:', error);
  }
}

function renderSettings() {
  const app = document.getElementById('app');
  if (!app) return;

  app.innerHTML = `
    <div class="settings-container">
      <header class="settings-header">
        <h2>Settings</h2>
        <button id="close-btn" class="close-btn" title="Close (Esc)">&times;</button>
      </header>
      <div class="settings-content">
        <section class="settings-section">
          <h3>Window Mode</h3>
          <div class="setting-row">
            <span>Current: <strong id="current-mode">${currentSettings.window_mode === 'fullscreen' ? 'Fullscreen' : 'Windowed'}</strong></span>
            <button id="toggle-fullscreen-btn" class="btn btn-primary">Toggle Fullscreen (F11)</button>
          </div>
        </section>

        <section class="settings-section">
          <h3>Window Size</h3>
          <div class="setting-row">
            <span>Current: <strong id="current-resolution">${currentSettings.window_width} x ${currentSettings.window_height}</strong></span>
          </div>
          <div class="resolutions-grid" id="resolutions-grid">
            ${STANDARD_RESOLUTIONS.map(res => `
              <button class="resolution-btn" data-width="${res.width}" data-height="${res.height}">
                ${res.label}
              </button>
            `).join('')}
          </div>
        </section>

        <section class="settings-section">
          <h3>Custom Resolution</h3>
          <div class="setting-row">
            <label>
              Width:
              <input type="number" id="custom-width" min="800" max="7680" placeholder="Width" />
            </label>
            <label>
              Height:
              <input type="number" id="custom-height" min="600" max="4320" placeholder="Height" />
            </label>
            <button id="apply-custom-resolution" class="btn btn-secondary">Apply</button>
          </div>
        </section>

        <section class="settings-section">
          <h3>System Tray</h3>
          <div class="setting-row setting-toggle">
            <span>Minimize to tray on close</span>
            <label class="toggle-switch">
              <input type="checkbox" id="minimize-to-tray" ${currentTraySettings.minimize_to_tray ? 'checked' : ''}>
              <span class="toggle-slider"></span>
            </label>
          </div>
          <div class="setting-row setting-toggle">
            <span>Start minimized to tray</span>
            <label class="toggle-switch">
              <input type="checkbox" id="start-minimized" ${currentTraySettings.start_minimized ? 'checked' : ''}>
              <span class="toggle-slider"></span>
            </label>
          </div>
        </section>
      </div>
    </div>
  `;

  setupEventListeners();
}

function setupEventListeners() {
  // Close button
  const closeBtn = document.getElementById('close-btn');
  closeBtn?.addEventListener('click', closeWindow);

  // Toggle fullscreen button
  const toggleBtn = document.getElementById('toggle-fullscreen-btn');
  toggleBtn?.addEventListener('click', toggleFullscreen);

  // Resolution buttons
  document.querySelectorAll('.resolution-btn').forEach((btn) => {
    btn.addEventListener('click', () => {
      const width = parseInt(btn.getAttribute('data-width') || '0', 10);
      const height = parseInt(btn.getAttribute('data-height') || '0', 10);
      setWindowSize(width, height);
    });
  });

  // Custom resolution apply button
  const applyBtn = document.getElementById('apply-custom-resolution');
  applyBtn?.addEventListener('click', setCustomResolution);

  // Enter key in custom inputs
  const customWidth = document.getElementById('custom-width') as HTMLInputElement;
  const customHeight = document.getElementById('custom-height') as HTMLInputElement;
  [customWidth, customHeight].forEach((input) => {
    input?.addEventListener('keydown', (e) => {
      if (e.key === 'Enter') {
        setCustomResolution();
      }
    });
  });

  // Tray settings toggles
  const minimizeToTray = document.getElementById('minimize-to-tray') as HTMLInputElement;
  minimizeToTray?.addEventListener('change', () => {
    invoke('tray_set_minimize_to_tray', { minimize: minimizeToTray.checked })
      .catch(console.error);
    currentTraySettings.minimize_to_tray = minimizeToTray.checked;
  });

  const startMinimized = document.getElementById('start-minimized') as HTMLInputElement;
  startMinimized?.addEventListener('change', () => {
    invoke('tray_set_start_minimized', { start_minimized: startMinimized.checked })
      .catch(console.error);
    currentTraySettings.start_minimized = startMinimized.checked;
  });
}

async function toggleFullscreen() {
  try {
    const isFullscreen = await invoke<boolean>('toggle_fullscreen');
    currentSettings.window_mode = isFullscreen ? 'fullscreen' : 'windowed';
    updateUI();
  } catch (error) {
    console.error('Failed to toggle fullscreen:', error);
  }
}

async function setWindowSize(width: number, height: number) {
  try {
    await invoke('set_window_size', { width, height });
    currentSettings.window_width = width;
    currentSettings.window_height = height;
    updateUI();
  } catch (error) {
    console.error('Failed to set window size:', error);
    alert(`Failed to set window size: ${error}`);
  }
}

async function setCustomResolution() {
  const widthInput = document.getElementById('custom-width') as HTMLInputElement;
  const heightInput = document.getElementById('custom-height') as HTMLInputElement;

  const width = parseInt(widthInput?.value || '0', 10);
  const height = parseInt(heightInput?.value || '0', 10);

  if (isNaN(width) || isNaN(height)) {
    alert('Please enter valid numbers for width and height');
    return;
  }

  if (width < 800 || height < 600) {
    alert('Minimum window size is 800x600');
    return;
  }

  try {
    await setWindowSize(width, height);
    currentSettings.custom_width = width;
    currentSettings.custom_height = height;
    await saveSettings();
    updateUI();
  } catch (error) {
    console.error('Failed to set custom resolution:', error);
  }
}

async function saveSettings() {
  try {
    await invoke('save_settings', { settings: currentSettings });
  } catch (error) {
    console.error('Failed to save settings:', error);
  }
}

function updateUI() {
  const modeDisplay = document.getElementById('current-mode');
  if (modeDisplay) {
    modeDisplay.textContent = currentSettings.window_mode === 'fullscreen' ? 'Fullscreen' : 'Windowed';
  }

  const resDisplay = document.getElementById('current-resolution');
  if (resDisplay) {
    resDisplay.textContent = `${currentSettings.window_width} x ${currentSettings.window_height}`;
  }

  const customWidth = document.getElementById('custom-width') as HTMLInputElement;
  const customHeight = document.getElementById('custom-height') as HTMLInputElement;
  if (customWidth && currentSettings.custom_width) {
    customWidth.value = currentSettings.custom_width.toString();
  }
  if (customHeight && currentSettings.custom_height) {
    customHeight.value = currentSettings.custom_height.toString();
  }

  document.querySelectorAll('.resolution-btn').forEach((btn) => {
    const width = parseInt(btn.getAttribute('data-width') || '0', 10);
    const height = parseInt(btn.getAttribute('data-height') || '0', 10);
    if (width === currentSettings.window_width && height === currentSettings.window_height) {
      btn.classList.add('active');
    } else {
      btn.classList.remove('active');
    }
  });
}

function closeWindow() {
  invoke('close_settings_window').catch(console.error);
}

// Initialize when DOM is ready
if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', init);
} else {
  init();
}