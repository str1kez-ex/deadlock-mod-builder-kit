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

  const DEADLOCK_HEROES = [
    'Abrams', 'Bebop', 'Calico', 'Dynamo', 'Grey Talon', 'Haze', 'Holliday', 'Infernus',
    'Ivy', 'Kelvin', 'Lady Geist', 'Lash', 'McGinnis', 'Mirage', 'Mo & Krill', 'Paradox',
    'Pocket', 'Rat King', 'Seven', 'Shiv', 'Vindicta', 'Viscous', 'Warden', 'Wraith', 'Wrecker', 'Yamato'
  ];

  const HERO_ICONS = {
    'Abrams': '/heroes/bull_sm_psd.png',
    'Bebop': '/heroes/bebop_sm_psd.png',
    'Calico': '/heroes/drifter_sm_psd_d94dcb01.png',
    'Dynamo': '/heroes/nano_sm_psd.png',
    'Grey Talon': '/heroes/archer_sm_psd.png',
    'Haze': '/heroes/haze_card_psd.png',
    'Holliday': '/heroes/astro_sm_psd.png',
    'Infernus': '/heroes/inferno_sm_psd.png',
    'Ivy': '/heroes/familiar_sm_psd.png',
    'Kelvin': '/heroes/kelvin_sm_psd.png',
    'Lady Geist': '/heroes/vampirebat_sm_psd.png',
    'Lash': '/heroes/lash_sm_psd.png',
    'McGinnis': '/heroes/engineer_sm_psd.png',
    'Mirage': '/heroes/mirage_sm_psd.png',
    'Mo & Krill': '/heroes/digger_sm_psd.png',
    'Paradox': '/heroes/chrono_sm_psd.png',
    'Pocket': '/heroes/deadpack_sm_psd.png',
    'Rat King': '/heroes/ratking_sm_psd.png',
    'Seven': '/heroes/gigawatt_sm_psd.png',
    'Shiv': '/heroes/shiv_sm_psd.png',
    'Vindicta': '/heroes/hornet_sm_png.png',
    'Viscous': '/heroes/viscous_sm_psd.png',
    'Warden': '/heroes/warden_sm_psd.png',
    'Wraith': '/heroes/wraith_sm_psd.png',
    'Wrecker': '/heroes/necro_sm_psd.png',
    'Yamato': '/heroes/yamato_sm_psd.png',
  };

  function getHeroIcon(heroName) {
    return HERO_ICONS[heroName] || '';
  }

  const THEMES = [
    { id: 'green', name: 'Stock Emerald', hex: '#2ea069', secondaryHex: '#4ade80' },
    { id: 'purple', name: 'Occult Purple', hex: '#9333ea', secondaryHex: '#c084fc' },
    { id: 'red', name: 'Crimson Strike', hex: '#dc2626', secondaryHex: '#f87171' },
    { id: 'parchment', name: 'Deadlock Sand (#EFE0BE)', hex: '#EFE0BE', secondaryHex: '#d8c49d' },
    { id: 'blue', name: 'Cobalt Astral', hex: '#0284c7', secondaryHex: '#38bdf8' },
  ];

  // ── Application State ──────────────────────────────────────────────────────
  let activeTab = $state('mods'); // 'mods' | 'characters' | 'installed' | 'tracker_meta' | 'tracker_builds' | 'create_mod' | 'vpk_merge' | 'dashboard' | 'logs' | 'settings'
  let currentTheme = $state('green');
  let deadlockBuildId = $state('26770241');
  let config = $state(null);
  let tools = $state([]);
  let addons = $state([]);
  let logs = $state([]);
  let progress = $state({}); // tool_id -> InstallProgress

  // Mod Details & Launch Modals
  let selectedModId = $state(null);
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
  let modsSort = $state('default'); // 'default' (popular), 'updated' (recently updated), 'new' (newest)
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

  // Tracker Meta & Builds State
  let metaRoleFilter = $state('All');
  let metaSearchQuery = $state('');
  let selectedBuildHero = $state('Seven');

  // Logs state
  let logsCopied = $state(false);

  // Confirmation Modals
  let modal = $state(null);
  let locateModal = $state(null);

  function appendLog(level, message) {
    const time = new Date().toLocaleTimeString();
    logs = [{ time, level, message }, ...logs.slice(0, 300)];
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

    appendLog('info', 'System initialization: Deadlock Mod Build Kit v0.1.4');
    try {
      config = await invoke('get_config');
      tools = config.tools || [];
      appendLog('success', `Environment configuration loaded: ${tools.length} managed tools.`);

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

  function openModDetails(id) {
    selectedModId = id;
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

  async function launchGameVanilla() {
    appendLog('info', 'Executing Deadlock in Vanilla mode (addons disabled)…');
    try {
      await invoke('launch_game_vanilla');
    } catch (e) {
      appendLog('error', `Vanilla launch failed: ${e}`);
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

  // ── Meta Leaderboard Data ──────────────────────────────────────────────────
  const META_HEROES_EXPANDED = [
    { rank: 1, name: 'Seven', tier: 'S+', winrate: '55.4%', pickrate: '31.2%', banrate: '22.4%', role: 'Spirit Hypercarry', gpm: '840', dpm: '2,420', counter: 'Vindicta', synergy: 'Dynamo' },
    { rank: 2, name: 'Ivy', tier: 'S+', winrate: '54.8%', pickrate: '24.1%', banrate: '18.1%', role: 'Support / Airborne Buffer', gpm: '710', dpm: '1,890', counter: 'Haze', synergy: 'Seven' },
    { rank: 3, name: 'Vindicta', tier: 'S', winrate: '53.9%', pickrate: '26.8%', banrate: '24.5%', role: 'Long Range Assassin', gpm: '820', dpm: '2,650', counter: 'Lash', synergy: 'Ivy' },
    { rank: 4, name: 'Infernus', tier: 'A+', winrate: '53.4%', pickrate: '28.0%', banrate: '14.2%', role: 'DoT Brawler / Shredder', gpm: '790', dpm: '2,380', counter: 'Kelvin', synergy: 'Abrams' },
    { rank: 5, name: 'Abrams', tier: 'A+', winrate: '52.9%', pickrate: '30.5%', banrate: '19.8%', role: 'Initiator / Melee Tank', gpm: '750', dpm: '1,980', counter: 'Seven', synergy: 'Infernus' },
    { rank: 6, name: 'Haze', tier: 'A', winrate: '52.5%', pickrate: '34.2%', banrate: '28.0%', role: 'Stealth Bullet Carry', gpm: '860', dpm: '2,510', counter: 'Warden', synergy: 'Dynamo' },
    { rank: 7, name: 'Pocket', tier: 'A', winrate: '52.1%', pickrate: '21.0%', banrate: '12.6%', role: 'Elusive Burst Mage', gpm: '780', dpm: '2,240', counter: 'Shiv', synergy: 'Ivy' },
    { rank: 8, name: 'Dynamo', tier: 'A', winrate: '51.8%', pickrate: '18.9%', banrate: '15.4%', role: 'Teamfight Initiator / Support', gpm: '680', dpm: '1,620', counter: 'Vindicta', synergy: 'Seven' },
    { rank: 9, name: 'Lady Geist', tier: 'B+', winrate: '51.2%', pickrate: '19.5%', banrate: '11.0%', role: 'Life Drain / Sustained Spirit', gpm: '770', dpm: '2,310', counter: 'Haze', synergy: 'Abrams' },
    { rank: 10, name: 'Warden', tier: 'B+', winrate: '50.7%', pickrate: '23.4%', banrate: '8.5%', role: 'Lockdown / Crowd Control', gpm: '740', dpm: '1,920', counter: 'Pocket', synergy: 'Infernus' },
    { rank: 11, name: 'Viscous', tier: 'B', winrate: '50.1%', pickrate: '17.2%', banrate: '6.2%', role: 'Goo Brawler / Frontline Utility', gpm: '710', dpm: '1,780', counter: 'Seven', synergy: 'Abrams' },
    { rank: 12, name: 'Bebop', tier: 'B', winrate: '49.4%', pickrate: '27.6%', banrate: '21.0%', role: 'Hook Specialist / Laser Burst', gpm: '760', dpm: '2,150', counter: 'Pocket', synergy: 'Seven' },
    { rank: 13, name: 'Shiv', tier: 'B', winrate: '48.9%', pickrate: '20.1%', banrate: '9.4%', role: 'Bleed Execute / Skirmisher', gpm: '780', dpm: '2,080', counter: 'Vindicta', synergy: 'Infernus' },
    { rank: 14, name: 'Rat King', tier: 'A', winrate: '52.0%', pickrate: '15.8%', banrate: '8.9%', role: 'Summoner / Area Swarm', gpm: '810', dpm: '2,190', counter: 'Ivy', synergy: 'Abrams' },
  ];

  // ── Hero Build Guides Data ─────────────────────────────────────────────────
  const HERO_BUILDS = {
    'Seven': {
      title: 'Storm Surge & Hyper Spirit Scaling',
      winrate: '58.4%',
      matches: '24,800',
      skillPriority: 'Max 3 (Power Surge) → Max 1 (Lightning Ball) → Max 4 (Storm Cloud) → Max 2 (Static Charge)',
      playstyle: 'Farm jungle aggressively during minute 3-8 using Lightning Ball. Transition to mid-game lane sieging by anchoring Storm Cloud atop high ground with Superior Reach.',
      earlyItems: ['Restorative Shot (500)', 'Mystic Reach (500)', 'Extra Spirit (500)', 'Sprint Boots (500)'],
      midItems: ['Quicksilver Reload (1,250)', 'Enduring Speed (1,250)', 'Duration Extender (1,250)', 'Bullet Lifesteal (1,250)'],
      lateItems: ['Superior Cooldown (3,000)', 'Superior Stamina (3,000)', 'Surge of Power (3,000)', 'Torment Pulse (3,000)'],
      luxuryItems: ['Boundless Spirit (6,200)', 'Curse (6,200)', 'Diviner\'s Kevlar (6,200)'],
    },
    'Abrams': {
      title: 'Heavy Melee Siphon Initiator',
      winrate: '56.8%',
      matches: '21,400',
      skillPriority: 'Max 1 (Siphon Life) → Max 2 (Shoulder Charge) → Max 4 (Seismic Impact) → Max 3 (Infernal Resilience)',
      playstyle: 'Close gaps with Shoulder Charge into point-blank Heavy Melee stun. Use Siphon Life to outsustain dual-lane trades and initiate teamfights with Seismic Impact.',
      earlyItems: ['Melee Lifesteal (500)', 'Close Quarters (500)', 'Extra Health (500)', 'Kinetic Dash (500)'],
      midItems: ['Melee Charge (1,250)', 'Restorative Locket (1,250)', 'Combat Barrier (1,250)', 'Berserker (1,250)'],
      lateItems: ['Torment Pulse (3,000)', 'Superior Stamina (3,000)', 'Warp Stone (3,000)', 'Debuff Remover (3,000)'],
      luxuryItems: ['Colossus (6,200)', 'Phantom Strike (6,200)', 'Leech (6,200)'],
    },
    'Infernus': {
      title: 'Afterburn DoT & Toxic Shredder',
      winrate: '57.1%',
      matches: '19,600',
      skillPriority: 'Max 3 (Afterburn) → Max 1 (Catalyst) → Max 2 (Flame Dash) → Max 4 (Concussive Combustion)',
      playstyle: 'Stack Afterburn triggers on enemy tanks with rapid fire rate. Use Flame Dash for escape and repositioning, activating Concussive Combustion in clustered corridors.',
      earlyItems: ['Basic Magazine (500)', 'Toxic Bullets (1,250)', 'Extra Stamina (500)', 'Mystic Burst (500)'],
      midItems: ['Kinetic Dash (1,250)', 'Enduring Speed (1,250)', 'Slowing Bullets (1,250)', 'Duration Extender (1,250)'],
      lateItems: ['Titanic Magazine (3,000)', 'Superior Stamina (3,000)', 'Pristine Emblem (3,000)', 'Spiritual Overflow (3,000)'],
      luxuryItems: ['Ricochet (6,200)', 'Silencer (6,200)', 'Siphon Bullets (6,200)'],
    },
    'Ivy': {
      title: 'Airborne Kudzu Bomber & Carry Buffer',
      winrate: '56.2%',
      matches: '16,200',
      skillPriority: 'Max 1 (Kudzu Bomb) → Max 2 (Watcher\'s Covenant) → Max 4 (Air Drop) → Max 3 (Stone Form)',
      playstyle: 'Tether to your primary hypercarry with Watcher\'s Covenant to grant immense fire rate and bullet lifesteal. Drop airborne initiators into the pit with Air Drop.',
      earlyItems: ['Extra Stamina (500)', 'Mystic Reach (500)', 'Healing Rite (500)', 'High-Velocity Mag (500)'],
      midItems: ['Quicksilver Reload (1,250)', 'Restorative Locket (1,250)', 'Enduring Speed (1,250)', 'Rescue Beam (1,250)'],
      lateItems: ['Superior Cooldown (3,000)', 'Superior Duration (3,000)', 'Heroic Aura (3,000)', 'Debuff Remover (3,000)'],
      luxuryItems: ['Curse (6,200)', 'Diviner\'s Kevlar (6,200)', 'Magic Carpet (6,200)'],
    },
    'Vindicta': {
      title: 'Crow Familiar & Longshot Sniper',
      winrate: '55.9%',
      matches: '18,900',
      skillPriority: 'Max 4 (Assassinate) → Max 2 (Flight) → Max 1 (Stake) → Max 3 (Crow Familiar)',
      playstyle: 'Take aerial position using Flight. Execute low-HP retreating enemies from across the map using Assassinate to grant bonus gold rewards to your entire squad.',
      earlyItems: ['Headshot Booster (500)', 'Extra Stamina (500)', 'Hollow Point Rounds (500)', 'Mystic Reach (500)'],
      midItems: ['Long Range (1,250)', 'Sharpshooter (3,000)', 'Warp Stone (3,000)', 'Enduring Speed (1,250)'],
      lateItems: ['Pristine Emblem (3,000)', 'Glass Cannon (6,200)', 'Superior Duration (3,000)', 'Headhunter (3,000)'],
      luxuryItems: ['Lucky Shot (6,200)', 'Crippling Headshot (6,200)', 'Vampiric Burst (6,200)'],
    },
  };

  let currentBuild = $derived(HERO_BUILDS[selectedBuildHero]);
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
              <span>Browse Mods</span>
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
              <span>Characters</span>
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
              <span>Installed Mods</span>
            </span>
            {#if addons.length > 0}
              <Badge variant="neutral" mono={true} text={String(addons.length)} />
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
              <span>Meta Tier List</span>
            </span>
            <Badge variant="theme" text="Live" />
          </button>

          <!-- Tracker Builds -->
          <button
            type="button"
            onclick={() => (activeTab = 'tracker_builds')}
            style={activeTab === 'tracker_builds' ? 'border-left-color: var(--accent-primary); background-color: var(--accent-active-sidebar); color: var(--accent-light);' : undefined}
            class="w-full h-8 px-2.5 rounded-[4px] flex items-center gap-2 text-xs transition cursor-pointer text-left
              {activeTab === 'tracker_builds'
                ? 'border-l-2 font-semibold'
                : 'text-[#94a3b8] hover:text-[#f1f5f9] hover:bg-[#181e28]'}"
          >
            <svg class="w-3.5 h-3.5 stroke-current fill-none stroke-[1.8]" viewBox="0 0 24 24">
              <path d="M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.77-3.77a6 6 0 0 1-7.94 7.94l-6.91 6.91a2.12 2.12 0 0 1-3-3l6.91-6.91a6 6 0 0 1 7.94-7.94l-3.76 3.76z" />
            </svg>
            <span>Hero Builds</span>
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
              <span>Create Mod</span>
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
            <span>VPK Fusion</span>
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
              <span>Tool Manager</span>
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
            <span>Activity Logs</span>
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
            <span>Settings</span>
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
        <span>v0.1.4</span>
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
          {#if activeTab === 'mods'}Community Mods
          {:else if activeTab === 'characters'}Characters Loadouts
          {:else if activeTab === 'installed'}Installed Mods
          {:else if activeTab === 'tracker_meta'}Tracker · Meta Tier List
          {:else if activeTab === 'tracker_builds'}Tracker · Hero Builds
          {:else if activeTab === 'create_mod'}Create Mod Wizard
          {:else if activeTab === 'vpk_merge'}VPK Fusion Engine
          {:else if activeTab === 'dashboard'}Tool Manager
          {:else if activeTab === 'logs'}Activity Logs
          {:else}Settings
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
          title="Launch Deadlock in vanilla mode without custom addons (-disableaddons)"
        >
          <span class="inline-flex items-center gap-1.5">
            <svg class="w-3.5 h-3.5 stroke-current fill-none stroke-[2]" viewBox="0 0 24 24">
              <circle cx="12" cy="12" r="10" />
              <line x1="4.93" y1="4.93" x2="19.07" y2="19.07" />
            </svg>
            <span>Play (Vanilla)</span>
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
            <span>Play with Mods</span>
          </span>
        </Button>

        <Button
          variant="tonal"
          size="sm"
          onclick={() => {
            if (activeTab === 'mods') syncWithGameBanana();
            else if (activeTab === 'characters' || activeTab === 'installed') loadAddons();
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
      <!-- TAB 1: COMMUNITY MODS (GAMEBANANA BROWSER WITH INFINITE SCROLL)     -->
      <!-- ─────────────────────────────────────────────────────────────────── -->
      {#if activeTab === 'mods'}
        <div class="space-y-4 max-w-7xl mx-auto">
          <!-- Banner & Search / Filter Controls -->
          <div class="p-4 bg-[#171c26] border border-[#252f40] rounded-[6px] space-y-3">
            <div class="flex flex-wrap items-center justify-between gap-3">
              <div>
                <div class="flex items-center gap-2.5">
                  <h2 class="text-sm font-bold text-[#f1f5f9]">Deadlock Community Mods</h2>
                  <button
                    type="button"
                    onclick={syncWithGameBanana}
                    disabled={isPinging}
                    class="h-6 px-2.5 rounded-[4px] bg-[#1e2533] hover:bg-[#283245] border border-[#313d52] text-[11px] font-mono text-[#94a3b8] hover:text-[#f1f5f9] flex items-center gap-1.5 transition cursor-pointer"
                    title="Click to refresh GameBanana feed and measure server latency"
                  >
                    <span style="background-color: var(--accent-light);" class="w-1.5 h-1.5 rounded-full {isPinging ? 'animate-ping' : ''}"></span>
                    <span>Sync with GameBanana</span>
                    {#if gbLatencyMs !== null}
                      <span style="color: var(--accent-light);" class="font-bold ml-1">{gbLatencyMs} ms</span>
                    {/if}
                  </button>
                </div>
                <p class="text-xs text-[#828fa3] mt-1">
                  Synchronized with GameBanana repository. Preview screenshots, inspect file variants, and install with a single click.
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
                  <option value="All">All Heroes</option>
                  {#each DEADLOCK_HEROES as hero}
                    <option value={hero}>{hero}</option>
                  {/each}
                </select>

                <div class="w-full sm:w-64">
                  <input
                    type="text"
                    placeholder="Search mods…"
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
                  onclick={() => openModDetails(item.id)}
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
                {#each DEADLOCK_HEROES.filter(h => !characterSearch.trim() || h.toLowerCase().includes(characterSearch.toLowerCase())) as hero}
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
                  onclick={() => {
                    selectedModForInspector = {
                      isDefault: true,
                      filename: 'valve_source2_base.vpk',
                      mod_title: `${selectedCharacter} Default Vanilla Model`,
                      enabled: !activeHeroSkin,
                    };
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
                      onclick={() => (selectedModForInspector = mod)}
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
              <div class="flex items-center gap-1.5 overflow-x-auto text-xs">
                {#each ['All', 'Carries', 'Brawlers', 'Initiators', 'Mages', 'Support'] as role}
                  <button
                    type="button"
                    onclick={() => (metaRoleFilter = role)}
                    style={metaRoleFilter === role ? 'background-color: var(--accent-primary); color: var(--accent-on-primary);' : undefined}
                    class="h-6 px-2.5 rounded-[4px] text-[11px] font-medium transition cursor-pointer
                      {metaRoleFilter === role
                        ? 'font-semibold'
                        : 'bg-[#1b222f] hover:bg-[#252f40] text-[#94a3b8] hover:text-[#f1f5f9] border border-[#283446]'}"
                  >
                    {role}
                  </button>
                {/each}
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
                {#each META_HEROES_EXPANDED as mh}
                  <tr class="hover:bg-[#1a212e] transition">
                    <td class="py-2.5 text-[#828fa3]">#{mh.rank}</td>
                    <td class="py-2.5 font-sans font-semibold text-[#f1f5f9]">
                      <div class="flex items-center gap-2">
                        {#if getHeroIcon(mh.name)}
                          <img src={getHeroIcon(mh.name)} alt={mh.name} class="w-6 h-6 rounded-[3px] object-contain border border-[#2a3648]" />
                        {/if}
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
                        {#if getHeroIcon(mh.synergy)}
                          <img src={getHeroIcon(mh.synergy)} alt={mh.synergy} class="w-4 h-4 rounded-full" />
                        {/if}
                        {mh.synergy}
                      </span>
                    </td>
                    <td class="py-2.5 text-[#fca5a5] font-sans">
                      <span class="flex items-center gap-1.5">
                        {#if getHeroIcon(mh.counter)}
                          <img src={getHeroIcon(mh.counter)} alt={mh.counter} class="w-4 h-4 rounded-full" />
                        {/if}
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
      <!-- TAB 5: TRACKER · HERO BUILDS & PROGRESSION                          -->
      <!-- ─────────────────────────────────────────────────────────────────── -->
      {:else if activeTab === 'tracker_builds'}
        <div class="space-y-4 max-w-6xl mx-auto">
          <!-- Builds Hero Selector Bar -->
          <div class="p-4 bg-[#171c26] border border-[#252f40] rounded-[6px] space-y-3">
            <h2 class="text-sm font-bold text-[#f1f5f9]">Competitive Deadlock Hero Builds</h2>
            <div class="flex items-center gap-2 overflow-x-auto pb-1">
              {#each Object.keys(HERO_BUILDS) as hero}
                <button
                  type="button"
                  onclick={() => (selectedBuildHero = hero)}
                  style={selectedBuildHero === hero ? 'border-color: var(--accent-primary); background-color: var(--accent-active-sidebar);' : undefined}
                  class="h-8 px-3 rounded-[4px] border flex items-center gap-2 text-xs font-semibold transition cursor-pointer shrink-0
                    {selectedBuildHero === hero
                      ? 'text-[#f1f5f9]'
                      : 'bg-[#151a24] border-[#252f40] text-[#94a3b8] hover:text-[#f1f5f9]'}"
                >
                  {#if getHeroIcon(hero)}
                    <img src={getHeroIcon(hero)} alt={hero} class="w-4 h-4 rounded-full" />
                  {/if}
                  <span>{hero}</span>
                </button>
              {/each}
            </div>
          </div>

          <!-- Selected Build Breakdown -->
          {#if currentBuild}
            <div class="space-y-4">
              <!-- Build Title & Summary -->
              <div class="p-5 bg-[#171c26] border border-[#252f40] rounded-[6px] space-y-2.5">
                <div class="flex flex-wrap items-center justify-between gap-3">
                  <div class="flex items-center gap-3">
                    {#if getHeroIcon(selectedBuildHero)}
                      <img src={getHeroIcon(selectedBuildHero)} alt={selectedBuildHero} class="w-10 h-10 rounded-[5px] object-contain border border-[#2b374a]" />
                    {/if}
                    <div>
                      <h3 class="text-sm font-bold text-[#f1f5f9]">{selectedBuildHero} — {currentBuild.title}</h3>
                      <div class="text-[11px] font-mono text-[#828fa3] mt-0.5">
                        Win Rate: <strong class="text-[#34d399]">{currentBuild.winrate}</strong> across {currentBuild.matches} tracked matches
                      </div>
                    </div>
                  </div>
                </div>

                <div class="p-3 bg-[#11141c] border border-[#212938] rounded-[5px] space-y-1">
                  <div class="text-[10px] uppercase font-bold text-[#5d6b82]">Skill Leveling Priority</div>
                  <div class="text-xs font-mono text-[#cbd5e1]">{currentBuild.skillPriority}</div>
                </div>

                <p class="text-xs text-[#94a3b8] leading-relaxed pt-1">
                  {currentBuild.playstyle}
                </p>
              </div>

              <!-- Item Progression Stages -->
              <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-3.5">
                <!-- Early -->
                <div class="p-4 bg-[#171c26] border border-[#252f40] rounded-[6px] space-y-2">
                  <div class="text-xs font-bold text-[#38bdf8] border-b border-[#212938] pb-1.5">
                    1. Early Game (500 Souls)
                  </div>
                  <div class="space-y-1.5 text-xs font-mono">
                    {#each currentBuild.earlyItems as it}
                      <div class="p-1.5 bg-[#12151c] rounded border border-[#212938] text-[#cbd5e1]">
                        {it}
                      </div>
                    {/each}
                  </div>
                </div>

                <!-- Mid -->
                <div class="p-4 bg-[#171c26] border border-[#252f40] rounded-[6px] space-y-2">
                  <div class="text-xs font-bold text-[#34d399] border-b border-[#212938] pb-1.5">
                    2. Mid Game Core (1,250 Souls)
                  </div>
                  <div class="space-y-1.5 text-xs font-mono">
                    {#each currentBuild.midItems as it}
                      <div class="p-1.5 bg-[#12151c] rounded border border-[#212938] text-[#cbd5e1]">
                        {it}
                      </div>
                    {/each}
                  </div>
                </div>

                <!-- Late -->
                <div class="p-4 bg-[#171c26] border border-[#252f40] rounded-[6px] space-y-2">
                  <div class="text-xs font-bold text-[#fbbf24] border-b border-[#212938] pb-1.5">
                    3. Late Spikes (3,000 Souls)
                  </div>
                  <div class="space-y-1.5 text-xs font-mono">
                    {#each currentBuild.lateItems as it}
                      <div class="p-1.5 bg-[#12151c] rounded border border-[#212938] text-[#cbd5e1]">
                        {it}
                      </div>
                    {/each}
                  </div>
                </div>

                <!-- Luxury -->
                <div class="p-4 bg-[#171c26] border border-[#252f40] rounded-[6px] space-y-2">
                  <div class="text-xs font-bold text-[#c084fc] border-b border-[#212938] pb-1.5">
                    4. End Game / Luxury (6,200)
                  </div>
                  <div class="space-y-1.5 text-xs font-mono">
                    {#each currentBuild.luxuryItems as it}
                      <div class="p-1.5 bg-[#12151c] rounded border border-[#212938] text-[#cbd5e1]">
                        {it}
                      </div>
                    {/each}
                  </div>
                </div>
              </div>
            </div>
          {/if}
        </div>

      <!-- ─────────────────────────────────────────────────────────────────── -->
      <!-- TAB 6: CREATE MOD WIZARD (DEADLOCK MOD BUILDER STYLE)               -->
      <!-- ─────────────────────────────────────────────────────────────────── -->
      {:else if activeTab === 'create_mod'}
        <CreateModWizard
          heroes={DEADLOCK_HEROES}
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
      <!-- TAB 9: ACTIVITY LOGS                                                -->
      <!-- ─────────────────────────────────────────────────────────────────── -->
      {:else if activeTab === 'logs'}
        <div class="space-y-3 max-w-5xl mx-auto h-full flex flex-col">
          <div class="flex items-center justify-between">
            <h2 class="text-sm font-bold text-[#f1f5f9]">System & Process Logs</h2>
            <div class="flex items-center gap-2">
              <Button variant="tonal" size="sm" onclick={copyLogsToClipboard}>
                <span class="inline-flex items-center gap-1.5">
                  <svg class="w-3.5 h-3.5 stroke-current fill-none stroke-[2]" viewBox="0 0 24 24">
                    <rect x="9" y="9" width="13" height="13" rx="2" ry="2" />
                    <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
                  </svg>
                  <span>{logsCopied ? 'Copied!' : 'Copy Logs'}</span>
                </span>
              </Button>

              <Button variant="tonal" size="sm" onclick={() => (logs = [])}>
                Clear Logs
              </Button>
            </div>
          </div>

          <div class="flex-1 min-h-[450px] p-3.5 bg-[#0e1117] border border-[#202734] rounded-[5px] font-mono text-[11px] overflow-y-auto space-y-1">
            {#each logs as log}
              <div class="flex items-start gap-2.5 leading-relaxed">
                <span class="text-[#55637a] shrink-0 select-none">{log.time}</span>
                <span class="shrink-0 uppercase font-bold
                  {log.level === 'error' ? 'text-[#ef4444]' : log.level === 'warn' ? 'text-[#f59e0b]' : log.level === 'success' ? 'text-[var(--accent-light)]' : 'text-[#60a5fa]'}">
                  [{log.level}]
                </span>
                <span class="text-[#e2e8f0] break-all">{log.message}</span>
              </div>
            {/each}
          </div>
        </div>

      <!-- ─────────────────────────────────────────────────────────────────── -->
      <!-- TAB 10: SETTINGS (THEMES & DIRECTORIES)                             -->
      <!-- ─────────────────────────────────────────────────────────────────── -->
      {:else if activeTab === 'settings'}
        <div class="space-y-4 max-w-3xl mx-auto">
          <!-- Theme Switcher Card with Circular Color Palette Swatches -->
          <div class="p-4 bg-[#171c26] border border-[#252f40] rounded-[6px] space-y-3">
            <h2 class="text-sm font-bold text-[#f1f5f9]">Color Theme</h2>
            <p class="text-xs text-[#828fa3]">
              Select an accent color palette for the user interface.
            </p>

            <div class="grid grid-cols-1 sm:grid-cols-5 gap-2.5 pt-1">
              {#each THEMES as t}
                <button
                  type="button"
                  onclick={() => applyTheme(t.id)}
                  style={currentTheme === t.id ? `border-color: ${t.hex}; box-shadow: 0 0 10px ${t.hex}44;` : undefined}
                  class="p-3 rounded-[5px] bg-[#12151c] border transition cursor-pointer text-left select-none space-y-2.5
                    {currentTheme === t.id ? 'ring-1 ring-white/20' : 'border-[#263143] hover:border-[#384863]'}"
                >
                  <div class="flex items-center justify-between">
                    <div
                      class="w-6 h-6 rounded-full border border-black/40 shadow-xs flex items-center justify-center relative overflow-hidden"
                      style="background: radial-gradient(circle at 35% 35%, {t.secondaryHex}, {t.hex});"
                    >
                      <div class="w-2 h-2 rounded-full bg-white/30"></div>
                    </div>
                    {#if currentTheme === t.id}
                      <span style="color: {t.hex};" class="text-xs font-bold">✓</span>
                    {/if}
                  </div>
                  <div class="text-[11px] font-semibold text-[#f1f5f9] leading-tight truncate">
                    {t.name}
                  </div>
                </button>
              {/each}
            </div>
          </div>

          <!-- Environment Settings Card -->
          <div class="p-4 bg-[#171c26] border border-[#252f40] rounded-[6px] space-y-4">
            <h2 class="text-sm font-bold text-[#f1f5f9]">Environment & Directories</h2>

            <!-- Deadlock Game Path -->
            <div class="space-y-1.5">
              <div class="text-xs font-semibold text-[#e2e8f0]">Deadlock Installation Directory</div>
              <div class="text-[11px] text-[#828fa3]">
                Path to the Deadlock game directory (containing <code style="color: var(--accent-light);" class="font-mono">game/citadel</code>).
              </div>
              <TextInput
                id="cfg-deadlock-path"
                value={config?.deadlock_game_dir || ''}
                placeholder="C:\SteamLibrary\steamapps\common\Deadlock"
                actionLabel="Auto-Detect"
                onaction={detectDeadlock}
                readonly={true}
              />
            </div>

            <!-- Addons Directory Button -->
            <div class="pt-3 border-t border-[#212938] flex items-center justify-between">
              <div>
                <div class="text-xs font-semibold text-[#e2e8f0]">Addons Folder (<span class="font-mono text-[11px]">citadel/addons</span>)</div>
                <div class="text-[11px] text-[#828fa3]">Direct directory where game VPK mod packages reside.</div>
              </div>
              <Button variant="tonal" size="sm" onclick={openAddonsDir}>
                Open Addons Dir
              </Button>
            </div>

            <!-- Kit Workspace Dir -->
            <div class="pt-3 border-t border-[#212938] flex items-center justify-between">
              <div>
                <div class="text-xs font-semibold text-[#e2e8f0]">DMBK Toolkit Workspace</div>
                <div class="text-[11px] text-[#828fa3]">Local directory where CSDK13 and cache archives are isolated.</div>
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
              <span class="font-mono ml-1.5">v0.1.4</span>
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
