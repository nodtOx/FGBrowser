<script lang="ts">
  import { appUpdateStatus, checkForUpdates } from '$lib/stores/appUpdater';
  import { invoke } from '@tauri-apps/api/core';
  import { downloadDir } from '@tauri-apps/api/path';
  import { onMount } from 'svelte';

  let activeSection = 'general';
  let defaultDownloadPath = '';
  let crawlerRunning = false;
  let crawlerProgress = '';
  let crawlerMaxPages = 50;

  // General Settings
  let autoStart = false;
  let minimizeToTray = false;
  let closeToTray = false;
  let notifications = true;

  // Download Settings
  let downloadPath = '';
  let maxSimultaneousDownloads = 3;
  let autoStartDownloads = true;
  let seedAfterComplete = true;
  let seedRatio = 1.5;

  // Network Settings
  let maxDownloadSpeed = 0; // 0 = unlimited
  let maxUploadSpeed = 500; // KB/s
  let port = 6881;
  let useUPnP = true;
  let useDHT = true;

  // Appearance Settings
  let fontSize = 14;
  let compactMode = false;
  let showThumbnails = true;
  let animationsEnabled = true;

  // Database Settings
  let dbPath = '../repacks.db';
  let autoRefresh = false;
  let refreshInterval = 24; // hours

  function selectSection(section: string) {
    activeSection = section;
  }

  // Load settings from database
  async function loadSettings() {
    try {
      const settings: any = await invoke('get_settings');

      autoStart = settings.auto_start ?? false;
      minimizeToTray = settings.minimize_to_tray ?? false;
      closeToTray = settings.close_to_tray ?? false;
      notifications = settings.notifications ?? true;
      downloadPath = settings.download_path || defaultDownloadPath;
      maxSimultaneousDownloads = settings.max_simultaneous_downloads ?? 3;
      autoStartDownloads = settings.auto_start_downloads ?? true;
      seedAfterComplete = settings.seed_after_complete ?? true;
      seedRatio = settings.seed_ratio ?? 1.5;
      maxDownloadSpeed = settings.max_download_speed ?? 0;
      maxUploadSpeed = settings.max_upload_speed ?? 500;
      port = settings.port ?? 6881;
      useUPnP = settings.use_upnp ?? true;
      useDHT = settings.use_dht ?? true;
      fontSize = settings.font_size ?? 14;
      compactMode = settings.compact_mode ?? false;
      showThumbnails = settings.show_thumbnails ?? true;
      animationsEnabled = settings.animations_enabled ?? true;
      dbPath = settings.db_path || '../repacks.db';
      autoRefresh = settings.auto_refresh ?? false;
      refreshInterval = settings.refresh_interval ?? 24;
    } catch (e) {
      console.error('Failed to load settings:', e);
    }
  }

  // Auto-save settings to database
  async function saveSettings() {
    try {
      await invoke('save_settings', {
        settings: {
          auto_start: autoStart,
          minimize_to_tray: minimizeToTray,
          close_to_tray: closeToTray,
          notifications: notifications,
          download_path: downloadPath,
          max_simultaneous_downloads: maxSimultaneousDownloads,
          auto_start_downloads: autoStartDownloads,
          seed_after_complete: seedAfterComplete,
          seed_ratio: seedRatio,
          max_download_speed: maxDownloadSpeed,
          max_upload_speed: maxUploadSpeed,
          port: port,
          use_upnp: useUPnP,
          use_dht: useDHT,
          font_size: fontSize,
          compact_mode: compactMode,
          show_thumbnails: showThumbnails,
          animations_enabled: animationsEnabled,
          db_path: dbPath,
          auto_refresh: autoRefresh,
          refresh_interval: refreshInterval,
        },
      });
    } catch (e) {
      console.error('Failed to save settings:', e);
    }
  }

  // Debounced auto-save whenever any setting changes
  let saveTimeout: number;
  let settingsLoaded = false;

  $: {
    if (typeof window !== 'undefined' && settingsLoaded) {
      autoStart,
        minimizeToTray,
        closeToTray,
        notifications,
        downloadPath,
        maxSimultaneousDownloads,
        autoStartDownloads,
        seedAfterComplete,
        seedRatio,
        maxDownloadSpeed,
        maxUploadSpeed,
        port,
        useUPnP,
        useDHT,
        fontSize,
        compactMode,
        showThumbnails,
        animationsEnabled,
        dbPath,
        autoRefresh,
        refreshInterval;

      // Debounce saves to avoid excessive writes
      clearTimeout(saveTimeout);
      saveTimeout = setTimeout(() => {
        saveSettings();
      }, 500);
    }
  }

  function browseFolder(setting: 'download' | 'database') {
    // TODO: Implement folder browser dialog with Tauri
    console.log('Browse folder for:', setting);
  }

  async function runCrawler() {
    if (crawlerRunning) return;

    crawlerRunning = true;
    crawlerProgress = 'Starting crawler...';

    try {
      console.log('⚙️ Settings calling start_crawler (no parameters - hardcoded in Rust)');
      const result: any = await invoke('start_crawler');

      crawlerProgress = `Completed! Found ${result.total_games} games`;

      // Reset after a delay
      setTimeout(() => {
        crawlerProgress = '';
        crawlerRunning = false;
      }, 5000);
    } catch (error) {
      crawlerProgress = `Error: ${error}`;
      console.error('Crawler error:', error);

      setTimeout(() => {
        crawlerProgress = '';
        crawlerRunning = false;
      }, 5000);
    }
  }

  function resetSettings() {
    if (confirm('Are you sure you want to reset all settings to default values?')) {
      // Reset to defaults
      autoStart = false;
      minimizeToTray = false;
      closeToTray = false;
      notifications = true;
      downloadPath = defaultDownloadPath;
      maxSimultaneousDownloads = 3;
      autoStartDownloads = true;
      seedAfterComplete = true;
      seedRatio = 1.5;
      maxDownloadSpeed = 0;
      maxUploadSpeed = 500;
      port = 6881;
      useUPnP = true;
      useDHT = true;
      fontSize = 14;
      compactMode = false;
      showThumbnails = true;
      animationsEnabled = true;
      dbPath = '../repacks.db';
      autoRefresh = false;
      refreshInterval = 24;
      console.log('Settings reset to defaults');
    }
  }

  onMount(async () => {
    // Get the user's Downloads directory
    try {
      const userDownloadDir = await downloadDir();
      // downloadDir() already includes trailing separator
      defaultDownloadPath = `${userDownloadDir}FitGirl`;
    } catch (e) {
      console.error('Failed to get download directory:', e);
      defaultDownloadPath = 'Downloads/FitGirl';
    }

    // Load settings from database
    await loadSettings();
    settingsLoaded = true;
  });
</script>

<div class="settings-layout">
  <!-- Sidebar -->
  <div class="settings-sidebar">
    <div class="sidebar-section">
      <button class="sidebar-item" class:active={activeSection === 'general'} on:click={() => selectSection('general')}>
        General
      </button>
      <button
        class="sidebar-item"
        class:active={activeSection === 'downloads'}
        on:click={() => selectSection('downloads')}
      >
        Downloads
      </button>
      <button class="sidebar-item" class:active={activeSection === 'network'} on:click={() => selectSection('network')}>
        Network
      </button>
      <button
        class="sidebar-item"
        class:active={activeSection === 'appearance'}
        on:click={() => selectSection('appearance')}
      >
        Appearance
      </button>
      <button
        class="sidebar-item"
        class:active={activeSection === 'database'}
        on:click={() => selectSection('database')}
      >
        Database
      </button>
    </div>

    <div class="sidebar-spacer"></div>

    <div class="sidebar-section">
      <button class="sidebar-item" class:active={activeSection === 'about'} on:click={() => selectSection('about')}>
        About
      </button>
    </div>
  </div>

  <!-- Main Content -->
  <div class="settings-content">
    <div class="settings-header">
      <h2 class="settings-title">
        {activeSection === 'general'
          ? 'General Settings'
          : activeSection === 'downloads'
            ? 'Download Settings'
            : activeSection === 'network'
              ? 'Network Settings'
              : activeSection === 'appearance'
                ? 'Appearance Settings'
                : activeSection === 'database'
                  ? 'Database Settings'
                  : activeSection === 'about'
                    ? 'About'
                    : ''}
      </h2>
    </div>

    <div class="settings-panel">
      {#if activeSection === 'general'}
        <div class="setting-group">
          <div class="group-title">Startup</div>

          <div class="setting-item">
            <label class="setting-label">
              <input type="checkbox" bind:checked={autoStart} />
              <span>Launch on system startup</span>
            </label>
            <div class="setting-description">Automatically start FitGirl Browser when you log in</div>
          </div>

          <div class="setting-item">
            <label class="setting-label">
              <input type="checkbox" bind:checked={minimizeToTray} />
              <span>Minimize to system tray</span>
            </label>
            <div class="setting-description">Hide window to tray instead of minimizing to taskbar</div>
          </div>

          <div class="setting-item">
            <label class="setting-label">
              <input type="checkbox" bind:checked={closeToTray} />
              <span>Close to system tray</span>
            </label>
            <div class="setting-description">Keep app running in tray when closing window</div>
          </div>
        </div>

        <div class="setting-group">
          <div class="group-title">Notifications</div>

          <div class="setting-item">
            <label class="setting-label">
              <input type="checkbox" bind:checked={notifications} />
              <span>Enable notifications</span>
            </label>
            <div class="setting-description">Show desktop notifications for downloads and updates</div>
          </div>
        </div>

        <div class="setting-group">
          <div class="group-title">Privacy & Telemetry</div>

          <div class="setting-item">
            <div class="setting-description" style="padding: 12px; background: rgba(99, 102, 241, 0.1); border-radius: 6px; border-left: 3px solid rgb(99, 102, 241);">
              <strong>Anonymous Usage Tracking:</strong> This app automatically sends anonymous crash reports and usage statistics to help improve the experience.
              No personal data or game information is collected. Powered by Sentry.
            </div>
          </div>
        </div>
      {:else if activeSection === 'downloads'}
        <div class="setting-group">
          <div class="group-title">Download Location</div>

          <div class="setting-item">
            <div class="setting-label-text">Download folder</div>
            <div class="input-with-button">
              <input type="text" class="setting-input" bind:value={downloadPath} />
              <button class="browse-button" on:click={() => browseFolder('download')}>Browse</button>
            </div>
            <div class="setting-description">Default location for downloaded games</div>
          </div>
        </div>

        <div class="setting-group">
          <div class="group-title">Download Behavior</div>

          <div class="setting-item">
            <div class="setting-label-text">Maximum simultaneous downloads</div>
            <input type="number" class="setting-input" bind:value={maxSimultaneousDownloads} min="1" max="10" />
            <div class="setting-description">Number of games to download at the same time</div>
          </div>

          <div class="setting-item">
            <label class="setting-label">
              <input type="checkbox" bind:checked={autoStartDownloads} />
              <span>Auto-start downloads</span>
            </label>
            <div class="setting-description">Automatically start downloads when added to queue</div>
          </div>

          <div class="setting-item">
            <label class="setting-label">
              <input type="checkbox" bind:checked={seedAfterComplete} />
              <span>Seed after completion</span>
            </label>
            <div class="setting-description">Continue seeding after download completes</div>
          </div>

          <div class="setting-item">
            <div class="setting-label-text">Seed ratio limit</div>
            <input type="number" class="setting-input" bind:value={seedRatio} min="0" max="10" step="0.1" />
            <div class="setting-description">Stop seeding after reaching this ratio (0 = unlimited)</div>
          </div>
        </div>
      {:else if activeSection === 'network'}
        <div class="setting-group">
          <div class="group-title">Bandwidth Limits</div>

          <div class="setting-item">
            <div class="setting-label-text">Maximum download speed (KB/s)</div>
            <input type="number" class="setting-input" bind:value={maxDownloadSpeed} min="0" step="100" />
            <div class="setting-description">0 = unlimited</div>
          </div>

          <div class="setting-item">
            <div class="setting-label-text">Maximum upload speed (KB/s)</div>
            <input type="number" class="setting-input" bind:value={maxUploadSpeed} min="0" step="50" />
            <div class="setting-description">0 = unlimited</div>
          </div>
        </div>

        <div class="setting-group">
          <div class="group-title">Connection</div>

          <div class="setting-item">
            <div class="setting-label-text">Listening port</div>
            <input type="number" class="setting-input" bind:value={port} min="1024" max="65535" />
            <div class="setting-description">Port for incoming connections (1024-65535)</div>
          </div>

          <div class="setting-item">
            <label class="setting-label">
              <input type="checkbox" bind:checked={useUPnP} />
              <span>Enable UPnP/NAT-PMP</span>
            </label>
            <div class="setting-description">Automatically configure router port forwarding</div>
          </div>

          <div class="setting-item">
            <label class="setting-label">
              <input type="checkbox" bind:checked={useDHT} />
              <span>Enable DHT</span>
            </label>
            <div class="setting-description">Use Distributed Hash Table for peer discovery</div>
          </div>
        </div>
      {:else if activeSection === 'appearance'}
        <div class="setting-group">
          <div class="group-title">Display</div>

          <div class="setting-item">
            <div class="setting-label-text">Font size</div>
            <input type="range" class="setting-slider" bind:value={fontSize} min="10" max="20" step="1" />
            <div class="setting-value">{fontSize}px</div>
            <div class="setting-description">Base font size for the interface</div>
          </div>

          <div class="setting-item">
            <label class="setting-label">
              <input type="checkbox" bind:checked={compactMode} />
              <span>Compact mode</span>
            </label>
            <div class="setting-description">Reduce spacing and padding for more content on screen</div>
          </div>

          <div class="setting-item">
            <label class="setting-label">
              <input type="checkbox" bind:checked={showThumbnails} />
              <span>Show thumbnails</span>
            </label>
            <div class="setting-description">Display game thumbnails in list view (when available)</div>
          </div>

          <div class="setting-item">
            <label class="setting-label">
              <input type="checkbox" bind:checked={animationsEnabled} />
              <span>Enable animations</span>
            </label>
            <div class="setting-description">Smooth transitions and animations</div>
          </div>
        </div>

        <div class="setting-group">
          <div class="group-title">Theme</div>

          <div class="setting-item">
            <div class="setting-description">
              Use the theme selector in the header (press T) to change themes. Custom themes can be added to the
              themes/custom/ folder.
            </div>
          </div>
        </div>
      {:else if activeSection === 'database'}
        <div class="setting-group">
          <div class="group-title">Database Location</div>

          <div class="setting-item">
            <div class="setting-label-text">Database path</div>
            <div class="input-with-button">
              <input type="text" class="setting-input" bind:value={dbPath} />
              <button class="browse-button" on:click={() => browseFolder('database')}>Browse</button>
            </div>
            <div class="setting-description">Path to repacks.db file</div>
          </div>
        </div>

        <div class="setting-group">
          <div class="group-title">Manual Crawler</div>

          <div class="setting-item">
            <div class="setting-label-text">Number of pages to crawl</div>
            <input type="number" class="setting-input" bind:value={crawlerMaxPages} min="1" max="200" />
            <div class="setting-description">How many pages to fetch (0 = crawl all pages)</div>
          </div>

          <div class="setting-item">
            <button class="action-button primary" on:click={runCrawler} disabled={crawlerRunning}>
              {crawlerRunning ? 'Crawling...' : 'Run Crawler Now'}
            </button>
            {#if crawlerProgress}
              <div class="crawler-status">{crawlerProgress}</div>
            {/if}
            <div class="setting-description">Manually trigger the crawler to update the database</div>
          </div>
        </div>

        <div class="setting-group">
          <div class="group-title">Auto-Refresh</div>

          <div class="setting-item">
            <label class="setting-label">
              <input type="checkbox" bind:checked={autoRefresh} />
              <span>Auto-refresh database</span>
            </label>
            <div class="setting-description">Automatically run crawler to update database</div>
          </div>

          <div class="setting-item">
            <div class="setting-label-text">Refresh interval (hours)</div>
            <input type="number" class="setting-input" bind:value={refreshInterval} min="1" max="168" />
            <div class="setting-description">How often to refresh the database automatically</div>
          </div>
        </div>
      {:else if activeSection === 'about'}
        <div class="about-content">
          <div class="about-logo">
            <div class="logo-text">FitGirl Browser</div>
          </div>

          <div class="about-info">
            <div class="info-item">
              <span class="info-label">Version:</span>
              <span class="info-value">0.1.1</span>
            </div>
            <div class="info-item">
              <span class="info-label">Build:</span>
              <span class="info-value">2025.01.18</span>
            </div>
            <div class="info-item">
              <span class="info-label">Framework:</span>
              <span class="info-value">Tauri 2.0 + Svelte</span>
            </div>
          </div>

          <div class="update-check-section">
            <button 
              class="btn-check-update" 
              on:click={() => checkForUpdates(false)}
              disabled={$appUpdateStatus.checking}
            >
              {#if $appUpdateStatus.checking}
                Checking for updates...
              {:else if $appUpdateStatus.available}
                Update Available! (v{$appUpdateStatus.latestVersion})
              {:else if $appUpdateStatus.error}
                Check Failed - Try Again
              {:else}
                Check for Updates
              {/if}
            </button>
            {#if $appUpdateStatus.available}
              <p class="update-available-text">
                A new version is available. The notification will appear in the top-right corner.
              </p>
            {:else if !$appUpdateStatus.checking && !$appUpdateStatus.error}
              <p class="update-info-text">
                App automatically checks for updates on startup.
              </p>
            {/if}
          </div>

          <div class="about-description">
            A beautiful desktop application for browsing FitGirl repacks. Inspired by PSVita's PKGj interface.
          </div>

          <div class="about-links">
            <button class="link-button">Check for Updates</button>
            <button class="link-button">View Documentation</button>
            <button class="link-button">Report Issue</button>
          </div>

          <div class="about-copyright">MIT License - Feel free to use, modify, and distribute</div>
        </div>
      {/if}
    </div>

    {#if activeSection !== 'about'}
      <div class="settings-footer">
        <div class="settings-footer-info">
          <span class="auto-save-indicator">Settings are saved automatically</span>
        </div>
        <button class="action-button" on:click={resetSettings}>Reset to Defaults</button>
      </div>
    {/if}
  </div>
</div>

<style>
  .settings-layout {
    display: flex;
    height: 100%;
    overflow: hidden;
  }

  /* Sidebar Styles */
  .settings-sidebar {
    width: var(--sidebar-width);
    background-color: var(--color-background);
    border-right: 1px solid var(--color-border);
    overflow-y: auto;
    padding: 8px 0;
    display: flex;
    flex-direction: column;
    gap: 16px;
    font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
    font-size: calc(var(--base-font-size) * 0.9);
  }

  .sidebar-section {
    display: flex;
    flex-direction: column;
  }

  .sidebar-spacer {
    flex: 1;
  }

  .sidebar-item {
    padding: 3px 12px;
    color: var(--color-textSecondary);
    cursor: pointer;
    transition: var(--transition);
    line-height: 1.4;
    background: none;
    border: none;
    text-align: left;
    width: 100%;
    font-family: inherit;
    font-size: inherit;
  }

  .sidebar-item:hover {
    background-color: var(--color-hover);
    color: var(--color-text);
  }

  .sidebar-item.active {
    background-color: var(--color-primary);
    color: var(--color-selectedText);
    font-weight: 600;
  }

  /* Content Styles */
  .settings-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .settings-header {
    padding: 12px 16px;
    background-color: var(--color-backgroundSecondary);
    border-bottom: 1px solid var(--color-border);
  }

  .settings-title {
    font-size: 18px;
    font-weight: 600;
    color: var(--color-text);
    margin: 0;
    font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  }

  .settings-panel {
    flex: 1;
    overflow-y: auto;
    padding: 20px;
  }

  /* Setting Groups */
  .setting-group {
    margin-bottom: 32px;
  }

  .group-title {
    font-size: 16px;
    font-weight: 600;
    color: var(--color-primary);
    margin-bottom: 16px;
    padding-bottom: 8px;
    border-bottom: 1px solid var(--color-border);
    font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  }

  .setting-item {
    margin-bottom: 20px;
  }

  .setting-label {
    display: flex;
    align-items: center;
    gap: 10px;
    cursor: pointer;
    color: var(--color-text);
    font-size: 14px;
    font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  }

  .setting-label input[type='checkbox'] {
    width: 18px;
    height: 18px;
    cursor: pointer;
    accent-color: var(--color-primary);
  }

  .setting-label-text {
    font-size: 14px;
    font-weight: 500;
    color: var(--color-text);
    margin-bottom: 8px;
    font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  }

  .setting-description {
    font-size: 12px;
    color: var(--color-textMuted);
    margin-top: 4px;
    margin-left: 28px;
    font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  }

  /* Input Styles */
  .setting-input {
    width: 100%;
    max-width: 500px;
    padding: 10px 12px;
    background-color: var(--color-background);
    border: 1px solid var(--color-border);
    border-radius: var(--border-radius);
    color: var(--color-text);
    font-size: 13px;
    outline: none;
    transition: var(--transition);
    font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  }

  .setting-input:focus {
    border-color: var(--color-primary);
    box-shadow: 0 0 0 2px rgba(233, 69, 96, 0.2);
  }

  .input-with-button {
    display: flex;
    gap: 8px;
    max-width: 600px;
  }

  .input-with-button .setting-input {
    flex: 1;
  }

  .browse-button {
    padding: 10px 20px;
    background-color: var(--color-backgroundTertiary);
    border: 1px solid var(--color-border);
    border-radius: var(--border-radius);
    color: var(--color-text);
    font-size: 13px;
    cursor: pointer;
    transition: var(--transition);
    font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  }

  .browse-button:hover {
    background-color: var(--color-hover);
    border-color: var(--color-primary);
  }

  .setting-slider {
    width: 100%;
    max-width: 300px;
    height: 6px;
    background: var(--color-border);
    border-radius: var(--border-radius);
    outline: none;
    appearance: none;
    -webkit-appearance: none;
  }

  .setting-slider::-webkit-slider-thumb {
    appearance: none;
    -webkit-appearance: none;
    width: 18px;
    height: 18px;
    border-radius: 50%;
    background: var(--color-primary);
    cursor: pointer;
  }

  .setting-slider::-moz-range-thumb {
    width: 18px;
    height: 18px;
    border-radius: 50%;
    background: var(--color-primary);
    cursor: pointer;
    border: none;
  }

  .setting-value {
    display: inline-block;
    margin-left: 12px;
    color: var(--color-primary);
    font-weight: 600;
    font-size: 14px;
    min-width: 40px;
    font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  }

  /* Footer */
  .settings-footer {
    padding: 16px 20px;
    background-color: var(--color-backgroundSecondary);
    border-top: 1px solid var(--color-border);
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 12px;
  }

  .settings-footer-info {
    flex: 1;
  }

  .auto-save-indicator {
    font-size: 13px;
    color: var(--color-textMuted);
    font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
    font-style: italic;
  }

  .action-button {
    padding: 10px 24px;
    background-color: var(--color-backgroundTertiary);
    border: 1px solid var(--color-border);
    border-radius: var(--border-radius);
    color: var(--color-text);
    font-size: 14px;
    cursor: pointer;
    transition: var(--transition);
    font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
    font-weight: 500;
  }

  .action-button:hover {
    background-color: var(--color-hover);
    border-color: var(--color-primary);
  }

  .action-button:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .crawler-status {
    margin-top: 12px;
    padding: 10px 12px;
    background-color: var(--color-backgroundTertiary);
    border: 1px solid var(--color-border);
    border-radius: var(--border-radius);
    color: var(--color-text);
    font-size: 13px;
    font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  }

  /* About Section */
  .about-content {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 32px;
    padding: 40px;
    max-width: 600px;
    margin: 0 auto;
  }

  .about-logo {
    text-align: center;
  }

  .logo-text {
    font-size: 32px;
    font-weight: 700;
    color: var(--color-primary);
    font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
    letter-spacing: 1px;
  }

  .about-info {
    display: flex;
    flex-direction: column;
    gap: 12px;
    width: 100%;
  }

  .info-item {
    display: flex;
    justify-content: space-between;
    padding: 8px 0;
    border-bottom: 1px solid var(--color-border);
    font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  }

  .info-label {
    font-weight: 600;
    color: var(--color-textSecondary);
    font-size: 14px;
  }

  .info-value {
    color: var(--color-text);
    font-size: 14px;
  }

  .about-description {
    text-align: center;
    color: var(--color-textSecondary);
    font-size: 14px;
    line-height: 1.6;
    font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  }

  .about-links {
    display: flex;
    flex-direction: column;
    gap: 8px;
    width: 100%;
  }

  .update-check-section {
    margin: 24px 0;
    padding: 20px;
    background: var(--color-backgroundSecondary);
    border-radius: var(--border-radius);
    border: 1px solid var(--color-border);
    text-align: center;
  }

  .btn-check-update {
    padding: 12px 32px;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    color: white;
    border: none;
    border-radius: 8px;
    font-weight: 600;
    cursor: pointer;
    font-size: 14px;
    transition: all 0.2s;
    min-width: 200px;
  }

  .btn-check-update:hover:not(:disabled) {
    transform: translateY(-2px);
    box-shadow: 0 4px 12px rgba(102, 126, 234, 0.4);
  }

  .btn-check-update:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .update-available-text {
    margin-top: 12px;
    color: #10b981;
    font-size: 14px;
    font-weight: 500;
  }

  .update-info-text {
    margin-top: 12px;
    color: var(--color-textSecondary);
    font-size: 13px;
  }

  .link-button {
    padding: 12px 24px;
    background-color: var(--color-backgroundTertiary);
    border: 1px solid var(--color-border);
    border-radius: var(--border-radius);
    color: var(--color-text);
    font-size: 14px;
    cursor: pointer;
    transition: var(--transition);
    font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  }

  .link-button:hover {
    background-color: var(--color-hover);
    border-color: var(--color-primary);
  }

  .about-copyright {
    text-align: center;
    color: var(--color-textMuted);
    font-size: 12px;
    font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  }
</style>
