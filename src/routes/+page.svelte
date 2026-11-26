<script>
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import { onMount, onDestroy } from 'svelte';

  // Component library imports
  import Button from '$lib/components/Button.svelte';
  import Badge from '$lib/components/Badge.svelte';
  import TextInput from '$lib/components/TextInput.svelte';
  import Toggle from '$lib/components/Toggle.svelte';
  import ToolCard from '$lib/components/ToolCard.svelte';
  import ModalDialog from '$lib/components/ModalDialog.svelte';
  import ModCard from '$lib/components/ModCard.svelte';
  import ModDetailModal from '$lib/components/ModDetailModal.svelte';
  import LaunchModal from '$lib/components/LaunchModal.svelte';
  import VpkInspector from '$lib/components/VpkInspector.svelte';
  import CreateModWizard from '$lib/components/CreateModWizard.svelte';
  import VpkMergeEngine from '$lib/components/VpkMergeEngine.svelte';
  import HeroIcon from '$lib/components/HeroIcon.svelte';
  import DownloadItemCard from '$lib/components/DownloadItemCard.svelte';

  // v1.5 — Import from centralized constants (FIX §2.6, §2.5)
  import { DEADLOCK_HEROES, HERO_NAMES, HERO_ICONS, getHeroIcon } from '$lib/constants/heroes';
  // v1.5 — Stores
  import { initUiScale, setUiScale, initLang, setLang, getLang, getUiScale, SCALE_OPTIONS } from '$lib/stores/settings';
  import { getDownloads, getActiveDownloads, getCompletedDownloads, clearCompleted, setupDownloadListener } from '$lib/stores/downloads';
  import { getPlayer, loadPlayer, setManualAccountId } from '$lib/stores/player';
  import { t } from '$lib/i18n';

  // Hero icon helper (using centralized map, FIX §2.6)
  function getHeroIconLegacy(heroName) {
    return getHeroIcon(heroName);
  }

  const THEMES = [
    { id: 'green', name: 'Stock Emerald', hex: '#2ea069', secondaryHex: '#4ade80' },
    { id: 'purple', name: 'Occult Purple', hex: '#9333ea', secondaryHex: '#c084fc' },
    { id: 'red', name: 'Crimson Strike', hex: '#dc2626', secondaryHex: '#f87171' },
    { id: 'parchment', name: 'Deadlock Sand (#EFE0BE)', hex: '#EFE0BE', secondaryHex: '#d8c49d' },
    { id: 'blue', name: 'Cobalt Astral', hex: '#0284c7', secondaryHex: '#38bdf8' },
  ];

  // ── Application State ──────────────────────────────────────────────────────
  let activeTab = $state('home'); // 'home' | 'mods' | 'characters' | 'installed' | 'downloads' | 'tracker_meta' | 'profile' | 'create_mod' | 'vpk_merge' | 'dashboard' | 'logs' | 'settings'
  let currentTheme = $state('green');
  let deadlockBuildId = $state('26770241');
  let config = $state(null);
  let tools = $state([]);
  let addons = $state([]);
  let logs = $state([]);
  let gameLogs = $state([]);
  let logsFilter = $state('all'); // 'all' | 'info' | 'error' | 'game'
  let progress = $state({}); // tool_id -> InstallProgress

  // FIX §2.1 — Vanilla mode state
  let vanillaActive = $state(false);
  let vanillaIsolatedCount = $state(0);

  // Mod Details & Launch Modals
  let selectedModId = $state(null);
  let selectedModItemType = $state('Mod'); // FIX §2.3: carry real item_type
  let isModModalOpen = $state(false);
  let isLaunchModalOpen = $state(false);

  // Inspector State (Characters tab)
  let selectedModForInspector = $state(null);

  // Community Mods State (GameBanana)
  let modsList = $state([]);
  let modsLoading = $state(false);
  let modsLoadingMore = $state(false);
  let modsError = $state('');
  let modsPage = $state(1);
  let modsTotal = $state(0);
  let searchQuery = $state('');
  let searchDebounceTimer = null;
  let activeCategory = $state('All');
  let selectedHeroFilter = $state('All');
  let modsSort = $state('default');
  let gbLatencyMs = $state(null);
  let isPinging = $state(false);

  // Infinite Scroll Sentinel
  let sentinelEl = $state(null);
  let observer = null;

  // Characters Tab State
  let characterSearch = $state('');
  let selectedCharacter = $state('Abrams');

  // Installed Mods Tab State
  let installedCategoryFilter = $state('All');
  let installedSort = $state('newest');
  let installedSearch = $state('');
  let installedStatus = $state('all');

  // Tracker Meta State — FIX §2.4: reactive filtering
  let metaRoleFilter = $state('All');
  let metaSearchQuery = $state('');
  let selectedBuildHero = $state('Seven');

  // v1.5 — UI settings
  let uiScale = $state(100);
  let uiLang = $state('ru');
  let diskFreeBytes = $state(null);
  let manualAccountInput = $state('');
  let settingsCameraMsg = $state('');

  // Logs state
  let logsCopied = $state(false);
  let logsSaved = $state(false);

  // Confirmation Modals
  let modal = $state(null);
  let locateModal = $state(null);

  function appendLog(level, message) {
    const time = new Date().toLocaleTimeString();
    logs = [{ time, level, message, source: 'app' }, ...logs.slice(0, 300)];
    invoke('write_session_log', { level, message }).catch(() => {});
  }

  function applyTheme(themeId) {
    currentTheme = themeId;
    if (typeof document !== 'undefined') {
      document.documentElement.setAttribute('data-theme', themeId);
      localStorage.setItem('dmbk_app_theme', themeId);
    }
  }

  async function copyLogsToClipboard() {
    if (!logs.length) return;
    const text = logs.map(l => `[${l.time}] [${l.level.toUpperCase()}] ${l.message}`).join('\n');
    try {
      await navigator.clipboard.writeText(text);
      logsCopied = true;
      setTimeout(() => { logsCopied = false; }, 2000);
    } catch (e) {
      console.error('Failed to copy logs', e);
    }
  }

  // ── Lifecycle & IPC ────────────────────────────────────────────────────────
  onMount(async () => {
    // Restore theme
    const savedTheme = localStorage.getItem('dmbk_app_theme') || 'green';
    applyTheme(savedTheme);

    // v1.5 — Restore UI scale and language
    initUiScale();
    initLang();
    uiScale = getUiScale();
    uiLang = getLang();

    appendLog('info', 'System initialization: Deadlock Mod Build Kit v1.5');
    try {
      config = await invoke('get_config');
      tools = config.tools || [];
      // Sync language and scale from config if not overridden in localStorage
      if (!localStorage.getItem('dmbk_lang') && config.ui_language) {
        initLang(config.ui_language);
        uiLang = getLang();
      }
      if (!localStorage.getItem('dmbk_ui_scale') && config.ui_scale) {
        initUiScale(config.ui_scale);
        uiScale = getUiScale();
      }
      appendLog('success', `Environment configuration loaded: ${tools.length} managed tools.`);

      // Free disk space metric (§4.1.3)
      loadDiskFreeSpace();

      // Detect Deadlock Build ID
      try {
        deadlockBuildId = await invoke('get_deadlock_build_version');
      } catch {
        deadlockBuildId = '26770241';
      }

      if (config.deadlock_game_dir) {
        appendLog('success', `Deadlock path verified: ${config.deadlock_game_dir}`);
        loadAddons();
      } else {
        appendLog('warn', 'Deadlock path not found in default libraries. Running Steam discovery.');
        detectDeadlock();
      }

      // Listen for toolchain install progress
      await listen('install_progress', (e) => {
        const payload = e.payload;
        progress = { ...progress, [payload.tool_id]: payload };

        if (payload.state === 'done') {
          appendLog('success', `[${payload.tool_id}] ${payload.message}`);
          refreshConfig();
        } else if (payload.state === 'error') {
          appendLog('error', `[${payload.tool_id}] ${payload.message}`);
        }
      });

      // v1.5 — Setup download listener
      await setupDownloadListener();

      // Load player profile
      loadPlayer();

      // Initial sync with GameBanana
      syncWithGameBanana();
    } catch (err) {
      appendLog('error', `Initialization error: ${err}`);
    }

    // Setup infinite scroll observer
    setupInfiniteScroll();
  });

  onDestroy(() => {
    if (observer) {
      observer.disconnect();
    }
  });

  function setupInfiniteScroll() {
    if (observer) observer.disconnect();
    if (typeof window === 'undefined') return;

    observer = new IntersectionObserver((entries) => {
      const entry = entries[0];
      if (entry && entry.isIntersecting && !modsLoading && !modsLoadingMore && activeTab === 'mods') {
        if (modsList.length < modsTotal) {
          loadMoreMods();
        }
      }
    }, { rootMargin: '200px' });

    if (sentinelEl) {
      observer.observe(sentinelEl);
    }
  }

  $effect(() => {
    if (sentinelEl && observer) {
      observer.observe(sentinelEl);
    }
  });

  async function refreshConfig() {
    try {
      config = await invoke('get_config');
      tools = config.tools || [];
    } catch (e) {
      appendLog('error', `Failed to sync configuration: ${e}`);
    }
  }

  async function loadAddons() {
    try {
      addons = await invoke('get_addons');
    } catch (e) {
      console.error(e);
    }
  }

  // ── GameBanana Community Loader & Latency ───────────────────────────────────
  async function syncWithGameBanana() {
    isPinging = true;
    const start = performance.now();
    try {
      const pingPromise = invoke('ping_gamebanana');
      const loadPromise = loadCommunityMods(1, searchQuery);
      const [pingMs] = await Promise.all([pingPromise, loadPromise]);
      gbLatencyMs = pingMs;
    } catch (e) {
      gbLatencyMs = Math.round(performance.now() - start);
    } finally {
      isPinging = false;
    }
  }

  async function loadCommunityMods(page = 1, search = '') {
    modsLoading = true;
    modsError = '';
    modsPage = page;

    try {
      let query = search.trim();
      if (selectedHeroFilter !== 'All') {
        query = query ? `${query} ${selectedHeroFilter}` : selectedHeroFilter;
      } else if (activeCategory !== 'All' && !query) {
        query = activeCategory;
      }

      const res = await invoke('fetch_gamebanana_mods', {
        page,
        perPage: 20,
        search: query || null,
        sort: modsSort,
      });

      modsList = res.records || [];
      modsTotal = res.total_records || 0;
      appendLog('info', `Synced ${modsList.length} community mods from GameBanana (Sort: ${modsSort}).`);
    } catch (err) {
      modsError = String(err);
      appendLog('error', `Failed to load GameBanana feed: ${err}`);
    } finally {
      modsLoading = false;
    }
  }

  async function loadMoreMods() {
    if (modsLoadingMore || modsList.length >= modsTotal) return;
    modsLoadingMore = true;
    try {
      const nextPage = modsPage + 1;
      let query = searchQuery.trim();
      if (selectedHeroFilter !== 'All') {
        query = query ? `${query} ${selectedHeroFilter}` : selectedHeroFilter;
      } else if (activeCategory !== 'All' && !query) {
        query = activeCategory;
      }

      const res = await invoke('fetch_gamebanana_mods', {
        page: nextPage,
        perPage: 20,
        search: query || null,
        sort: modsSort,
      });

      if (res.records && res.records.length > 0) {
        modsList = [...modsList, ...res.records];
        modsPage = nextPage;
      }
    } catch (err) {
      appendLog('error', `Failed to load more mods: ${err}`);
    } finally {
      modsLoadingMore = false;
    }
  }

  function handleSearchInput(e) {
    searchQuery = e.target.value;
    clearTimeout(searchDebounceTimer);
    searchDebounceTimer = setTimeout(() => {
      loadCommunityMods(1, searchQuery);
    }, 450);
  }

  function handleCategoryFilter(category) {
    activeCategory = category;
    loadCommunityMods(1, searchQuery);
  }

  function handleHeroFilter(hero) {
    selectedHeroFilter = hero;
    loadCommunityMods(1, searchQuery);
  }

  function handleSortChange(sort) {
    modsSort = sort;
    loadCommunityMods(1, searchQuery);
  }

  // FIX §2.3 — openModDetails must carry the real item_type from the feed record
  function openModDetails(id, itemType = 'Mod') {
    selectedModId = id;
    selectedModItemType = itemType;
    isModModalOpen = true;
  }

  // ── Characters Loadout Management ──────────────────────────────────────────
  async function activateCharacterMod(heroName, filename) {
    try {
      addons = await invoke('set_hero_mod', {
        heroName,
        activeFilename: filename || null,
      });
      appendLog('info', `Character loadout updated: ${heroName} -> ${filename || 'Default'}`);
    } catch (err) {
      appendLog('error', `Failed to update character mod: ${err}`);
    }
  }

  // ── Tool Actions ───────────────────────────────────────────────────────────
  async function launchTool(id) {
    appendLog('info', `Executing process [${id}]…`);
    try {
      await invoke('launch_tool', { toolId: id });
    } catch (err) {
      appendLog('error', `Execution failure: ${err}`);
    }
  }

  async function openFolder(id) {
    try {
      await invoke('open_tool_folder', { toolId: id });
    } catch (e) {
      appendLog('error', `Cannot open directory: ${e}`);
    }
  }

  async function openKitDir() {
    try {
      await invoke('open_mod_kit_folder');
    } catch (e) {
      console.error(e);
    }
  }

  async function openAddonsDir() {
    try {
      await invoke('open_game_addons_folder');
    } catch (e) {
      appendLog('error', 'Deadlock citadel directory not configured.');
    }
  }

  // v1.5 — UI Scale & Language
  function changeUiScale(scale) {
    uiScale = scale;
    setUiScale(scale);
  }

  function changeUiLang(lang) {
    uiLang = lang;
    setLang(lang);
  }

  async function loadDiskFreeSpace() {
    try {
      diskFreeBytes = await invoke('get_disk_free_space');
    } catch {
      diskFreeBytes = null;
    }
  }

  function formatDiskBytes(bytes) {
    if (bytes === null || bytes === undefined) return '—';
    const gb = bytes / (1024 * 1024 * 1024);
    if (gb >= 1) return `${gb.toFixed(1)} GB`;
    return `${(bytes / (1024 * 1024)).toFixed(0)} MB`;
  }

  async function applyCameraFixFromSettings() {
    settingsCameraMsg = '';
    try {
      const res = await invoke('fix_mods_camera');
      settingsCameraMsg = res.message;
      appendLog('success', res.message);
    } catch (e) {
      settingsCameraMsg = String(e);
      appendLog('error', `Camera fix failed: ${e}`);
    }
  }

  async function saveLogsToDisk() {
    const combined = [
      ...logs.map(l => ({ ...l, source: 'app' })),
      ...(Array.isArray(gameLogs) ? gameLogs : []),
    ];
    if (!combined.length) return;
    const text = combined
      .map(l => `[${l.time}] [${(l.source === 'game' ? 'GAME' : l.level).toUpperCase()}] ${l.message}`)
      .join('\n');
    try {
      const saved = await invoke('save_logs_dialog', { content: text });
      if (saved) {
        logsSaved = true;
        setTimeout(() => { logsSaved = false; }, 2000);
      }
    } catch (e) {
      appendLog('error', `Failed to save log file: ${e}`);
    }
  }

  // FIX §2.1 — Vanilla launch with physical addon isolation
  async function launchGameVanilla() {
    appendLog('info', 'Launching Deadlock in Vanilla (Clean) mode — isolating mods…');
    try {
      const result = await invoke('launch_game_vanilla');
      vanillaActive = true;
      vanillaIsolatedCount = result.isolated_count;
      appendLog('success', result.message);
    } catch (e) {
      appendLog('error', `Vanilla launch failed: ${e}`);
    }
  }

  // FIX §2.1 — Restore addons after Vanilla
  async function restoreVanillaAddons() {
    try {
      const count = await invoke('restore_vanilla_addons');
      vanillaActive = false;
      vanillaIsolatedCount = 0;
      appendLog('success', `Vanilla mode ended. ${count} mod(s) restored.`);
      loadAddons();
    } catch (e) {
      appendLog('error', `Failed to restore mods: ${e}`);
    }
  }

  // v1.5 — Load game logs from Deadlock console
  async function loadGameLogs() {
    try {
      const entries = await invoke('get_game_logs');
      gameLogs = Array.isArray(entries) ? entries : [];
      appendLog('info', `Loaded ${gameLogs.length} line(s) from Deadlock console log.`);
    } catch (e) {
      appendLog('error', `Failed to read game logs: ${e}`);
    }
  }

  // v1.5 — Open logs folder
  async function openLogsFolder() {
    try { await invoke('open_logs_folder'); } catch (e) {
      appendLog('error', `Cannot open logs folder: ${e}`);
    }
  }

  async function launchGameWithMods() {
    appendLog('info', 'Executing Deadlock with active mods…');
    try {
      await invoke('launch_game');
      isLaunchModalOpen = false;
    } catch (e) {
      appendLog('error', `Game launch failed: ${e}`);
    }
  }

  async function detectDeadlock() {
    appendLog('info', 'Searching Steam library folders…');
    try {
      const res = await invoke('detect_deadlock');
      if (res) {
        appendLog('success', `Deadlock citadel directory located: ${res}`);
        await refreshConfig();
        loadAddons();
      } else {
        appendLog('warn', 'Deadlock was not detected in registered Steam libraries.');
      }
    } catch (e) {
      appendLog('error', `Discovery error: ${e}`);
    }
  }

  // ── Locate Existing Executable ─────────────────────────────────────────────
  function openLocateModal(tool) {
    locateModal = {
      toolId: tool.id,
      toolName: tool.name,
      pathInput: '',
      error: '',
    };
  }

  async function confirmLocate() {
    if (!locateModal || !locateModal.pathInput.trim()) return;
    try {
      config = await invoke('set_custom_tool_path', {
        toolId: locateModal.toolId,
        executablePath: locateModal.pathInput.trim(),
      });
      tools = config.tools;
      appendLog('success', `Linked existing executable for ${locateModal.toolName}`);
      locateModal = null;
    } catch (e) {
      locateModal.error = String(e);
      appendLog('error', `Linking failed: ${e}`);
    }
  }

  // ── Addon Management ───────────────────────────────────────────────────────
  async function toggleAddon(filename, enable) {
    try {
      addons = await invoke('toggle_addon_state', { filename, enable });
      appendLog('info', `Addon ${filename} state updated: ${enable ? 'active' : 'disabled'}`);
    } catch (e) {
      appendLog('error', `Error updating addon state: ${e}`);
    }
  }

  async function deleteAddon(filename) {
    try {
      addons = await invoke('delete_addon', { filename });
      if (selectedModForInspector?.filename === filename) {
        selectedModForInspector = null;
      }
      appendLog('warn', `Addon removed: ${filename}`);
    } catch (e) {
      appendLog('error', `Error removing addon: ${e}`);
    }
  }

  // ── Confirmations ──────────────────────────────────────────────────────────
  function confirmRemoveTool(tool) {
    modal = {
      title: `Remove ${tool.name}?`,
      description: `All binaries in "${tool.install_subdir}" will be permanently removed. Custom projects in citadel/addons will remain untouched.`,
      isDestructive: true,
      onConfirm: async () => {
        try {
          config = await invoke('remove_tool', { toolId: tool.id });
          tools = config.tools;
          delete progress[tool.id];
          progress = { ...progress };
          appendLog('warn', `Toolchain uninstalled: ${tool.name}`);
        } catch (e) {
          appendLog('error', `Uninstallation error: ${e}`);
        }
        modal = null;
      },
    };
  }

  let installedCount = $derived(tools.filter((t) => t.installed).length);
  let totalCount = $derived(tools.length);

  // Selected hero's installed mods
  let selectedHeroMods = $derived(
    addons.filter(a => a.detected_hero?.toLowerCase() === selectedCharacter.toLowerCase())
  );
  let activeHeroSkin = $derived(selectedHeroMods.find(a => a.enabled));

  // Filtered Installed Mods
  let filteredInstalledMods = $derived(
    addons
      .filter((a) => {
        if (installedCategoryFilter !== 'All' && a.category !== installedCategoryFilter) return false;
        if (installedStatus === 'active' && !a.enabled) return false;
        if (installedStatus === 'disabled' && a.enabled) return false;
        if (installedSearch.trim() && !a.filename.toLowerCase().includes(installedSearch.toLowerCase())) return false;
        return true;
      })
      .sort((a, b) => {
        if (installedSort === 'name') return a.filename.localeCompare(b.filename);
        if (installedSort === 'size') return b.size_bytes - a.size_bytes;
        return 0;
      })
  );

  // ── Meta Leaderboard Data (v1.5 Full Roster 45 Heroes, FIX §2.4) ───────────
  const META_HEROES_EXPANDED = [
    { rank: 1,  name: 'Seven',         tier: 'S+', winrate: '55.4%', pickrate: '31.2%', banrate: '22.4%', role: 'Spirit Hypercarry',            gpm: '840', dpm: '2,420', counter: 'Vindicta',    synergy: 'Dynamo' },
    { rank: 2,  name: 'Ivy',           tier: 'S+', winrate: '54.8%', pickrate: '24.1%', banrate: '18.1%', role: 'Support / Airborne Buffer',    gpm: '710', dpm: '1,890', counter: 'Haze',        synergy: 'Seven' },
    { rank: 3,  name: 'Vindicta',      tier: 'S+', winrate: '54.2%', pickrate: '26.8%', banrate: '24.5%', role: 'Long Range Assassin Carry',    gpm: '820', dpm: '2,650', counter: 'Lash',        synergy: 'Ivy' },
    { rank: 4,  name: 'Mina',          tier: 'S+', winrate: '54.0%', pickrate: '29.4%', banrate: '26.1%', role: 'Vampire Burst Mage / Carry',   gpm: '830', dpm: '2,490', counter: 'Dynamo',      synergy: 'Mo & Krill' },
    { rank: 5,  name: 'Rat King',      tier: 'S',  winrate: '53.8%', pickrate: '18.8%', banrate: '12.9%', role: 'Summoner / Area Initiator',    gpm: '810', dpm: '2,290', counter: 'Ivy',         synergy: 'Abrams' },
    { rank: 6,  name: 'Infernus',      tier: 'S',  winrate: '53.4%', pickrate: '28.0%', banrate: '14.2%', role: 'DoT Mage / Carry',             gpm: '790', dpm: '2,380', counter: 'Kelvin',      synergy: 'Abrams' },
    { rank: 7,  name: 'Billy',         tier: 'S',  winrate: '53.2%', pickrate: '25.6%', banrate: '19.3%', role: 'Melee Brawler / Initiator',    gpm: '765', dpm: '2,120', counter: 'Vindicta',    synergy: 'Paige' },
    { rank: 8,  name: 'Apollo',        tier: 'S',  winrate: '53.1%', pickrate: '22.4%', banrate: '15.0%', role: 'Mobile Carry',                 gpm: '835', dpm: '2,440', counter: 'Warden',      synergy: 'Rem' },
    { rank: 9,  name: 'Abrams',        tier: 'A+', winrate: '52.9%', pickrate: '30.5%', banrate: '19.8%', role: 'Initiator / Melee Brawler',    gpm: '750', dpm: '1,980', counter: 'Seven',       synergy: 'Infernus' },
    { rank: 10, name: 'Drifter',       tier: 'A+', winrate: '52.7%', pickrate: '23.8%', banrate: '21.5%', role: 'Stalker Assassin Carry',       gpm: '845', dpm: '2,460', counter: 'Warden',      synergy: 'The Doorman' },
    { rank: 11, name: 'Haze',          tier: 'A+', winrate: '52.6%', pickrate: '34.2%', banrate: '28.0%', role: 'Stealth Bullet Carry',         gpm: '860', dpm: '2,510', counter: 'Warden',      synergy: 'Dynamo' },
    { rank: 12, name: 'Celeste',       tier: 'A+', winrate: '52.4%', pickrate: '19.1%', banrate: '10.3%', role: 'Duelist Spirit Mage',          gpm: '795', dpm: '2,320', counter: 'Lash',        synergy: 'Dynamo' },
    { rank: 13, name: 'The Doorman',   tier: 'A+', winrate: '52.2%', pickrate: '20.4%', banrate: '13.8%', role: 'Portal Utility Initiator',     gpm: '735', dpm: '1,940', counter: 'Infernus',    synergy: 'Dynamo' },
    { rank: 14, name: 'Pocket',        tier: 'A+', winrate: '52.1%', pickrate: '21.0%', banrate: '12.6%', role: 'Elusive Burst Mage',           gpm: '780', dpm: '2,240', counter: 'Shiv',        synergy: 'Ivy' },
    { rank: 15, name: 'Dynamo',        tier: 'A',  winrate: '51.9%', pickrate: '18.9%', banrate: '15.4%', role: 'Teamfight Initiator / Support', gpm: '680', dpm: '1,620', counter: 'Vindicta',   synergy: 'Seven' },
    { rank: 16, name: 'Victor',        tier: 'A',  winrate: '51.8%', pickrate: '21.7%', banrate: '14.0%', role: 'Undying Tank Brawler',         gpm: '755', dpm: '2,030', counter: 'Vindicta',    synergy: 'Kelvin' },
    { rank: 17, name: 'Solomon',       tier: 'A',  winrate: '51.8%', pickrate: '16.5%', banrate: '8.7%',  role: 'Ward & Sustain Support',       gpm: '705', dpm: '1,740', counter: 'Haze',        synergy: 'Seven' },
    { rank: 18, name: 'Rem',           tier: 'A',  winrate: '51.6%', pickrate: '15.2%', banrate: '7.4%',  role: 'Arcane Tome Support',          gpm: '715', dpm: '1,820', counter: 'Calico',      synergy: 'Apollo' },
    { rank: 19, name: 'Lash',          tier: 'A',  winrate: '51.5%', pickrate: '25.0%', banrate: '13.5%', role: 'Grapple Initiator / Brawler',  gpm: '765', dpm: '2,120', counter: 'Pocket',      synergy: 'Dynamo' },
    { rank: 20, name: 'Silver',        tier: 'A',  winrate: '51.4%', pickrate: '17.8%', banrate: '9.1%',  role: 'Cybernetic Marksman Carry',    gpm: '825', dpm: '2,410', counter: 'Abrams',      synergy: 'Kelvin' },
    { rank: 21, name: 'Paige',         tier: 'A',  winrate: '51.4%', pickrate: '19.6%', banrate: '11.2%', role: 'Conjuration Support',          gpm: '695', dpm: '1,760', counter: 'Haze',        synergy: 'Billy' },
    { rank: 22, name: 'Venator',       tier: 'A',  winrate: '51.3%', pickrate: '14.9%', banrate: '6.8%',  role: 'Trap Initiator',               gpm: '745', dpm: '2,010', counter: 'Seven',       synergy: 'Ivy' },
    { rank: 23, name: 'Lady Geist',    tier: 'B+', winrate: '51.2%', pickrate: '19.5%', banrate: '11.0%', role: 'Life Drain Spirit Mage',       gpm: '770', dpm: '2,310', counter: 'Haze',        synergy: 'Abrams' },
    { rank: 24, name: 'Vyper',         tier: 'B+', winrate: '51.1%', pickrate: '22.1%', banrate: '12.4%', role: 'Sliding Venom Carry',          gpm: '815', dpm: '2,390', counter: 'Mo & Krill',  synergy: 'Viscous' },
    { rank: 25, name: 'Violet',        tier: 'B+', winrate: '51.0%', pickrate: '16.0%', banrate: '7.9%',  role: 'Spectral Burst Mage',          gpm: '775', dpm: '2,280', counter: 'Warden',      synergy: 'Mo & Krill' },
    { rank: 26, name: 'Graves',        tier: 'B+', winrate: '50.9%', pickrate: '18.2%', banrate: '10.2%', role: 'Ferocious Melee Brawler',      gpm: '755', dpm: '2,040', counter: 'Vindicta',    synergy: 'Nurse Harrow' },
    { rank: 27, name: 'Warden',        tier: 'B+', winrate: '50.7%', pickrate: '23.4%', banrate: '8.5%',  role: 'Lockdown / Brawler',           gpm: '740', dpm: '1,920', counter: 'Pocket',      synergy: 'Infernus' },
    { rank: 28, name: 'Kelvin',        tier: 'B+', winrate: '50.6%', pickrate: '17.4%', banrate: '9.8%',  role: 'Frost Dome Support',           gpm: '700', dpm: '1,790', counter: 'Lash',        synergy: 'Haze' },
    { rank: 29, name: 'Nurse Harrow',  tier: 'B+', winrate: '50.5%', pickrate: '14.1%', banrate: '5.6%',  role: 'Combat Medic Support',         gpm: '690', dpm: '1,650', counter: 'Vindicta',    synergy: 'Graves' },
    { rank: 30, name: 'Calico',        tier: 'B+', winrate: '50.4%', pickrate: '15.9%', banrate: '6.4%',  role: 'Agile Assassin Carry',         gpm: '800', dpm: '2,260', counter: 'Mo & Krill',  synergy: 'Dynamo' },
    { rank: 31, name: 'Baba',          tier: 'B',  winrate: '50.3%', pickrate: '12.8%', banrate: '4.9%',  role: 'Hex Utility Support',          gpm: '695', dpm: '1,710', counter: 'Haze',        synergy: 'Seven' },
    { rank: 32, name: 'Deadman Danny', tier: 'B',  winrate: '50.2%', pickrate: '13.5%', banrate: '5.1%',  role: 'Frontline Brawler',            gpm: '735', dpm: '1,880', counter: 'Seven',       synergy: 'Ivy' },
    { rank: 33, name: 'Viscous',       tier: 'B',  winrate: '50.1%', pickrate: '17.2%', banrate: '6.2%',  role: 'Goo Brawler / Utility',        gpm: '710', dpm: '1,780', counter: 'Seven',       synergy: 'Abrams' },
    { rank: 34, name: 'Paradox',       tier: 'B',  winrate: '49.9%', pickrate: '16.4%', banrate: '7.1%',  role: 'Time-Swap Mage / Initiator',   gpm: '745', dpm: '1,990', counter: 'Abrams',      synergy: 'Bebop' },
    { rank: 35, name: 'Mirage',        tier: 'B',  winrate: '49.8%', pickrate: '18.0%', banrate: '8.3%',  role: 'Djinn Mark Mage / Carry',      gpm: '785', dpm: '2,180', counter: 'Haze',        synergy: 'Kelvin' },
    { rank: 36, name: 'McGinnis',      tier: 'B',  winrate: '49.7%', pickrate: '20.4%', banrate: '6.9%',  role: 'Turret Siege Brawler',         gpm: '775', dpm: '2,140', counter: 'Vindicta',    synergy: 'Ivy' },
    { rank: 37, name: 'Mo & Krill',    tier: 'B',  winrate: '49.6%', pickrate: '16.8%', banrate: '7.7%',  role: 'Burrow Brawler / Lockdown',    gpm: '720', dpm: '1,760', counter: 'Seven',       synergy: 'Vindicta' },
    { rank: 38, name: 'Wraith',        tier: 'B',  winrate: '49.5%', pickrate: '22.7%', banrate: '9.2%',  role: 'Card Trick Bullet Carry',      gpm: '815', dpm: '2,340', counter: 'Warden',      synergy: 'Dynamo' },
    { rank: 39, name: 'Bebop',         tier: 'B',  winrate: '49.4%', pickrate: '27.6%', banrate: '21.0%', role: 'Hook Initiator / Laser',       gpm: '760', dpm: '2,150', counter: 'Pocket',      synergy: 'Seven' },
    { rank: 40, name: 'Yamato',        tier: 'B',  winrate: '49.2%', pickrate: '19.3%', banrate: '10.4%', role: 'Blade Skirmisher Initiator',   gpm: '770', dpm: '2,110', counter: 'Warden',      synergy: 'Ivy' },
    { rank: 41, name: 'Holliday',      tier: 'C+', winrate: '49.1%', pickrate: '14.5%', banrate: '4.5%',  role: 'Lasso Gunslinger Carry',       gpm: '760', dpm: '2,090', counter: 'Abrams',      synergy: 'Dynamo' },
    { rank: 42, name: 'Shiv',          tier: 'C+', winrate: '48.9%', pickrate: '20.1%', banrate: '9.4%',  role: 'Bleed Brawler',                gpm: '780', dpm: '2,080', counter: 'Vindicta',    synergy: 'Infernus' },
    { rank: 43, name: 'Grey Talon',    tier: 'C+', winrate: '48.6%', pickrate: '16.2%', banrate: '5.8%',  role: 'Charged Bow Carry',            gpm: '755', dpm: '2,210', counter: 'Lash',        synergy: 'Kelvin' },
    { rank: 44, name: 'Wrecker',       tier: 'C',  winrate: '48.2%', pickrate: '11.4%', banrate: '3.2%',  role: 'Heavy Siege Initiator',        gpm: '715', dpm: '1,840', counter: 'Seven',       synergy: 'McGinnis' },
    { rank: 45, name: 'Sinclair',      tier: 'C',  winrate: '47.9%', pickrate: '10.6%', banrate: '4.1%',  role: 'Illusionist Mage',             gpm: '740', dpm: '1,960', counter: 'Haze',        synergy: 'Paradox' },
  ];

  // FIX §2.4 — Reactive filtered meta heroes list
  let filteredMetaHeroes = $derived(
    META_HEROES_EXPANDED.filter(h => {
      if (metaRoleFilter !== 'All') {
        const roleWords = h.role.toLowerCase();
        const filter = metaRoleFilter.toLowerCase();
        const roleMap = {
          carries: ['carry', 'hypercarry', 'assassin', 'sniper', 'marksman', 'gunslinger'],
          brawlers: ['brawler', 'melee', 'tank', 'skirmisher', 'siege'],
          initiators: ['initiator', 'lockdown', 'summoner', 'hook', 'grapple'],
          mages: ['mage', 'spirit', 'burst', 'burn', 'dot', 'arcane', 'illusionist'],
          support: ['support', 'buffer', 'utility', 'medic'],
        };
        const keywords = roleMap[filter] ?? [filter];
        if (!keywords.some(kw => roleWords.includes(kw))) return false;
      }
      if (metaSearchQuery.trim() && !h.name.toLowerCase().includes(metaSearchQuery.toLowerCase())) return false;
      return true;
    })
  );
</script>

<div class="h-screen w-screen flex overflow-hidden bg-[#0c0e12] text-[#f1f5f9] select-none">

  <!-- ═════════════════════════════════════════════════════════════════════════ -->
  <!-- SIDEBAR: TACTICAL MONOCHROME DESIGN (230px)                               -->
  <!-- ═════════════════════════════════════════════════════════════════════════ -->
  <aside
    class="w-[230px] min-w-[230px] bg-[#12151b] border-r border-[#1f2633] flex flex-col justify-between shrink-0 select-none"
    aria-label="Navigation"
  >
    <!-- Top Header & Menu Items -->
    <div class="overflow-y-auto">
      <!-- Brand Header with User Icon & Game Build Version -->
      <div class="px-4 py-3.5 flex items-center gap-3 border-b border-[#1c222e]">
        <img
          src="/app-icon.png"
          alt="DMBK"
          class="w-7 h-7 rounded-[4px] object-contain border border-[#2b3547] shrink-0"
          onerror={(e) => { e.currentTarget.src = '/app-icon.ico'; }}
        />
        <div class="min-w-0">
          <div class="text-xs font-bold text-[#f1f5f9] tracking-tight leading-tight truncate">Deadlock</div>
          <div class="text-[10px] text-[#828fa3] leading-tight truncate">Mod Build Kit</div>
          <div class="text-[9px] font-mono text-[var(--accent-light)] leading-tight truncate mt-0.5">
            Build #{deadlockBuildId}
          </div>
        </div>
      </div>

      <!-- Navigation Groups -->
      <nav class="space-y-4 px-2.5 py-4">

        <!-- GROUP: MODDING -->
        <div class="space-y-1">
          <div class="px-2 pb-1 text-[10px] font-bold uppercase tracking-wider text-[#5d6b82]">
            Modding
          </div>

          <!-- Home Dashboard (v1.5) -->
          <button
            type="button"
            onclick={() => (activeTab = 'home')}
            style={activeTab === 'home' ? 'border-left-color: var(--accent-primary); background-color: var(--accent-active-sidebar); color: var(--accent-light);' : undefined}
            class="w-full h-8 px-2.5 rounded-[4px] flex items-center gap-2 text-xs transition cursor-pointer text-left
              {activeTab === 'home'
                ? 'border-l-2 font-semibold'
                : 'text-[#94a3b8] hover:text-[#f1f5f9] hover:bg-[#181e28]'}"
          >
            <svg class="w-3.5 h-3.5 stroke-current fill-none stroke-[1.8]" viewBox="0 0 24 24">
              <path d="M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z" />
              <polyline points="9 22 9 12 15 12 15 22" />
            </svg>
            <span>{t('nav.home')}</span>
          </button>

          <!-- Browse Mods -->
          <button
            type="button"
            onclick={() => (activeTab = 'mods')}
            style={activeTab === 'mods' ? 'border-left-color: var(--accent-primary); background-color: var(--accent-active-sidebar); color: var(--accent-light);' : undefined}
            class="w-full h-8 px-2.5 rounded-[4px] flex items-center justify-between text-xs transition cursor-pointer text-left
              {activeTab === 'mods'
                ? 'border-l-2 font-semibold'
                : 'text-[#94a3b8] hover:text-[#f1f5f9] hover:bg-[#181e28]'}"
          >
            <span class="flex items-center gap-2">
              <svg class="w-3.5 h-3.5 stroke-current fill-none stroke-[1.8]" viewBox="0 0 24 24">
                <polygon points="12 2 2 7 12 12 22 7 12 2" />
                <polyline points="2 17 12 22 22 17" />
                <polyline points="2 12 12 17 22 12" />
              </svg>
              <span>{t('nav.mods')}</span>
            </span>
            <Badge variant="theme" text="Online" />
          </button>

          <!-- Characters Loadout & Inspector -->
          <button
            type="button"
            onclick={() => { activeTab = 'characters'; loadAddons(); }}
            style={activeTab === 'characters' ? 'border-left-color: var(--accent-primary); background-color: var(--accent-active-sidebar); color: var(--accent-light);' : undefined}
            class="w-full h-8 px-2.5 rounded-[4px] flex items-center justify-between text-xs transition cursor-pointer text-left
              {activeTab === 'characters'
                ? 'border-l-2 font-semibold'
                : 'text-[#94a3b8] hover:text-[#f1f5f9] hover:bg-[#181e28]'}"
          >
            <span class="flex items-center gap-2">
              <svg class="w-3.5 h-3.5 stroke-current fill-none stroke-[1.8]" viewBox="0 0 24 24">
                <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2" />
                <circle cx="12" cy="7" r="4" />
              </svg>
              <span>{t('nav.characters')}</span>
            </span>
          </button>

          <!-- Installed Mods -->
          <button
            type="button"
            onclick={() => { activeTab = 'installed'; loadAddons(); }}
            style={activeTab === 'installed' ? 'border-left-color: var(--accent-primary); background-color: var(--accent-active-sidebar); color: var(--accent-light);' : undefined}
            class="w-full h-8 px-2.5 rounded-[4px] flex items-center justify-between text-xs transition cursor-pointer text-left
              {activeTab === 'installed'
                ? 'border-l-2 font-semibold'
                : 'text-[#94a3b8] hover:text-[#f1f5f9] hover:bg-[#181e28]'}"
          >
            <span class="flex items-center gap-2">
              <svg class="w-3.5 h-3.5 stroke-current fill-none stroke-[1.8]" viewBox="0 0 24 24">
                <path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z" />
                <polyline points="3.27 6.96 12 12.01 20.73 6.96" />
                <line x1="12" y1="22.08" x2="12" y2="12" />
              </svg>
              <span>{t('nav.installed')}</span>
            </span>
            {#if addons.length > 0}
              <Badge variant="neutral" mono={true} text={String(addons.length)} />
            {/if}
          </button>

          <!-- Downloads (v1.5) -->
          <button
            type="button"
            onclick={() => (activeTab = 'downloads')}
            style={activeTab === 'downloads' ? 'border-left-color: var(--accent-primary); background-color: var(--accent-active-sidebar); color: var(--accent-light);' : undefined}
            class="w-full h-8 px-2.5 rounded-[4px] flex items-center justify-between text-xs transition cursor-pointer text-left
              {activeTab === 'downloads'
                ? 'border-l-2 font-semibold'
                : 'text-[#94a3b8] hover:text-[#f1f5f9] hover:bg-[#181e28]'}"
          >
            <span class="flex items-center gap-2">
              <svg class="w-3.5 h-3.5 stroke-current fill-none stroke-[1.8]" viewBox="0 0 24 24">
                <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
                <polyline points="7 10 12 15 17 10" />
                <line x1="12" y1="15" x2="12" y2="3" />
              </svg>
              <span>{t('nav.downloads')}</span>
            </span>
            {#if getActiveDownloads().length > 0}
              <Badge variant="theme" mono={true} text={String(getActiveDownloads().length)} />
            {/if}
          </button>
        </div>

        <!-- GROUP: TRACKER (Renamed from Profile & Steam) -->
        <div class="space-y-1">
          <div class="px-2 pb-1 text-[10px] font-bold uppercase tracking-wider text-[#5d6b82]">
            Tracker
          </div>

          <!-- Tracker Meta -->
          <button
            type="button"
            onclick={() => (activeTab = 'tracker_meta')}
            style={activeTab === 'tracker_meta' ? 'border-left-color: var(--accent-primary); background-color: var(--accent-active-sidebar); color: var(--accent-light);' : undefined}
            class="w-full h-8 px-2.5 rounded-[4px] flex items-center justify-between text-xs transition cursor-pointer text-left
              {activeTab === 'tracker_meta'
                ? 'border-l-2 font-semibold'
                : 'text-[#94a3b8] hover:text-[#f1f5f9] hover:bg-[#181e28]'}"
          >
            <span class="flex items-center gap-2">
              <svg class="w-3.5 h-3.5 stroke-current fill-none stroke-[1.8]" viewBox="0 0 24 24">
                <path d="M18 20V10" />
                <path d="M12 20V4" />
                <path d="M6 20v-6" />
              </svg>
              <span>{t('nav.meta')}</span>
            </span>
            <Badge variant="theme" text="Live" />
          </button>

          <!-- Player Profile (v1.5 — replaces Hero Builds per §4.4) -->
          <button
            type="button"
            onclick={() => (activeTab = 'profile')}
            style={activeTab === 'profile' ? 'border-left-color: var(--accent-primary); background-color: var(--accent-active-sidebar); color: var(--accent-light);' : undefined}
            class="w-full h-8 px-2.5 rounded-[4px] flex items-center gap-2 text-xs transition cursor-pointer text-left
              {activeTab === 'profile'
                ? 'border-l-2 font-semibold'
                : 'text-[#94a3b8] hover:text-[#f1f5f9] hover:bg-[#181e28]'}"
          >
            <svg class="w-3.5 h-3.5 stroke-current fill-none stroke-[1.8]" viewBox="0 0 24 24">
              <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2" />
              <circle cx="12" cy="7" r="4" />
            </svg>
            <span>{t('nav.profile')}</span>
          </button>
        </div>

        <!-- GROUP: TOOLKIT & AUTHORING -->
        <div class="space-y-1">
          <div class="px-2 pb-1 text-[10px] font-bold uppercase tracking-wider text-[#5d6b82]">
            Toolkit
          </div>

          <!-- Create Mod Wizard (Inspired by Deadlock Mod Builder) -->
          <button
            type="button"
            onclick={() => (activeTab = 'create_mod')}
            style={activeTab === 'create_mod' ? 'border-left-color: var(--accent-primary); background-color: var(--accent-active-sidebar); color: var(--accent-light);' : undefined}
            class="w-full h-8 px-2.5 rounded-[4px] flex items-center justify-between text-xs transition cursor-pointer text-left
              {activeTab === 'create_mod'
                ? 'border-l-2 font-semibold'
                : 'text-[#94a3b8] hover:text-[#f1f5f9] hover:bg-[#181e28]'}"
          >
            <span class="flex items-center gap-2">
              <svg class="w-3.5 h-3.5 stroke-current fill-none stroke-[1.8]" viewBox="0 0 24 24">
                <path d="M12 5v14" />
                <path d="M5 12h14" />
              </svg>
              <span>{t('nav.create_mod')}</span>
            </span>
            <Badge variant="theme" text="Wizard" />
          </button>

          <!-- VPK Fusion Engine (VPK Enhanced Merger) -->
          <button
            type="button"
            onclick={() => { activeTab = 'vpk_merge'; loadAddons(); }}
            style={activeTab === 'vpk_merge' ? 'border-left-color: var(--accent-primary); background-color: var(--accent-active-sidebar); color: var(--accent-light);' : undefined}
            class="w-full h-8 px-2.5 rounded-[4px] flex items-center gap-2 text-xs transition cursor-pointer text-left
              {activeTab === 'vpk_merge'
                ? 'border-l-2 font-semibold'
                : 'text-[#94a3b8] hover:text-[#f1f5f9] hover:bg-[#181e28]'}"
          >
            <svg class="w-3.5 h-3.5 stroke-current fill-none stroke-[1.8]" viewBox="0 0 24 24">
              <polyline points="16 3 21 3 21 8" />
              <line x1="4" y1="20" x2="21" y2="3" />
              <polyline points="21 16 21 21 16 21" />
              <line x1="15" y1="15" x2="21" y2="21" />
              <line x1="4" y1="4" x2="9" y2="9" />
            </svg>
            <span>{t('nav.vpk_merge')}</span>
          </button>

          <!-- Tool Manager -->
          <button
            type="button"
            onclick={() => (activeTab = 'dashboard')}
            style={activeTab === 'dashboard' ? 'border-left-color: var(--accent-primary); background-color: var(--accent-active-sidebar); color: var(--accent-light);' : undefined}
            class="w-full h-8 px-2.5 rounded-[4px] flex items-center justify-between text-xs transition cursor-pointer text-left
              {activeTab === 'dashboard'
                ? 'border-l-2 font-semibold'
                : 'text-[#94a3b8] hover:text-[#f1f5f9] hover:bg-[#181e28]'}"
          >
            <span class="flex items-center gap-2">
              <svg class="w-3.5 h-3.5 stroke-current fill-none stroke-[1.8]" viewBox="0 0 24 24">
                <circle cx="12" cy="12" r="3" />
                <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z" />
              </svg>
              <span>{t('nav.dashboard')}</span>
            </span>
            <Badge variant="neutral" mono={true} text="{installedCount}/{totalCount}" />
          </button>
        </div>

        <!-- GROUP: SYSTEM -->
        <div class="space-y-1">
          <div class="px-2 pb-1 text-[10px] font-bold uppercase tracking-wider text-[#5d6b82]">
            System
          </div>

          <!-- Activity Logs -->
          <button
            type="button"
            onclick={() => (activeTab = 'logs')}
            style={activeTab === 'logs' ? 'border-left-color: var(--accent-primary); background-color: var(--accent-active-sidebar); color: var(--accent-light);' : undefined}
            class="w-full h-8 px-2.5 rounded-[4px] flex items-center gap-2 text-xs transition cursor-pointer text-left
              {activeTab === 'logs'
                ? 'border-l-2 font-semibold'
                : 'text-[#94a3b8] hover:text-[#f1f5f9] hover:bg-[#181e28]'}"
          >
            <svg class="w-3.5 h-3.5 stroke-current fill-none stroke-[1.8]" viewBox="0 0 24 24">
              <polyline points="4 17 10 11 4 5" />
              <line x1="12" y1="19" x2="20" y2="19" />
            </svg>
            <span>{t('nav.logs')}</span>
          </button>

          <!-- Settings -->
          <button
            type="button"
            onclick={() => (activeTab = 'settings')}
            style={activeTab === 'settings' ? 'border-left-color: var(--accent-primary); background-color: var(--accent-active-sidebar); color: var(--accent-light);' : undefined}
            class="w-full h-8 px-2.5 rounded-[4px] flex items-center gap-2 text-xs transition cursor-pointer text-left
              {activeTab === 'settings'
                ? 'border-l-2 font-semibold'
                : 'text-[#94a3b8] hover:text-[#f1f5f9] hover:bg-[#181e28]'}"
          >
            <svg class="w-3.5 h-3.5 stroke-current fill-none stroke-[1.8]" viewBox="0 0 24 24">
              <path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z" />
              <circle cx="12" cy="12" r="3" />
            </svg>
            <span>{t('nav.settings')}</span>
          </button>
        </div>
      </nav>
    </div>

    <!-- Sidebar Footer with Version & Creator -->
    <div class="p-3 border-t border-[#1c222e] bg-[#101318] space-y-1.5">
      <div>
        {#if config?.deadlock_game_dir}
          <div style="color: var(--accent-light);" class="inline-flex items-center gap-1.5 text-[11px] font-medium">
            <span style="background-color: var(--accent-primary);" class="w-1.5 h-1.5 rounded-full animate-pulse"></span>
            <span>Deadlock linked</span>
          </div>
        {:else}
          <div class="inline-flex items-center gap-1.5 text-[11px] font-medium text-[#f59e0b]">
            <span class="w-1.5 h-1.5 rounded-full bg-[#f59e0b]"></span>
            <span>Game not linked</span>
          </div>
        {/if}
      </div>
      <div class="text-[10px] font-mono text-[#5d6b82] flex items-center justify-between pt-0.5 border-t border-[#1a1f29]">
        <span>v0.1.5</span>
        <span class="text-[#8d99ae]">Made by str1kez</span>
      </div>
    </div>
  </aside>

  <!-- ═════════════════════════════════════════════════════════════════════════ -->
  <!-- MAIN WORKSPACE                                                            -->
  <!-- ═════════════════════════════════════════════════════════════════════════ -->
  <div class="flex-1 flex flex-col min-w-0 overflow-hidden bg-[#151922]">

    <!-- Top Action Bar -->
    <header class="h-12 px-5 bg-[#12161f] border-b border-[#202734] flex items-center justify-between shrink-0">
      <!-- Breadcrumbs -->
      <nav aria-label="Breadcrumb" class="flex items-center gap-2 text-xs">
        <span style="color: var(--accent-light);" class="font-bold">Deadlock</span>
        <span class="text-[#55637a]">/</span>
        <span class="text-[#f1f5f9] font-medium capitalize">
          {#if activeTab === 'home'}{t('nav.home')}
          {:else if activeTab === 'mods'}{t('nav.mods')}
          {:else if activeTab === 'characters'}{t('nav.characters')}
          {:else if activeTab === 'installed'}{t('nav.installed')}
          {:else if activeTab === 'downloads'}{t('nav.downloads')}
          {:else if activeTab === 'tracker_meta'}{t('nav.meta')}
          {:else if activeTab === 'profile'}{t('nav.profile')}
          {:else if activeTab === 'create_mod'}{t('nav.create_mod')}
          {:else if activeTab === 'vpk_merge'}{t('nav.vpk_merge')}
          {:else if activeTab === 'dashboard'}{t('nav.dashboard')}
          {:else if activeTab === 'logs'}{t('nav.logs')}
          {:else}{t('nav.settings')}
          {/if}
        </span>
      </nav>

      <!-- Global Actions: Launch Vanilla & Launch with Mods -->
      <div class="flex items-center gap-2">
        <!-- Play Vanilla (No Mods) -->
        <Button
          variant="tonal"
          size="sm"
          onclick={launchGameVanilla}
          title="Launch Deadlock in Clean Vanilla mode (temporarily isolates custom VPK mods)"
        >
          <span class="inline-flex items-center gap-1.5">
            <svg class="w-3.5 h-3.5 stroke-current fill-none stroke-[2]" viewBox="0 0 24 24">
              <circle cx="12" cy="12" r="10" />
              <line x1="4.93" y1="4.93" x2="19.07" y2="19.07" />
            </svg>
            <span>{t('home.play_vanilla')}</span>
          </span>
        </Button>

        <!-- Play with Mods -->
        <Button
          variant="filled"
          size="sm"
          onclick={() => (isLaunchModalOpen = true)}
          title="Launch Deadlock with active mods loaded"
        >
          <span class="inline-flex items-center gap-1.5">
            <svg class="w-3.5 h-3.5 fill-current" viewBox="0 0 24 24">
              <polygon points="5 3 19 12 5 21 5 3" />
            </svg>
            <span>{t('home.play_modded')}</span>
          </span>
        </Button>

        <Button
          variant="tonal"
          size="sm"
          onclick={() => {
            if (activeTab === 'mods') syncWithGameBanana();
            else if (activeTab === 'characters' || activeTab === 'installed') loadAddons();
            else if (activeTab === 'home') { loadAddons(); loadPlayer(); loadDiskFreeSpace(); }
            else if (activeTab === 'profile') loadPlayer();
            else if (activeTab === 'logs') loadGameLogs();
            else refreshConfig();
          }}
          title="Refresh view"
        >
          <svg class="w-3.5 h-3.5 stroke-current fill-none stroke-[2]" viewBox="0 0 24 24">
            <polyline points="23 4 23 10 17 10" />
            <path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10" />
          </svg>
        </Button>
      </div>
    </header>

    <!-- Main Scrollable Content Viewport -->
    <main class="flex-1 overflow-y-auto p-5">

      <!-- ─────────────────────────────────────────────────────────────────── -->
      <!-- TAB 0: HOME DASHBOARD (v1.5)                                       -->
      <!-- ─────────────────────────────────────────────────────────────────── -->
      {#if activeTab === 'home'}
        <div class="space-y-4 max-w-5xl mx-auto">
          <!-- Welcome Banner -->
          <div class="p-5 bg-[#171c26] border border-[#252f40] rounded-[6px] flex flex-wrap items-center justify-between gap-4">
            <div class="flex items-center gap-4">
              <div class="w-12 h-12 rounded-full bg-[#1a212e] border border-[#2a3648] flex items-center justify-center text-xl font-bold text-[var(--accent-light)] shrink-0">
                {(getPlayer()?.persona_name?.[0] || 'D').toUpperCase()}
              </div>
              <div>
                <div class="text-xs text-[#828fa3]">{t('home.welcome')}</div>
                <div class="text-base font-bold text-[#f1f5f9]">{getPlayer()?.persona_name || 'Steam User'}</div>
                <div class="text-xs text-[#828fa3] font-mono mt-0.5">
                  Account ID: {getPlayer()?.account_id || '—'} · {t('home.deadlock_build')} #{deadlockBuildId}
                </div>
              </div>
            </div>
            <div class="flex flex-col items-end gap-1.5">
              {#if config?.deadlock_game_dir}
                <div style="color: var(--accent-light);" class="inline-flex items-center gap-1.5 text-xs font-medium">
                  <span style="background-color: var(--accent-primary);" class="w-2 h-2 rounded-full animate-pulse"></span>
                  <span>Deadlock Linked</span>
                </div>
                <div class="text-[11px] font-mono text-[#5d6b82] truncate max-w-[260px]">{config.deadlock_game_dir}</div>
              {:else}
                <div class="inline-flex items-center gap-1.5 text-xs font-medium text-[#f59e0b]">
                  <span class="w-2 h-2 rounded-full bg-[#f59e0b]"></span>
                  <span>Game Not Linked</span>
                </div>
              {/if}
            </div>
          </div>

          <!-- Vanilla Mode Alert Banner -->
          {#if vanillaActive}
            <div class="p-4 bg-[#2d1a06] border border-[#f59e0b]/40 rounded-[6px] flex items-center justify-between gap-4">
              <div class="flex items-center gap-3">
                <svg class="w-5 h-5 text-[#f59e0b] fill-none stroke-current stroke-[2] shrink-0" viewBox="0 0 24 24">
                  <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z" />
                  <line x1="12" y1="9" x2="12" y2="13" />
                  <line x1="12" y1="17" x2="12.01" y2="17" />
                </svg>
                <div>
                  <div class="text-sm font-semibold text-[#f59e0b]">{t('home.vanilla_active')}</div>
                  <div class="text-xs text-[#d97706]">{vanillaIsolatedCount} {t('home.vanilla_desc')}</div>
                </div>
              </div>
              <button
                type="button"
                onclick={restoreVanillaAddons}
                class="px-3 py-1.5 bg-[#f59e0b] text-[#0c0e12] text-xs font-semibold rounded-[4px] hover:bg-[#fbbf24] transition cursor-pointer shrink-0"
              >
                {t('home.restore_mods')}
              </button>
            </div>
          {/if}

          <!-- Quick Launch Bar -->
          <div class="p-4 bg-[#171c26] border border-[#252f40] rounded-[6px] space-y-3">
            <h3 class="text-xs font-bold uppercase tracking-wider text-[#5d6b82]">{t('home.quick_launch')}</h3>
            <div class="flex flex-wrap gap-2.5">
              <Button variant="filled" size="sm" onclick={() => (isLaunchModalOpen = true)}>
                <span class="inline-flex items-center gap-1.5">
                  <svg class="w-3.5 h-3.5 fill-current" viewBox="0 0 24 24"><polygon points="5 3 19 12 5 21 5 3" /></svg>
                  <span>{t('home.play_modded')}</span>
                </span>
              </Button>
              <Button variant="tonal" size="sm" onclick={launchGameVanilla} disabled={vanillaActive}>
                <span class="inline-flex items-center gap-1.5">
                  <svg class="w-3.5 h-3.5 stroke-current fill-none stroke-[2]" viewBox="0 0 24 24">
                    <circle cx="12" cy="12" r="10" />
                    <line x1="4.93" y1="4.93" x2="19.07" y2="19.07" />
                  </svg>
                  <span>{t('home.play_vanilla')}</span>
                </span>
              </Button>
              <Button variant="tonal" size="sm" onclick={openAddonsDir}>
                <span class="inline-flex items-center gap-1.5">
                  <svg class="w-3.5 h-3.5 stroke-current fill-none stroke-[1.8]" viewBox="0 0 24 24">
                    <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
                  </svg>
                  <span>{t('home.open_addons')}</span>
                </span>
              </Button>
            </div>
          </div>

          <!-- Mod Stats Grid (§4.1.3) -->
          <div class="grid grid-cols-2 md:grid-cols-5 gap-3">
            <div class="p-4 bg-[#171c26] border border-[#252f40] rounded-[6px] space-y-1">
              <div class="text-[10px] uppercase font-bold text-[#5d6b82]">{t('home.active_count')}</div>
              <div style="color: var(--accent-light);" class="text-2xl font-bold font-mono">
                {addons.filter(a => a.enabled).length}
              </div>
              <div class="text-[11px] text-[#828fa3]">VPK</div>
            </div>
            <div class="p-4 bg-[#171c26] border border-[#252f40] rounded-[6px] space-y-1">
              <div class="text-[10px] uppercase font-bold text-[#5d6b82]">{t('home.disabled_count')}</div>
              <div class="text-2xl font-bold font-mono text-[#64748b]">
                {addons.filter(a => !a.enabled).length}
              </div>
              <div class="text-[11px] text-[#828fa3]">.disabled</div>
            </div>
            <div class="p-4 bg-[#171c26] border border-[#252f40] rounded-[6px] space-y-1">
              <div class="text-[10px] uppercase font-bold text-[#5d6b82]">{t('home.gb_status')}</div>
              <div class="text-2xl font-bold font-mono {gbLatencyMs !== null ? (gbLatencyMs < 200 ? 'text-[#34d399]' : gbLatencyMs < 500 ? 'text-[#fbbf24]' : 'text-[#ef4444]') : 'text-[#64748b]'}">
                {gbLatencyMs !== null ? `${gbLatencyMs}ms` : '—'}
              </div>
              <div class="text-[11px] text-[#828fa3]">GameBanana API</div>
            </div>
            <div class="p-4 bg-[#171c26] border border-[#252f40] rounded-[6px] space-y-1">
              <div class="text-[10px] uppercase font-bold text-[#5d6b82]">{t('home.disk_space')}</div>
              <div class="text-2xl font-bold font-mono text-[#38bdf8]">{formatDiskBytes(diskFreeBytes)}</div>
              <div class="text-[11px] text-[#828fa3]">available</div>
            </div>
            <div class="p-4 bg-[#171c26] border border-[#252f40] rounded-[6px] space-y-1 col-span-2 md:col-span-1">
              <div class="text-[10px] uppercase font-bold text-[#5d6b82]">{t('home.installed_count')}</div>
              <div class="text-2xl font-bold font-mono text-[#f1f5f9]">{addons.length}</div>
              <div class="text-[11px] text-[#828fa3]">packages</div>
            </div>
          </div>

          <!-- Quick Heroes Access (§4.1.4) -->
          <div class="p-4 bg-[#171c26] border border-[#252f40] rounded-[6px] space-y-2.5">
            <div class="flex items-center justify-between">
              <h3 class="text-xs font-bold uppercase tracking-wider text-[#5d6b82]">{t('nav.characters')}</h3>
              <button type="button" onclick={() => { activeTab = 'characters'; loadAddons(); }} class="text-[11px] text-[#5d6b82] hover:text-[var(--accent-light)] transition cursor-pointer">→</button>
            </div>
            <div class="flex items-center gap-2 overflow-x-auto pb-1">
              {#each ['Seven', 'Abrams', 'Vindicta', 'Haze', 'Infernus', 'Ivy', 'Apollo', 'Rat King', 'Celeste', 'Graves'] as h}
                <button
                  type="button"
                  onclick={() => { selectedCharacter = h; activeTab = 'characters'; loadAddons(); }}
                  class="px-2.5 py-1.5 rounded-[5px] bg-[#12161f] hover:bg-[#1a212e] border border-[#252f40] hover:border-[var(--accent-primary)] flex items-center gap-2 text-xs text-[#cbd5e1] shrink-0 transition cursor-pointer"
                >
                  <HeroIcon name={h} size={20} />
                  <span>{h}</span>
                </button>
              {/each}
            </div>
          </div>

          <!-- Recent App Logs (last 5) -->
          <div class="p-4 bg-[#171c26] border border-[#252f40] rounded-[6px] space-y-2">
            <div class="flex items-center justify-between">
              <h3 class="text-xs font-bold uppercase tracking-wider text-[#5d6b82]">{t('home.recent_activity')}</h3>
              <button type="button" onclick={() => (activeTab = 'logs')} class="text-[11px] text-[#5d6b82] hover:text-[var(--accent-light)] transition cursor-pointer">{t('home.view_all')}</button>
            </div>
            <div class="space-y-1 font-mono text-[11px]">
              {#each logs.slice(0, 5) as log}
                <div class="flex items-start gap-2 leading-relaxed">
                  <span class="text-[#55637a] shrink-0">{log.time}</span>
                  <span class="shrink-0 uppercase font-bold
                    {log.level === 'error' ? 'text-[#ef4444]' : log.level === 'warn' ? 'text-[#f59e0b]' : log.level === 'success' ? 'text-[var(--accent-light)]' : 'text-[#60a5fa]'}">
                    [{log.level}]
                  </span>
                  <span class="text-[#e2e8f0] break-all">{log.message}</span>
                </div>
              {:else}
                <div class="text-[#5d6b82] italic">{t('logs.empty')}</div>
              {/each}
            </div>
          </div>
        </div>

      <!-- ─────────────────────────────────────────────────────────────────── -->
      <!-- TAB 1: COMMUNITY MODS (GAMEBANANA BROWSER WITH INFINITE SCROLL)     -->
      <!-- ─────────────────────────────────────────────────────────────────── -->
      {:else if activeTab === 'mods'}
        <div class="space-y-4 max-w-7xl mx-auto">
          <!-- Banner & Search / Filter Controls -->
          <div class="p-4 bg-[#171c26] border border-[#252f40] rounded-[6px] space-y-3">
            <div class="flex flex-wrap items-center justify-between gap-3">
              <div>
                <div class="flex items-center gap-2.5">
                  <h2 class="text-sm font-bold text-[#f1f5f9]">{t('mods.title')}</h2>
                  <button
                    type="button"
                    onclick={syncWithGameBanana}
                    disabled={isPinging}
                    class="h-6 px-2.5 rounded-[4px] bg-[#1e2533] hover:bg-[#283245] border border-[#313d52] text-[11px] font-mono text-[#94a3b8] hover:text-[#f1f5f9] flex items-center gap-1.5 transition cursor-pointer"
                    title="Click to refresh GameBanana feed and measure server latency"
                  >
                    <span style="background-color: var(--accent-light);" class="w-1.5 h-1.5 rounded-full {isPinging ? 'animate-ping' : ''}"></span>
                    <span>{t('mods.sync')}</span>
                    {#if gbLatencyMs !== null}
                      <span style="color: var(--accent-light);" class="font-bold ml-1">{gbLatencyMs} ms</span>
                    {/if}
                  </button>
                </div>
                <p class="text-xs text-[#828fa3] mt-1">
                  {t('mods.subtitle')}
                </p>
              </div>

              <!-- Search Bar & Hero Selector -->
              <div class="flex items-center gap-2 w-full sm:w-auto">
                <select
                  value={selectedHeroFilter}
                  onchange={(e) => handleHeroFilter(e.target.value)}
                  class="h-8 px-2 bg-[#101319] text-xs text-[#f1f5f9] rounded-[5px] border border-[#263143] focus:border-[var(--accent-primary)] transition cursor-pointer"
                  title="Filter mods by Deadlock character"
                >
                  <option value="All">{t('mods.all_heroes')}</option>
                  {#each HERO_NAMES as hero}
                    <option value={hero}>{hero}</option>
                  {/each}
                </select>

                <div class="w-full sm:w-64">
                  <input
                    type="text"
                    placeholder={t('mods.search')}
                    value={searchQuery}
                    oninput={handleSearchInput}
                    class="h-8 w-full px-3 py-1.5 bg-[#101319] text-xs text-[#f1f5f9] placeholder-[#55637a] rounded-[5px] border border-[#263143] focus:border-[var(--accent-primary)] transition"
                  />
                </div>
              </div>
            </div>

            <!-- Category Pills & Sort Filters -->
            <div class="flex flex-wrap items-center justify-between gap-3 pt-2 border-t border-[#212938]">
              <div class="flex items-center gap-1.5 overflow-x-auto text-xs">
                {#each ['All', 'Skins', 'Sounds', 'UI'] as cat}
                  <button
                    type="button"
                    onclick={() => handleCategoryFilter(cat)}
                    style={activeCategory === cat ? 'background-color: var(--accent-primary); color: var(--accent-on-primary);' : undefined}
                    class="h-6 px-2.5 rounded-[4px] text-[11px] font-medium transition cursor-pointer
                      {activeCategory === cat
                        ? 'font-semibold'
                        : 'bg-[#1b222f] hover:bg-[#252f40] text-[#94a3b8] hover:text-[#f1f5f9] border border-[#283446]'}"
                  >
                    {cat}
                  </button>
                {/each}
              </div>

              <div class="flex items-center gap-1.5 text-xs">
                <span class="text-[11px] text-[#828fa3] mr-1">Sort:</span>
                {#each [
                  { id: 'default', label: 'Popular' },
                  { id: 'updated', label: 'Recently Updated' },
                  { id: 'new', label: 'Newest' },
                ] as s}
                  <button
                    type="button"
                    onclick={() => handleSortChange(s.id)}
                    style={modsSort === s.id ? 'border-color: var(--accent-primary); color: var(--accent-light);' : undefined}
                    class="h-6 px-2.5 rounded-[4px] text-[11px] transition cursor-pointer
                      {modsSort === s.id
                        ? 'bg-[#1e2635] border font-semibold'
                        : 'bg-[#151922] text-[#828fa3] hover:text-[#f1f5f9] border border-[#242e3f]'}"
                  >
                    {s.label}
                  </button>
                {/each}
              </div>
            </div>
          </div>

          <!-- Mod Grid -->
          {#if modsLoading}
            <div class="py-20 flex flex-col items-center justify-center gap-3">
              <div style="border-top-color: var(--accent-primary);" class="w-8 h-8 rounded-full border-2 border-white/20 animate-spin"></div>
              <p class="text-xs text-[#828fa3]">Fetching mod catalog from GameBanana…</p>
            </div>
          {:else if modsError}
            <div class="p-6 bg-[#3b171c] border border-[#ef4444]/30 rounded-[5px] text-center space-y-3">
              <h3 class="text-sm font-semibold text-[#fca5a5]">Unable to connect to GameBanana</h3>
              <p class="text-xs font-mono text-[#fca5a5]/80 max-w-md mx-auto">{modsError}</p>
              <Button variant="tonal" size="sm" onclick={() => loadCommunityMods(1, searchQuery)}>
                Retry Connection
              </Button>
            </div>
          {:else if modsList.length === 0}
            <div class="p-12 bg-[#171c26] border border-[#252f40] rounded-[6px] text-center space-y-2">
              <h3 class="text-sm font-semibold text-[#f1f5f9]">No mods found</h3>
              <p class="text-xs text-[#828fa3]">Try searching for different keywords or reset hero/category filter.</p>
              <Button variant="tonal" size="sm" onclick={() => { searchQuery = ''; activeCategory = 'All'; selectedHeroFilter = 'All'; loadCommunityMods(1, ''); }}>
                Clear Filters
              </Button>
            </div>
          {:else}
            <div class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-3.5">
              {#each modsList as item (item.id)}
                <ModCard
                  mod={item}
                  onclick={() => openModDetails(item.id, item.item_type)}
                />
              {/each}
            </div>

            <div bind:this={sentinelEl} class="py-6 flex flex-col items-center justify-center gap-2">
              {#if modsLoadingMore}
                <div class="flex items-center gap-2.5 text-xs text-[#94a3b8] font-mono">
                  <div style="border-top-color: var(--accent-primary);" class="w-4 h-4 rounded-full border-2 border-white/20 animate-spin"></div>
                  <span>Loading more mods automatically…</span>
                </div>
              {:else if modsList.length >= modsTotal && modsTotal > 0}
                <div class="text-xs font-mono text-[#5d6b82] py-2 border-t border-[#1c222e] w-full text-center">
                  End of catalog reached ({modsTotal} mods loaded)
                </div>
              {/if}
            </div>
          {/if}
        </div>

      <!-- ─────────────────────────────────────────────────────────────────── -->
      <!-- TAB 2: CHARACTERS (HERO SELECTOR, SKINS LIST, COLLAPSIBLE INSPECTOR)-->
      <!-- ─────────────────────────────────────────────────────────────────── -->
      {:else if activeTab === 'characters'}
        <div class="h-full flex flex-col space-y-4 max-w-7xl mx-auto">
          <!-- Header Banner -->
          <div class="p-4 bg-[#171c26] border border-[#252f40] rounded-[6px] flex flex-wrap items-center justify-between gap-3 shrink-0">
            <div>
              <h2 class="text-sm font-bold text-[#f1f5f9]">Character Mod Loadouts</h2>
              <p class="text-xs text-[#828fa3]">
                Manage skin packages per hero. Select any skin or vanilla model to inspect VPK details and internal files.
              </p>
            </div>
            <div class="text-xs font-mono text-[#828fa3] flex items-center gap-2">
              <span class="w-2 h-2 rounded-full" style="background-color: var(--accent-light);"></span>
              <span>1 Active skin per hero enforced</span>
            </div>
          </div>

          <!-- Main Layout Grid: Left (Heroes), Center (Skins), Right (Collapsible Inspector) -->
          <div class="flex-1 min-h-[520px] grid grid-cols-1 lg:grid-cols-12 gap-4">

            <!-- Column 1: Deadlock Heroes Selector (3 of 12) -->
            <div class="lg:col-span-3 bg-[#131720] border border-[#232d3d] rounded-[6px] flex flex-col overflow-hidden">
              <div class="p-3 border-b border-[#212938]">
                <input
                  type="text"
                  placeholder="Filter heroes…"
                  bind:value={characterSearch}
                  class="h-8 w-full px-3 py-1 bg-[#0d1016] text-xs text-[#f1f5f9] placeholder-[#55637a] rounded border border-[#263143] focus:border-[var(--accent-primary)] transition"
                />
              </div>

              <div class="flex-1 overflow-y-auto p-2 space-y-1">
                {#each HERO_NAMES.filter(h => !characterSearch.trim() || h.toLowerCase().includes(characterSearch.toLowerCase())) as hero}
                  {@const heroMods = addons.filter(a => a.detected_hero?.toLowerCase() === hero.toLowerCase())}
                  {@const hasActive = heroMods.some(a => a.enabled)}
                  {@const icon = getHeroIcon(hero)}

                  <button
                    type="button"
                    onclick={() => {
                      selectedCharacter = hero;
                      if (selectedModForInspector) {
                        selectedModForInspector = null;
                      }
                    }}
                    style={selectedCharacter === hero ? 'border-left-color: var(--accent-primary); background-color: var(--accent-active-sidebar);' : undefined}
                    class="w-full px-2.5 py-2 rounded-[4px] flex items-center justify-between text-xs transition cursor-pointer text-left
                      {selectedCharacter === hero
                        ? 'border-l-2 font-semibold text-[#f1f5f9]'
                        : 'text-[#94a3b8] hover:text-[#f1f5f9] hover:bg-[#181f2c]'}"
                  >
                    <div class="flex items-center gap-2.5 truncate">
                      {#if icon}
                        <img src={icon} alt={hero} class="w-6 h-6 rounded-[3px] object-contain shrink-0 border border-[#2a3648]" />
                      {:else}
                        <div class="w-6 h-6 rounded-[3px] bg-[#1a212e] border border-[#2a3648] flex items-center justify-center text-[10px] font-bold shrink-0">
                          {hero[0]}
                        </div>
                      {/if}
                      <span class="truncate">{hero}</span>
                    </div>

                    <div class="flex items-center gap-1.5 shrink-0">
                      {#if hasActive}
                        <span class="w-1.5 h-1.5 rounded-full" style="background-color: var(--accent-light);" title="Custom skin active"></span>
                      {/if}
                      {#if heroMods.length > 0}
                        <span class="px-1.5 py-0.5 text-[9px] font-mono bg-[#1c2330] text-[#94a3b8] rounded border border-[#2a364a]">
                          {heroMods.length}
                        </span>
                      {/if}
                    </div>
                  </button>
                {/each}
              </div>
            </div>

            <!-- Column 2: Installed Skins for Selected Hero (Flexible Column: 9 if collapsed, 5 if inspector open) -->
            <div class="{selectedModForInspector ? 'lg:col-span-5' : 'lg:col-span-9'} bg-[#131720] border border-[#232d3d] rounded-[6px] flex flex-col overflow-hidden transition-all duration-200">
              <div class="p-3.5 border-b border-[#212938] flex items-center justify-between">
                <div class="flex items-center gap-3">
                  {#if getHeroIcon(selectedCharacter)}
                    <img src={getHeroIcon(selectedCharacter)} alt={selectedCharacter} class="w-7 h-7 rounded-[4px] object-contain border border-[#2e3b52]" />
                  {/if}
                  <div>
                    <h3 class="text-xs font-bold text-[#f1f5f9] flex items-center gap-2">
                      <span>{selectedCharacter} Skins</span>
                      <Badge variant="theme" text="{selectedHeroMods.length} Mod{selectedHeroMods.length === 1 ? '' : 's'}" />
                    </h3>
                    <p class="text-[11px] text-[#828fa3]">Click card to inspect VPK structure, preview artwork, or equip.</p>
                  </div>
                </div>

                <button
                  type="button"
                  onclick={() => {
                    activeTab = 'mods';
                    selectedHeroFilter = selectedCharacter;
                    loadCommunityMods(1, '');
                  }}
                  style="color: var(--accent-light);"
                  class="text-[11px] hover:underline font-mono cursor-pointer"
                >
                  Find {selectedCharacter} mods ↗
                </button>
              </div>

              <div class="flex-1 overflow-y-auto p-3.5 space-y-3">
                <!-- Option 1: Default Game Skin -->
                <div
                  role="button"
                  tabindex="0"
                  onclick={() => {
                    selectedModForInspector = {
                      isDefault: true,
                      filename: 'valve_source2_base.vpk',
                      mod_title: `${selectedCharacter} Default Vanilla Model`,
                      enabled: !activeHeroSkin,
                    };
                  }}
                  onkeydown={(e) => {
                    if (e.target === e.currentTarget && (e.key === 'Enter' || e.key === ' ')) {
                      e.preventDefault();
                      selectedModForInspector = {
                        isDefault: true,
                        filename: 'valve_source2_base.vpk',
                        mod_title: `${selectedCharacter} Default Vanilla Model`,
                        enabled: !activeHeroSkin,
                      };
                    }
                  }}
                  class="p-3 rounded-[5px] border transition cursor-pointer flex items-center justify-between gap-3
                    {!activeHeroSkin ? 'bg-[#182230] border-[var(--accent-primary)]' : 'bg-[#151922] border-[#252f40] hover:border-[#33425b]'}"
                >
                  <div class="flex items-center gap-3 min-w-0">
                    <div class="w-10 h-10 rounded-[4px] bg-[#1d2533] border border-[#2e3b50] flex items-center justify-center text-xs font-mono shrink-0">
                      VAN
                    </div>
                    <div>
                      <div class="text-xs font-semibold text-[#f1f5f9]">Default Vanilla Model</div>
                      <div class="text-[10px] text-[#828fa3] font-mono">Original Valve Source 2 mesh</div>
                    </div>
                  </div>

                  <div class="flex items-center gap-2">
                    {#if !activeHeroSkin}
                      <span style="background-color: var(--accent-badge-bg); color: var(--accent-badge-text); border-color: var(--accent-border);" class="px-2.5 py-1 text-[10px] font-semibold rounded border inline-flex items-center gap-1.5">
                        <span style="background-color: var(--accent-light);" class="w-1.5 h-1.5 rounded-full"></span>
                        Equipped
                      </span>
                    {:else}
                      <Button variant="tonal" size="sm" onclick={(e) => { e.stopPropagation(); activateCharacterMod(selectedCharacter, null); }}>
                        Equip
                      </Button>
                    {/if}
                    <span class="text-[10px] font-mono text-[#828fa3]">Inspect →</span>
                  </div>
                </div>

                <!-- Custom Installed Mods for this hero -->
                {#if selectedHeroMods.length === 0}
                  <div class="p-8 text-center bg-[#151922] border border-[#212938] rounded-[5px] space-y-2">
                    <p class="text-xs text-[#94a3b8]">No custom skins installed for {selectedCharacter}.</p>
                    <Button
                      variant="filled"
                      size="sm"
                      onclick={() => {
                        activeTab = 'mods';
                        selectedHeroFilter = selectedCharacter;
                        loadCommunityMods(1, '');
                      }}
                    >
                      Browse {selectedCharacter} Mods
                    </Button>
                  </div>
                {:else}
                  {#each selectedHeroMods as mod (mod.filename)}
                    <div
                      role="button"
                      tabindex="0"
                      onclick={() => (selectedModForInspector = mod)}
                      onkeydown={(e) => {
                        if (e.target === e.currentTarget && (e.key === 'Enter' || e.key === ' ')) {
                          e.preventDefault();
                          selectedModForInspector = mod;
                        }
                      }}
                      class="p-3 rounded-[5px] border transition cursor-pointer flex items-center justify-between gap-3
                        {mod.enabled ? 'bg-[#182230] border-[var(--accent-primary)]' : 'bg-[#151922] border-[#252f40] hover:border-[#33425b]'}"
                    >
                      <div class="flex items-center gap-3 min-w-0">
                        {#if mod.thumbnail_url}
                          <img
                            src={mod.thumbnail_url}
                            alt={mod.mod_title || mod.filename}
                            class="w-12 h-12 rounded-[4px] object-cover border border-[#2e3b50] shrink-0"
                            onerror={(e) => { e.currentTarget.style.display = 'none'; }}
                          />
                        {:else}
                          <div class="w-10 h-10 rounded-[4px] bg-[#1d2533] border border-[#2e3b50] flex items-center justify-center text-xs font-mono text-[#828fa3] shrink-0">
                            VPK
                          </div>
                        {/if}

                        <div class="min-w-0">
                          <div class="text-xs font-semibold text-[#f1f5f9] truncate">
                            {mod.mod_title || mod.filename.replace('.vpk.disabled', '').replace('.vpk', '')}
                          </div>
                          {#if mod.author}
                            <div class="text-[10px] text-[#828fa3]">by {mod.author}</div>
                          {/if}
                          <div class="text-[10px] text-[#828fa3] font-mono flex items-center gap-2 mt-0.5">
                            <span style="color: var(--accent-light);">{(mod.size_bytes / 1048576).toFixed(1)} MB</span>
                            <span>·</span>
                            <span>{mod.filename}</span>
                          </div>
                        </div>
                      </div>

                      <div class="flex items-center gap-2 shrink-0">
                        {#if mod.enabled}
                          <span style="background-color: var(--accent-badge-bg); color: var(--accent-badge-text); border-color: var(--accent-border);" class="px-2.5 py-1 text-[10px] font-semibold rounded border inline-flex items-center gap-1.5">
                            <span style="background-color: var(--accent-light);" class="w-1.5 h-1.5 rounded-full"></span>
                            Equipped
                          </span>
                        {:else}
                          <Button variant="tonal" size="sm" onclick={(e) => { e.stopPropagation(); activateCharacterMod(selectedCharacter, mod.filename); }}>
                            Equip
                          </Button>
                        {/if}

                        <Button variant="destructive" size="sm" onclick={(e) => { e.stopPropagation(); deleteAddon(mod.filename); }} title="Delete skin">
                          ✕
                        </Button>
                      </div>
                    </div>
                  {/each}
                {/if}
              </div>
            </div>

            <!-- Column 3: Collapsible VPK Asset Inspector (4 of 12, collapsed when nothing selected) -->
            {#if selectedModForInspector}
              <div class="lg:col-span-4 h-full">
                <VpkInspector
                  selectedMod={selectedModForInspector}
                  hero={selectedCharacter}
                  heroIcon={getHeroIcon(selectedCharacter)}
                  onClose={() => (selectedModForInspector = null)}
                  onToggleEquip={() => {
                    if (selectedModForInspector.isDefault) {
                      activateCharacterMod(selectedCharacter, null);
                    } else {
                      activateCharacterMod(selectedCharacter, selectedModForInspector.enabled ? null : selectedModForInspector.filename);
                    }
                  }}
                  onDelete={() => {
                    if (selectedModForInspector?.filename) {
                      deleteAddon(selectedModForInspector.filename);
                    }
                  }}
                />
              </div>
            {/if}
          </div>
        </div>

      <!-- ─────────────────────────────────────────────────────────────────── -->
      <!-- TAB 3: INSTALLED MODS (WITH GAMEBANANA PREVIEW CARDS)               -->
      <!-- ─────────────────────────────────────────────────────────────────── -->
      {:else if activeTab === 'installed'}
        <div class="space-y-4 max-w-6xl mx-auto">
          <!-- Installed Mods Header Controls -->
          <div class="p-4 bg-[#171c26] border border-[#252f40] rounded-[6px] space-y-3">
            <div class="flex flex-wrap items-center justify-between gap-3">
              <div>
                <h2 class="text-sm font-bold text-[#f1f5f9]">Installed Deadlock Mods</h2>
                <p class="text-xs text-[#828fa3]">
                  All packages active in <code style="color: var(--accent-light);" class="font-mono">game/citadel/addons/</code> with GameBanana sync.
                </p>
              </div>

              <!-- Search input -->
              <div class="w-full sm:w-64">
                <input
                  type="text"
                  placeholder="Filter by mod filename…"
                  bind:value={installedSearch}
                  class="h-8 w-full px-3 py-1.5 bg-[#101319] text-xs text-[#f1f5f9] placeholder-[#55637a] rounded-[5px] border border-[#263143] focus:border-[var(--accent-primary)] transition"
                />
              </div>
            </div>

            <!-- Filter and Sort Bar -->
            <div class="flex flex-wrap items-center justify-between gap-2 pt-2 border-t border-[#212938]">
              <!-- Category Filters -->
              <div class="flex items-center gap-1.5 overflow-x-auto text-xs">
                {#each [
                  { id: 'All', label: 'All' },
                  { id: 'skins', label: 'Skins & Models' },
                  { id: 'sounds', label: 'Sounds & VOs' },
                  { id: 'ui', label: 'UI & HUD' },
                  { id: 'other', label: 'Other' },
                ] as c}
                  <button
                    type="button"
                    onclick={() => installedCategoryFilter = c.id}
                    style={installedCategoryFilter === c.id ? 'background-color: var(--accent-primary); color: var(--accent-on-primary);' : undefined}
                    class="h-6 px-2.5 rounded-[4px] text-[11px] font-medium transition cursor-pointer
                      {installedCategoryFilter === c.id
                        ? 'font-semibold'
                        : 'bg-[#1b222f] hover:bg-[#252f40] text-[#94a3b8] hover:text-[#f1f5f9] border border-[#283446]'}"
                  >
                    {c.label}
                  </button>
                {/each}
              </div>

              <!-- Status and Sort dropdowns -->
              <div class="flex items-center gap-2 text-xs">
                <select
                  bind:value={installedStatus}
                  class="h-6 px-2 bg-[#1b222f] text-[11px] text-[#f1f5f9] rounded-[4px] border border-[#283446] cursor-pointer"
                >
                  <option value="all">All States</option>
                  <option value="active">Active Only</option>
                  <option value="disabled">Disabled Only</option>
                </select>

                <select
                  bind:value={installedSort}
                  class="h-6 px-2 bg-[#1b222f] text-[11px] text-[#f1f5f9] rounded-[4px] border border-[#283446] cursor-pointer"
                >
                  <option value="newest">Sort: Default</option>
                  <option value="name">Sort: Name (A-Z)</option>
                  <option value="size">Sort: Size (Largest)</option>
                </select>
              </div>
            </div>
          </div>

          <!-- Mod Items Grid with GameBanana Previews -->
          {#if filteredInstalledMods.length === 0}
            <div class="p-12 bg-[#171c26] border border-[#252f40] rounded-[6px] text-center space-y-3">
              <h3 class="text-sm font-semibold text-[#f1f5f9]">No installed mods match your criteria</h3>
              <p class="text-xs text-[#828fa3]">Download mods from the Community Mods tab to see them listed here.</p>
              <Button variant="filled" size="sm" onclick={() => (activeTab = 'mods')}>
                Browse Community Mods
              </Button>
            </div>
          {:else}
            <div class="grid grid-cols-1 md:grid-cols-2 gap-3.5">
              {#each filteredInstalledMods as addon (addon.filename)}
                <div class="p-3.5 bg-[#171c26] border border-[#252f40] rounded-[6px] flex gap-3.5 justify-between">
                  <!-- Thumbnail / Hero Preview -->
                  <div class="w-20 h-20 rounded-[5px] bg-[#11141c] border border-[#242f40] overflow-hidden shrink-0 flex items-center justify-center">
                    {#if addon.thumbnail_url}
                      <img src={addon.thumbnail_url} alt={addon.filename} class="w-full h-full object-cover" />
                    {:else if addon.detected_hero && getHeroIcon(addon.detected_hero)}
                      <img src={getHeroIcon(addon.detected_hero)} alt={addon.detected_hero} class="w-12 h-12 object-contain" />
                    {:else}
                      <span class="text-xs font-mono text-[#55637a]">VPK</span>
                    {/if}
                  </div>

                  <!-- Details -->
                  <div class="flex-1 min-w-0 flex flex-col justify-between">
                    <div>
                      <div class="flex items-center gap-2">
                        <span class="text-xs font-bold truncate text-[#f1f5f9]">
                          {addon.mod_title || addon.filename}
                        </span>
                        {#if addon.detected_hero}
                          <Badge variant="theme" text={addon.detected_hero} />
                        {:else}
                          <Badge variant="neutral" text={t('common.global_mod')} />
                        {/if}
                      </div>
                      {#if addon.author}
                        <div class="text-[10px] text-[#828fa3]">by {addon.author}</div>
                      {/if}
                      <div class="text-[10px] font-mono text-[#828fa3] mt-1">
                        <span style="color: var(--accent-light);">{(addon.size_bytes / 1048576).toFixed(1)} MB</span>
                        <span>·</span>
                        <span>{addon.filename}</span>
                      </div>
                    </div>

                    <div class="flex items-center justify-between pt-2 border-t border-[#212938]">
                      <Toggle
                        id="toggle-{addon.filename}"
                        checked={addon.enabled}
                        onchange={(enabled) => toggleAddon(addon.filename, enabled)}
                        label={addon.enabled ? 'Active' : 'Disabled'}
                      />

                      <Button
                        variant="destructive"
                        size="sm"
                        onclick={() => deleteAddon(addon.filename)}
                        title="Delete mod"
                      >
                        Delete
                      </Button>
                    </div>
                  </div>
                </div>
              {/each}
            </div>
          {/if}
        </div>

      <!-- ─────────────────────────────────────────────────────────────────── -->
      <!-- TAB 4: TRACKER · META TIER LIST (DEEP STATS)                        -->
      <!-- ─────────────────────────────────────────────────────────────────── -->
      {:else if activeTab === 'tracker_meta'}
        <div class="space-y-4 max-w-7xl mx-auto">
          <!-- Meta Banner -->
          <div class="p-4 bg-[#171c26] border border-[#252f40] rounded-[6px] space-y-3">
            <div class="flex flex-wrap items-center justify-between gap-3">
              <div>
                <div class="flex items-center gap-2">
                  <h2 class="text-sm font-bold text-[#f1f5f9]">Deadlock Live Meta & Tier Leaderboard</h2>
                  <Badge variant="theme" text="Patch #26770241" />
                </div>
                <p class="text-xs text-[#828fa3] mt-1">
                  Aggregated from high-MMR matches via Statlocker & Deadlock Client API. Updated continuously.
                </p>
              </div>

              <!-- Role Filters -->
              <div class="flex flex-wrap items-center gap-1.5 overflow-x-auto text-xs">
                {#each [
                  { id: 'All', label: t('meta.role_all') },
                  { id: 'Carries', label: t('meta.role_carries') },
                  { id: 'Brawlers', label: t('meta.role_brawlers') },
                  { id: 'Initiators', label: t('meta.role_initiators') },
                  { id: 'Mages', label: t('meta.role_mages') },
                  { id: 'Support', label: t('meta.role_support') },
                ] as role}
                  <button
                    type="button"
                    onclick={() => (metaRoleFilter = role.id)}
                    style={metaRoleFilter === role.id ? 'background-color: var(--accent-primary); color: var(--accent-on-primary);' : undefined}
                    class="h-6 px-2.5 rounded-[4px] text-[11px] font-medium transition cursor-pointer
                      {metaRoleFilter === role.id
                        ? 'font-semibold'
                        : 'bg-[#1b222f] hover:bg-[#252f40] text-[#94a3b8] hover:text-[#f1f5f9] border border-[#283446]'}"
                  >
                    {role.label}
                  </button>
                {/each}
                <!-- FIX §2.4 — inline hero search -->
                <input
                  type="text"
                  placeholder={t('meta.search')}
                  bind:value={metaSearchQuery}
                  class="h-6 px-2 bg-[#101319] text-[11px] text-[#f1f5f9] placeholder-[#55637a] rounded border border-[#263143] focus:border-[var(--accent-primary)] transition"
                />
              </div>
            </div>
          </div>

          <!-- Detailed Tier Table -->
          <div class="p-4 bg-[#171c26] border border-[#252f40] rounded-[6px] space-y-3 overflow-x-auto">
            <table class="w-full text-left text-xs">
              <thead>
                <tr class="border-b border-[#212938] text-[10px] font-mono uppercase text-[#64748b]">
                  <th class="pb-2.5">Rank</th>
                  <th class="pb-2.5">Hero</th>
                  <th class="pb-2.5">Tier</th>
                  <th class="pb-2.5">Win Rate</th>
                  <th class="pb-2.5">Pick Rate</th>
                  <th class="pb-2.5">Ban Rate</th>
                  <th class="pb-2.5">Souls / Min</th>
                  <th class="pb-2.5">DPM</th>
                  <th class="pb-2.5">Best Synergy</th>
                  <th class="pb-2.5">Hard Counter</th>
                </tr>
              </thead>
              <tbody class="divide-y divide-[#1e2535] font-mono text-[11px]">
                {#each filteredMetaHeroes as mh}
                  <tr class="hover:bg-[#1a212e] transition">
                    <td class="py-2.5 text-[#828fa3]">#{mh.rank}</td>
                    <td class="py-2.5 font-sans font-semibold text-[#f1f5f9]">
                      <div class="flex items-center gap-2">
                        <HeroIcon name={mh.name} size={24} />
                        <span>{mh.name}</span>
                      </div>
                    </td>
                    <td class="py-2.5">
                      <span class="px-1.5 py-0.5 rounded font-bold text-[10px]
                        {mh.tier.startsWith('S') ? 'bg-[#3b2b14] text-[#fcd34d] border border-[#f59e0b]/40' : mh.tier.startsWith('A') ? 'bg-[#132a1e] text-[#34d399] border border-[#10b981]/40' : 'bg-[#1b2536] text-[#38bdf8] border border-[#0284c7]/40'}">
                        {mh.tier}
                      </span>
                    </td>
                    <td class="py-2.5 text-[#34d399] font-bold">{mh.winrate}</td>
                    <td class="py-2.5 text-[#cbd5e1]">{mh.pickrate}</td>
                    <td class="py-2.5 text-[#f87171]">{mh.banrate}</td>
                    <td class="py-2.5 text-[#e2e8f0]">{mh.gpm}</td>
                    <td class="py-2.5 text-[#fbbf24]">{mh.dpm}</td>
                    <td class="py-2.5 text-[#a7f3d0] font-sans">
                      <span class="flex items-center gap-1.5">
                        <HeroIcon name={mh.synergy} size={16} />
                        {mh.synergy}
                      </span>
                    </td>
                    <td class="py-2.5 text-[#fca5a5] font-sans">
                      <span class="flex items-center gap-1.5">
                        <HeroIcon name={mh.counter} size={16} />
                        {mh.counter}
                      </span>
                    </td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        </div>

      <!-- ─────────────────────────────────────────────────────────────────── -->
      <!-- TAB: DOWNLOADS (v1.5)                                               -->
      <!-- ─────────────────────────────────────────────────────────────────── -->
      {:else if activeTab === 'downloads'}
        <div class="space-y-4 max-w-3xl mx-auto">
          <div class="p-4 bg-[#171c26] border border-[#252f40] rounded-[6px] flex items-center justify-between gap-3">
            <div>
              <h2 class="text-sm font-bold text-[#f1f5f9]">{t('downloads.title')}</h2>
              <p class="text-xs text-[#828fa3] mt-1">{t('downloads.subtitle')}</p>
            </div>
            <Button variant="tonal" size="sm" onclick={() => clearCompleted()}>
              {t('downloads.clear')}
            </Button>
          </div>

          <!-- Active Downloads -->
          {#if getActiveDownloads().length > 0}
            <div class="space-y-2">
              <div class="text-[11px] uppercase font-bold tracking-wider text-[#5d6b82] px-1">{t('downloads.active')}</div>
              {#each getActiveDownloads() as dl (`${dl.mod_id}:${dl.filename}`)}
                <DownloadItemCard item={dl} />
              {/each}
            </div>
          {/if}

          <!-- Completed Downloads -->
          {#if getCompletedDownloads().length > 0}
            <div class="space-y-2">
              <div class="text-[11px] uppercase font-bold tracking-wider text-[#5d6b82] px-1">{t('downloads.completed')}</div>
              {#each getCompletedDownloads() as dl (`${dl.mod_id}:${dl.filename}`)}
                <DownloadItemCard item={dl} />
              {/each}
            </div>
          {/if}

          {#if getActiveDownloads().length === 0 && getCompletedDownloads().length === 0}
            <div class="p-12 bg-[#171c26] border border-[#252f40] rounded-[6px] text-center space-y-2">
              <svg class="w-10 h-10 mx-auto stroke-[#5d6b82] fill-none stroke-[1.5]" viewBox="0 0 24 24">
                <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
                <polyline points="7 10 12 15 17 10" />
                <line x1="12" y1="15" x2="12" y2="3" />
              </svg>
              <p class="text-xs text-[#828fa3]">{t('downloads.empty')}</p>
            </div>
          {/if}
        </div>

      <!-- ─────────────────────────────────────────────────────────────────── -->
      <!-- TAB: PLAYER PROFILE (v1.5 — replaces Hero Builds §4.4)              -->
      <!-- ─────────────────────────────────────────────────────────────────── -->
      {:else if activeTab === 'profile'}
        <div class="space-y-4 max-w-2xl mx-auto">
          <div class="p-5 bg-[#171c26] border border-[#252f40] rounded-[6px] space-y-4">
            <h2 class="text-sm font-bold text-[#f1f5f9]">{t('profile.title')}</h2>

            {#if getPlayer()}
              {@const player = getPlayer()}
              <div class="flex items-center gap-4">
                <div class="w-16 h-16 rounded-full bg-[#1a212e] border border-[#2a3648] flex items-center justify-center text-2xl font-bold text-[var(--accent-light)] shrink-0">
                  {(player.persona_name?.[0] || 'S').toUpperCase()}
                </div>
                <div class="space-y-1">
                  <div class="text-base font-bold text-[#f1f5f9]">{player.persona_name || 'Unknown'}</div>
                  <div class="text-xs font-mono text-[#828fa3]">SteamID64: {player.steam_id_64 || '—'}</div>
                  <div class="text-xs font-mono text-[#828fa3]">Account ID: {player.account_id || '—'}</div>
                </div>
              </div>

              <div class="flex flex-wrap gap-2 pt-2 border-t border-[#212938]">
                {#if player.statlocker_url}
                  <Button variant="tonal" size="sm" onclick={() => invoke('open_browser_url', { url: player.statlocker_url })}>
                    <span class="inline-flex items-center gap-1.5">
                      <svg class="w-3.5 h-3.5 stroke-current fill-none stroke-[2]" viewBox="0 0 24 24"><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6" /><polyline points="15 3 21 3 21 9" /><line x1="10" y1="14" x2="21" y2="3" /></svg>
                      {t('profile.open_statlocker')}
                    </span>
                  </Button>
                {/if}
                {#if player.account_id}
                  <Button variant="tonal" size="sm" onclick={() => invoke('open_browser_url', { url: `https://tracklock.gg/players/${player.account_id}` })}>
                    <span class="inline-flex items-center gap-1.5">
                      <svg class="w-3.5 h-3.5 stroke-current fill-none stroke-[2]" viewBox="0 0 24 24"><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6" /><polyline points="15 3 21 3 21 9" /><line x1="10" y1="14" x2="21" y2="3" /></svg>
                      {t('profile.open_tracklock')}
                    </span>
                  </Button>
                {/if}
              </div>
            {:else}
              <div class="py-8 text-center space-y-2">
                <p class="text-xs text-[#828fa3]">{t('profile.not_detected')}</p>
                <Button variant="tonal" size="sm" onclick={() => loadPlayer()}>{t('profile.retry')}</Button>
              </div>
            {/if}
          </div>

          <!-- Manual Account ID override -->
          <div class="p-4 bg-[#171c26] border border-[#252f40] rounded-[6px] space-y-3">
            <div>
              <h3 class="text-xs font-bold uppercase tracking-wider text-[#5d6b82]">{t('profile.set_account')}</h3>
              <p class="text-[11px] text-[#828fa3] mt-1">Specify your Steam32 Friend Code / Account ID if automatic detection is unavailable.</p>
            </div>
            <div class="flex items-center gap-2">
              <input
                type="number"
                bind:value={manualAccountInput}
                placeholder="e.g. 123456789"
                class="flex-1 h-8 px-3 bg-[#101319] text-xs font-mono text-[#f1f5f9] placeholder-[#55637a] rounded border border-[#263143] focus:border-[var(--accent-primary)] transition"
              />
              <Button variant="filled" size="sm" onclick={() => {
                const n = Number(manualAccountInput);
                if (Number.isInteger(n) && n > 0) setManualAccountId(n);
              }}>
                {t('profile.apply')}
              </Button>
            </div>
          </div>

          <!-- Stats Integration Card -->
          <div class="p-4 bg-[#171c26] border border-[#252f40] rounded-[6px] space-y-2">
            <h3 class="text-xs font-bold uppercase tracking-wider text-[#5d6b82]">{t('profile.recent_matches')}</h3>
            <p class="text-xs text-[#828fa3]">
              Full Deadlock match history, MMR rank badge, winrate and top hero stats are available directly on
              {#if getPlayer()?.statlocker_url}
                <button type="button" onclick={() => invoke('open_browser_url', { url: getPlayer().statlocker_url })} class="text-[var(--accent-light)] hover:underline cursor-pointer bg-transparent border-none p-0">Statlocker</button>
              {:else}
                <span class="text-[#5d6b82]">Statlocker</span>
              {/if}
              and
              {#if getPlayer()?.account_id}
                <button type="button" onclick={() => invoke('open_browser_url', { url: `https://tracklock.gg/players/${getPlayer().account_id}` })} class="text-[var(--accent-light)] hover:underline cursor-pointer bg-transparent border-none p-0">Tracklock.gg</button>
              {:else}
                <span class="text-[#5d6b82]">Tracklock.gg</span>
              {/if}.
            </p>
          </div>
        </div>

      <!-- ─────────────────────────────────────────────────────────────────── -->
      <!-- TAB 6: CREATE MOD WIZARD (DEADLOCK MOD BUILDER STYLE)               -->
      <!-- ─────────────────────────────────────────────────────────────────── -->
      {:else if activeTab === 'create_mod'}
        <CreateModWizard
          heroes={HERO_NAMES}
          getHeroIcon={getHeroIcon}
          onModCreated={() => { loadAddons(); }}
        />

      <!-- ─────────────────────────────────────────────────────────────────── -->
      <!-- TAB 7: VPK ENHANCED FUSION ENGINE (COMPONENT MERGER)                -->
      <!-- ─────────────────────────────────────────────────────────────────── -->
      {:else if activeTab === 'vpk_merge'}
        <VpkMergeEngine
          installedAddons={addons}
          onMergeCompleted={() => { loadAddons(); }}
        />

      <!-- ─────────────────────────────────────────────────────────────────── -->
      <!-- TAB 8: TOOLKIT DASHBOARD (SOURCE 2 VIEWER & CSDK13)                 -->
      <!-- ─────────────────────────────────────────────────────────────────── -->
      {:else if activeTab === 'dashboard'}
        <div class="space-y-4 max-w-7xl mx-auto">
          <div class="p-4 bg-[#171c26] border border-[#252f40] rounded-[6px] flex flex-wrap items-center justify-between gap-3">
            <div>
              <h2 class="text-sm font-bold text-[#f1f5f9]">Source 2 Modding Toolkit</h2>
              <p class="text-xs text-[#828fa3]">
                Managed dependencies and official development tools for Valve Deadlock. Click any tool to open its download page in your browser.
              </p>
            </div>
          </div>

          <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
            {#each tools as tool (tool.id)}
              <ToolCard
                {tool}
                progress={progress[tool.id]}
                onLaunch={() => launchTool(tool.id)}
                onOpenFolder={() => openFolder(tool.id)}
                onLocate={() => openLocateModal(tool)}
                onRemove={() => confirmRemoveTool(tool)}
              />
            {/each}
          </div>
        </div>

      <!-- ─────────────────────────────────────────────────────────────────── -->
      <!-- TAB 9: ACTIVITY LOGS (v1.5 Session & Deadlock Console Logs)         -->
      <!-- ─────────────────────────────────────────────────────────────────── -->
      {:else if activeTab === 'logs'}
        <div class="space-y-3 max-w-5xl mx-auto h-full flex flex-col">
          <div class="flex flex-wrap items-center justify-between gap-2">
            <h2 class="text-sm font-bold text-[#f1f5f9]">{t('logs.title')}</h2>
            <div class="flex flex-wrap items-center gap-2">
              {#each [
                { id: 'all', label: t('logs.filter_all') },
                { id: 'info', label: t('logs.filter_info') },
                { id: 'error', label: t('logs.filter_error') },
                { id: 'game', label: t('logs.filter_game') },
              ] as f}
                <button
                  type="button"
                  onclick={() => (logsFilter = f.id)}
                  style={logsFilter === f.id ? 'background-color: var(--accent-primary); color: var(--accent-on-primary);' : undefined}
                  class="h-6 px-2.5 rounded-[4px] text-[11px] font-medium transition cursor-pointer
                    {logsFilter === f.id ? 'font-semibold' : 'bg-[#1b222f] text-[#94a3b8] hover:text-[#f1f5f9] border border-[#283446]'}"
                >{f.label}</button>
              {/each}

              <Button variant="tonal" size="sm" onclick={loadGameLogs} title="Load Deadlock console.log">
                <span class="inline-flex items-center gap-1.5">
                  <svg class="w-3.5 h-3.5 stroke-current fill-none stroke-[2]" viewBox="0 0 24 24">
                    <polyline points="4 17 10 11 4 5" /><line x1="12" y1="19" x2="20" y2="19" />
                  </svg>
                  <span>{t('logs.load_game')}</span>
                </span>
              </Button>

              <Button variant="tonal" size="sm" onclick={saveLogsToDisk}>
                <span>{logsSaved ? t('logs.saved') : t('logs.save')}</span>
              </Button>

              <Button variant="tonal" size="sm" onclick={openLogsFolder}>
                <span>{t('logs.open_folder')}</span>
              </Button>

              <Button variant="tonal" size="sm" onclick={copyLogsToClipboard}>
                <span>{logsCopied ? t('logs.copied') : t('logs.copy')}</span>
              </Button>

              <Button variant="tonal" size="sm" onclick={() => { logs = []; gameLogs = []; }}>
                {t('logs.clear')}
              </Button>
            </div>
          </div>

          <div class="flex-1 min-h-[450px] p-3.5 bg-[#0e1117] border border-[#202734] rounded-[5px] font-mono text-[11px] overflow-y-auto space-y-1">
            {#each [
              ...logs.map(l => ({ ...l, source: 'app' })),
              ...(Array.isArray(gameLogs) ? gameLogs : []),
            ].filter(log => {
              if (logsFilter === 'all') return true;
              if (logsFilter === 'info') return log.level === 'info' || log.level === 'success';
              if (logsFilter === 'error') return log.level === 'error' || log.level === 'warn';
              if (logsFilter === 'game') return log.source === 'game';
              return true;
            }).slice(0, 300) as log}
              <div class="flex items-start gap-2.5 leading-relaxed">
                <span class="text-[#55637a] shrink-0 select-none">{log.time || '--:--:--'}</span>
                {#if log.source === 'game'}
                  <span class="shrink-0 uppercase font-bold text-[#a78bfa]">[GAME]</span>
                {:else}
                  <span class="shrink-0 uppercase font-bold
                    {log.level === 'error' ? 'text-[#ef4444]' : log.level === 'warn' ? 'text-[#f59e0b]' : log.level === 'success' ? 'text-[var(--accent-light)]' : 'text-[#60a5fa]'}">
                    [{log.level}]
                  </span>
                {/if}
                <span class="text-[#e2e8f0] break-all">{log.message}</span>
              </div>
            {:else}
              <div class="text-[#5d6b82] italic py-4 text-center">{t('logs.empty')}</div>
            {/each}
          </div>
        </div>

      <!-- ─────────────────────────────────────────────────────────────────── -->
      <!-- TAB 10: SETTINGS (LANGUAGE, UI SCALE, THEMES & DIRECTORIES)         -->
      <!-- ─────────────────────────────────────────────────────────────────── -->
      {:else if activeTab === 'settings'}
        <div class="space-y-4 max-w-3xl mx-auto">
          <!-- Language & UI Scaling (v1.5 §4.8, §4.9) -->
          <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
            <!-- Interface Language -->
            <div class="p-4 bg-[#171c26] border border-[#252f40] rounded-[6px] space-y-3">
              <div>
                <h2 class="text-sm font-bold text-[#f1f5f9]">{t('settings.language')}</h2>
                <p class="text-xs text-[#828fa3] mt-0.5">{t('settings.language_desc')}</p>
              </div>
              <div class="flex items-center gap-2">
                {#each [
                  { id: 'ru', label: 'Русский (RU)' },
                  { id: 'en', label: 'English (EN)' },
                ] as lng}
                  <button
                    type="button"
                    onclick={() => changeUiLang(lng.id)}
                    style={uiLang === lng.id ? 'background-color: var(--accent-primary); color: var(--accent-on-primary);' : undefined}
                    class="flex-1 h-8 px-3 rounded-[4px] text-xs transition cursor-pointer
                      {uiLang === lng.id
                        ? 'font-semibold'
                        : 'bg-[#12151c] border border-[#263143] text-[#94a3b8] hover:text-[#f1f5f9]'}"
                  >
                    {lng.label}
                  </button>
                {/each}
              </div>
            </div>

            <!-- UI Scale -->
            <div class="p-4 bg-[#171c26] border border-[#252f40] rounded-[6px] space-y-3">
              <div>
                <h2 class="text-sm font-bold text-[#f1f5f9]">{t('settings.scale')}</h2>
                <p class="text-xs text-[#828fa3] mt-0.5">{t('settings.scale_desc')}</p>
              </div>
              <div class="grid grid-cols-3 gap-1.5">
                {#each SCALE_OPTIONS as sc}
                  <button
                    type="button"
                    onclick={() => changeUiScale(sc)}
                    style={uiScale === sc ? 'background-color: var(--accent-primary); color: var(--accent-on-primary);' : undefined}
                    class="h-8 px-2 rounded-[4px] text-xs font-mono transition cursor-pointer
                      {uiScale === sc
                        ? 'font-bold'
                        : 'bg-[#12151c] border border-[#263143] text-[#94a3b8] hover:text-[#f1f5f9]'}"
                  >
                    {sc}%
                  </button>
                {/each}
              </div>
            </div>
          </div>

          <!-- Theme Switcher Card with Circular Color Palette Swatches -->
          <div class="p-4 bg-[#171c26] border border-[#252f40] rounded-[6px] space-y-3">
            <h2 class="text-sm font-bold text-[#f1f5f9]">{t('settings.theme')}</h2>
            <p class="text-xs text-[#828fa3]">
              {t('settings.theme_desc')}
            </p>

            <div class="grid grid-cols-1 sm:grid-cols-5 gap-2.5 pt-1">
              {#each THEMES as th}
                <button
                  type="button"
                  onclick={() => applyTheme(th.id)}
                  style={currentTheme === th.id ? `border-color: ${th.hex}; box-shadow: 0 0 10px ${th.hex}44;` : undefined}
                  class="p-3 rounded-[5px] bg-[#12151c] border transition cursor-pointer text-left select-none space-y-2.5
                    {currentTheme === th.id ? 'ring-1 ring-white/20' : 'border-[#263143] hover:border-[#384863]'}"
                >
                  <div class="flex items-center justify-between">
                    <div
                      class="w-6 h-6 rounded-full border border-black/40 shadow-xs flex items-center justify-center relative overflow-hidden"
                      style="background: radial-gradient(circle at 35% 35%, {th.secondaryHex}, {th.hex});"
                    >
                      <div class="w-2 h-2 rounded-full bg-white/30"></div>
                    </div>
                    {#if currentTheme === th.id}
                      <span style="color: {th.hex};" class="text-xs font-bold">✓</span>
                    {/if}
                  </div>
                  <div class="text-[11px] font-semibold text-[#f1f5f9] leading-tight truncate">
                    {th.name}
                  </div>
                </button>
              {/each}
            </div>
          </div>

          <!-- Environment Settings Card -->
          <div class="p-4 bg-[#171c26] border border-[#252f40] rounded-[6px] space-y-4">
            <h2 class="text-sm font-bold text-[#f1f5f9]">{t('settings.env')}</h2>

            <!-- Deadlock Game Path -->
            <div class="space-y-1.5">
              <div class="text-xs font-semibold text-[#e2e8f0]">{t('settings.deadlock_path')}</div>
              <div class="text-[11px] text-[#828fa3]">
                Path to the Deadlock game directory (containing <code style="color: var(--accent-light);" class="font-mono">game/citadel</code>).
              </div>
              <TextInput
                id="cfg-deadlock-path"
                value={config?.deadlock_game_dir || ''}
                placeholder="C:\SteamLibrary\steamapps\common\Deadlock"
                actionLabel={t('settings.detect')}
                onaction={detectDeadlock}
                readonly={true}
              />
            </div>

            <!-- Camera Calibration Fix -->
            <div class="pt-3 border-t border-[#212938] flex items-center justify-between gap-3">
              <div>
                <div class="text-xs font-semibold text-[#e2e8f0]">{t('settings.camera_fix')}</div>
                {#if settingsCameraMsg}
                  <div class="text-[11px] font-mono text-[#34d399] mt-0.5">{settingsCameraMsg}</div>
                {:else}
                  <div class="text-[11px] text-[#828fa3]">Writes camera cvars to game/citadel/cfg/dmbk_camera.cfg + autoexec.cfg.</div>
                {/if}
              </div>
              <Button variant="tonal" size="sm" onclick={applyCameraFixFromSettings}>
                {t('settings.apply_camera_fix')}
              </Button>
            </div>

            <!-- Addons Directory Button -->
            <div class="pt-3 border-t border-[#212938] flex items-center justify-between">
              <div>
                <div class="text-xs font-semibold text-[#e2e8f0]">Addons Folder (<span class="font-mono text-[11px]">citadel/addons</span>)</div>
                <div class="text-[11px] text-[#828fa3]">Direct directory where game VPK mod packages reside.</div>
              </div>
              <Button variant="tonal" size="sm" onclick={openAddonsDir}>
                {t('home.open_addons')}
              </Button>
            </div>

            <!-- Kit Workspace Dir -->
            <div class="pt-3 border-t border-[#212938] flex items-center justify-between">
              <div>
                <div class="text-xs font-semibold text-[#e2e8f0]">DMBK Toolkit Workspace</div>
                <div class="text-[11px] text-[#828fa3]">Local directory where CSDK13, session logs and cache archives reside.</div>
              </div>
              <Button variant="tonal" size="sm" onclick={openKitDir}>
                Open Kit Dir
              </Button>
            </div>
          </div>

          <!-- About / Creator Card -->
          <div class="p-4 bg-[#171c26] border border-[#252f40] rounded-[6px] flex items-center justify-between text-xs text-[#828fa3]">
            <div>
              <span class="font-semibold text-[#f1f5f9]">Deadlock Mod Build Kit</span>
              <span class="font-mono ml-1.5">v0.1.5</span>
            </div>
            <div class="font-mono text-[11px]">
              Author: <span style="color: var(--accent-light);" class="font-bold">str1kez</span>
            </div>
          </div>
        </div>
      {/if}
    </main>
  </div>

  <!-- ═════════════════════════════════════════════════════════════════════════ -->
  <!-- MODALS                                                                    -->
  <!-- ═════════════════════════════════════════════════════════════════════════ -->

  <!-- 1. Community Mod Details Modal (GameBanana Preview & Installer) -->
  <ModDetailModal
    modId={selectedModId}
    itemType={selectedModItemType}
    isOpen={isModModalOpen}
    onClose={() => { isModModalOpen = false; selectedModId = null; }}
    onInstalled={() => { loadAddons(); }}
  />

  <!-- 2. Play with Mods Launch Modal (Violet Camera Fix Notice) -->
  <LaunchModal
    isOpen={isLaunchModalOpen}
    onClose={() => (isLaunchModalOpen = false)}
    onLaunch={launchGameWithMods}
  />

  <!-- 3. Locate Existing Executable Modal -->
  {#if locateModal}
    <div
      class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/70 backdrop-blur-xs"
      role="dialog"
      aria-modal="true"
    >
      <div class="max-w-md w-full p-5 bg-[#171c26] border border-[#2b374a] rounded-[6px] shadow-2xl space-y-4">
        <div>
          <h3 class="text-sm font-semibold text-[#f1f5f9]">Locate {locateModal.toolName}</h3>
          <p class="text-xs text-[#828fa3] mt-1">Paste the full path to your existing executable file on disk.</p>
        </div>

        <TextInput
          id="locate-path-input"
          placeholder="C:\Tools\...\executable.exe"
          value={locateModal.pathInput}
          errorMessage={locateModal.error}
          oninput={(e) => { locateModal.pathInput = e.target.value; locateModal.error = ''; }}
        />

        <div class="flex items-center justify-end gap-2 pt-2 border-t border-[#232c3c]">
          <Button variant="tonal" size="sm" onclick={() => { locateModal = null; }}>
            Cancel
          </Button>
          <Button variant="filled" size="sm" onclick={confirmLocate}>
            Link Executable
          </Button>
        </div>
      </div>
    </div>
  {/if}

  <!-- 4. General Confirmation Modal -->
  {#if modal}
    <ModalDialog
      isOpen={true}
      title={modal.title}
      description={modal.description}
      isDestructive={modal.isDestructive}
      onConfirm={modal.onConfirm}
      onCancel={() => { modal = null; }}
    />
  {/if}
</div>
