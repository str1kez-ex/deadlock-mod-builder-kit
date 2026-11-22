<script>
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import { onMount } from 'svelte';
  import Button from './Button.svelte';
  import Badge from './Badge.svelte';
  import ProgressBar from './ProgressBar.svelte';

  let {
    modId = null,
    itemType = 'Mod',
    isOpen = false,
    onClose = undefined,
    onInstalled = undefined,
  } = $props();

  let loading = $state(true);
  let error = $state('');
  let mod = $state(null);
  let activeScreenshot = $state(0);
  let selectedFileIndex = $state(0);

  let installState = $state('idle'); // 'idle' | 'downloading' | 'extracting' | 'done' | 'error'
  let installProgress = $state(0);
  let installMsg = $state('');
  let installSpeed = $state(0);
  let installError = $state('');

  let unlistenProgress = null;
  let detectedHero = $state('Abrams');

  const DEADLOCK_HEROES = [
    'Abrams', 'Bebop', 'Calico', 'Dynamo', 'Grey Talon', 'Haze', 'Holliday', 'Infernus',
    'Ivy', 'Kelvin', 'Lady Geist', 'Lash', 'McGinnis', 'Mirage', 'Mo & Krill', 'Paradox',
    'Pocket', 'Rat King', 'Seven', 'Shiv', 'Vindicta', 'Viscous', 'Warden', 'Wraith', 'Wrecker', 'Yamato'
  ];

  function autoDetectHero(modObj) {
    if (!modObj) return 'Abrams';
    const text = `${modObj.name} ${modObj.category || ''}`.toLowerCase();
    for (const h of DEADLOCK_HEROES) {
      if (text.includes(h.toLowerCase())) return h;
    }
    if (text.includes('geist')) return 'Lady Geist';
    if (text.includes('talon')) return 'Grey Talon';
    if (text.includes('mokrill') || text.includes('mo & krill') || text.includes('mo and krill')) return 'Mo & Krill';
    if (text.includes('ratking') || text.includes('rat king')) return 'Rat King';
    return 'Abrams';
  }

  $effect(() => {
    if (isOpen && modId) {
      loadModDetails(modId, itemType);
    } else {
      resetState();
    }
  });

  function resetState() {
    loading = true;
    error = '';
    mod = null;
    activeScreenshot = 0;
    selectedFileIndex = 0;
    installState = 'idle';
    installProgress = 0;
    installMsg = '';
    installSpeed = 0;
    installError = '';
  }

  onMount(async () => {
    unlistenProgress = await listen('mod_download_progress', (e) => {
      const p = e.payload;
      if (p.mod_id === modId) {
        installState = p.state;
        installProgress = p.progress * 100;
        installMsg = p.message;
        installSpeed = p.speed_mbps;
        if (p.state === 'error') {
          installError = p.message;
        } else if (p.state === 'done') {
          onInstalled?.();
        }
      }
    });

    return () => {
      if (unlistenProgress) unlistenProgress();
    };
  });

  async function loadModDetails(id, type) {
    loading = true;
    error = '';
    try {
      mod = await invoke('fetch_gamebanana_mod_detail', { modId: id, itemType: type });
      selectedFileIndex = 0;
      activeScreenshot = 0;
      detectedHero = autoDetectHero(mod);
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  async function startInstall() {
    if (!mod || !mod.files || !mod.files[selectedFileIndex]) return;
    const file = mod.files[selectedFileIndex];

    installState = 'downloading';
    installProgress = 2;
    installMsg = 'Connecting to download server…';
    installError = '';

    try {
      await invoke('install_gamebanana_mod', {
        modId: mod.id,
        downloadUrl: file.download_url,
        filename: file.filename,
        heroName: detectedHero || null,
        modTitle: mod.name || null,
        thumbnailUrl: mod.screenshots?.[0] || null,
        author: mod.submitter_name || null,
        description: mod.description || null,
      });
      installState = 'done';
      installProgress = 100;
      installMsg = `Installed ${file.filename} into citadel/addons!`;
      onInstalled?.();
    } catch (err) {
      installState = 'error';
      installError = String(err);
      installMsg = `Installation failed: ${err}`;
    }
  }

  function handleKeydown(e) {
    if (!isOpen) return;
    if (e.key === 'Escape') {
      e.preventDefault();
      onClose?.();
    } else if (mod?.screenshots?.length > 1) {
      if (e.key === 'ArrowLeft') {
        activeScreenshot = (activeScreenshot - 1 + mod.screenshots.length) % mod.screenshots.length;
      } else if (e.key === 'ArrowRight') {
        activeScreenshot = (activeScreenshot + 1) % mod.screenshots.length;
      }
    }
  }

  async function launchGame() {
    try {
      await invoke('launch_game');
    } catch (e) {
      console.error(e);
    }
  }

  async function openAddonsFolder() {
    try {
      await invoke('open_game_addons_folder');
    } catch (e) {
      console.error(e);
    }
  }

  function formatDate(ts) {
    if (!ts || ts === 0) return 'N/A';
    try {
      const d = new Date(Number(ts) * 1000);
      return d.toLocaleDateString('en-US', {
        year: 'numeric',
        month: 'short',
        day: 'numeric',
      });
    } catch {
      return 'N/A';
    }
  }

  async function openExternalUrl(url) {
    if (!url) return;
    try {
      await invoke('open_browser_url', { url });
    } catch {
      window.open(url, '_blank');
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if isOpen}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/75 backdrop-blur-xs"
    role="presentation"
    onclick={(e) => {
      if (e.target === e.currentTarget) onClose?.();
    }}
  >
    <div
      class="max-w-4xl w-full max-h-[90vh] bg-[#141822] border border-[#2b374a] rounded-[6px] shadow-2xl flex flex-col overflow-hidden text-[#f1f5f9]"
      role="dialog"
      aria-modal="true"
      tabindex="-1"
      aria-label="Mod Details"
    >
      <!-- Modal Header -->
      <header class="px-5 py-3.5 bg-[#171c27] border-b border-[#252f42] flex items-center justify-between shrink-0">
        <div class="flex items-center gap-3 min-w-0 pr-4">
          <Badge variant="sdk" text={mod?.category || itemType} />
          <h2 class="text-sm font-semibold truncate text-[#f1f5f9]">
            {loading ? 'Loading Mod Information…' : mod?.name || 'Mod Details'}
          </h2>
        </div>

        <button
          type="button"
          onclick={onClose}
          class="w-7 h-7 rounded-[4px] bg-[#202735] hover:bg-[#2c3649] border border-[#313c50] text-[#94a3b8] hover:text-[#f1f5f9] flex items-center justify-center text-xs transition cursor-pointer shrink-0"
          title="Close dialog (Esc)"
          aria-label="Close modal"
        >
          ✕
        </button>
      </header>

      <!-- Modal Body -->
      <div class="flex-1 overflow-y-auto p-5 space-y-5">
        {#if loading}
          <div class="py-16 flex flex-col items-center justify-center gap-3 text-center">
            <div style="border-top-color: var(--accent-primary);" class="w-8 h-8 rounded-full border-2 border-white/20 animate-spin"></div>
            <p class="text-xs text-[#94a3b8]">Fetching mod metadata and download mirrors from GameBanana…</p>
          </div>
        {:else if error}
          <div class="p-6 bg-[#3b171c] border border-[#ef4444]/30 rounded-[5px] text-center space-y-3">
            <h3 class="text-sm font-semibold text-[#fca5a5]">Failed to load mod details</h3>
            <p class="text-xs font-mono text-[#fca5a5]/80 max-w-md mx-auto">{error}</p>
            <Button variant="tonal" size="sm" onclick={() => loadModDetails(modId, itemType)}>
              Retry Request
            </Button>
          </div>
        {:else if mod}
          <!-- Submitter Info & Stats Banner -->
          <div class="space-y-2 p-3 bg-[#181e2a] border border-[#263143] rounded-[5px]">
            <div class="flex flex-wrap items-center justify-between gap-3">
              <div class="flex items-center gap-2.5">
                {#if mod.submitter_avatar}
                  <img
                    src={mod.submitter_avatar}
                    alt={mod.submitter_name}
                    class="w-7 h-7 rounded-full border border-[#384863] object-cover shrink-0"
                    onerror={(e) => { e.currentTarget.style.display = 'none'; }}
                  />
                {:else}
                  <div class="w-7 h-7 rounded-full bg-[#263143] text-xs flex items-center justify-center font-bold">
                    {mod.submitter_name[0] || 'U'}
                  </div>
                {/if}
                <div class="text-xs">
                  <span class="text-[#828fa3]">Created by</span>
                  <span class="text-[#f1f5f9] font-medium ml-1">{mod.submitter_name}</span>
                </div>
              </div>

              <div class="flex items-center gap-4 text-xs font-mono text-[#828fa3]">
                <span class="flex items-center gap-1">
                  <svg class="w-3.5 h-3.5 stroke-current fill-none stroke-[1.8] text-[#828fa3]" viewBox="0 0 24 24">
                    <path d="M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z" />
                  </svg>
                  {mod.likes} likes
                </span>
                <span class="flex items-center gap-1">
                  <svg class="w-3.5 h-3.5 stroke-current fill-none stroke-[1.8] text-[#828fa3]" viewBox="0 0 24 24">
                    <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z" />
                    <circle cx="12" cy="12" r="3" />
                  </svg>
                  {mod.views.toLocaleString()} views
                </span>
                <button
                  type="button"
                  onclick={() => openExternalUrl(mod.profile_url)}
                  style="color: var(--accent-light);"
                  class="hover:underline flex items-center gap-1 font-sans cursor-pointer bg-transparent border-none p-0"
                >
                  <span>GameBanana</span> ↗
                </button>
              </div>
            </div>

            <!-- Published & Updated Dates Bar -->
            <div class="pt-2 border-t border-[#232c3c] flex flex-wrap items-center justify-between gap-3 text-[11px] font-mono text-[#94a3b8]">
              <div class="flex items-center gap-4">
                <span class="flex items-center gap-1.5" title="Original date published on GameBanana">
                  <svg class="w-3.5 h-3.5 stroke-current fill-none stroke-[1.8] text-[#64748b]" viewBox="0 0 24 24">
                    <rect x="3" y="4" width="18" height="18" rx="2" ry="2" />
                    <line x1="16" y1="2" x2="16" y2="6" />
                    <line x1="8" y1="2" x2="8" y2="6" />
                    <line x1="3" y1="10" x2="21" y2="10" />
                  </svg>
                  <span>Published: <span class="text-[#f1f5f9] font-medium">{formatDate(mod.date_added)}</span></span>
                </span>

                {#if mod.date_updated && mod.date_updated > 0}
                  <span class="flex items-center gap-1.5" title="Last revision or update timestamp">
                    <svg class="w-3.5 h-3.5 stroke-current fill-none stroke-[1.8] text-[#64748b]" viewBox="0 0 24 24">
                      <polyline points="23 4 23 10 17 10" />
                      <path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10" />
                    </svg>
                    <span>Updated: <span class="text-[#f1f5f9] font-medium">{formatDate(mod.date_updated)}</span></span>
                  </span>
                {/if}
              </div>

              {#if mod.files && mod.files.length > 0}
                <div class="text-[#828fa3]">
                  {mod.files.length} {mod.files.length === 1 ? 'file available' : 'files available'}
                </div>
              {/if}
            </div>
          </div>

          <!-- Screenshots Gallery / Carousel -->
          {#if mod.screenshots && mod.screenshots.length > 0}
            <div class="space-y-2">
              <div class="relative w-full h-[320px] bg-[#0c0e13] border border-[#232c3c] rounded-[5px] overflow-hidden flex items-center justify-center">
                <img
                  src={mod.screenshots[activeScreenshot]}
                  alt="{mod.name} preview {activeScreenshot + 1}"
                  class="max-w-full max-h-full object-contain select-none"
                />

                {#if mod.screenshots.length > 1}
                  <button
                    type="button"
                    onclick={() => activeScreenshot = (activeScreenshot - 1 + mod.screenshots.length) % mod.screenshots.length}
                    class="absolute left-3 top-1/2 -translate-y-1/2 w-8 h-8 rounded-full bg-black/60 hover:bg-black/85 text-white flex items-center justify-center text-sm border border-white/20 transition cursor-pointer"
                    title="Previous screenshot (←)"
                  >
                    ❮
                  </button>
                  <button
                    type="button"
                    onclick={() => activeScreenshot = (activeScreenshot + 1) % mod.screenshots.length}
                    class="absolute right-3 top-1/2 -translate-y-1/2 w-8 h-8 rounded-full bg-black/60 hover:bg-black/85 text-white flex items-center justify-center text-sm border border-white/20 transition cursor-pointer"
                    title="Next screenshot (→)"
                  >
                    ❯
                  </button>
                  <div class="absolute bottom-2 right-3 px-2 py-0.5 rounded bg-black/70 text-[10px] font-mono text-white/80 border border-white/10">
                    {activeScreenshot + 1} / {mod.screenshots.length}
                  </div>
                {/if}
              </div>

              <!-- Thumbnails strip -->
              {#if mod.screenshots.length > 1}
                <div class="flex items-center gap-2 overflow-x-auto pb-1">
                  {#each mod.screenshots as shot, idx}
                    <button
                      type="button"
                      onclick={() => activeScreenshot = idx}
                      style={activeScreenshot === idx ? 'border-color: var(--accent-primary);' : undefined}
                      class="w-16 h-12 rounded-[4px] border overflow-hidden shrink-0 transition cursor-pointer
                        {activeScreenshot === idx ? 'ring-1 ring-[var(--accent-primary)]' : 'border-[#263143] opacity-60 hover:opacity-100'}"
                    >
                      <img src={shot} alt="thumb {idx}" class="w-full h-full object-cover" />
                    </button>
                  {/each}
                </div>
              {/if}
            </div>
          {/if}

          <!-- Mod Description -->
          {#if mod.description || mod.text_html}
            <div class="space-y-1.5 p-3.5 bg-[#171c26] border border-[#263143] rounded-[5px]">
              <div class="text-[11px] font-semibold text-[#828fa3] uppercase tracking-wider">
                About this mod
              </div>
              {#if mod.description}
                <p class="text-xs text-[#e2e8f0] font-medium leading-relaxed">
                  {mod.description}
                </p>
              {/if}
              {#if mod.text_html}
                <div class="text-xs text-[#94a3b8] leading-relaxed max-h-40 overflow-y-auto pr-2 space-y-1">
                  {@html mod.text_html.replace(/<script\b[^<]*(?:(?!<\/script>)<[^<]*)*<\/script>/gi, '')}
                </div>
              {/if}
            </div>
          {/if}

          <!-- File Selection & Installation Box -->
          <div class="p-4 bg-[#181d29] border border-[#2b374a] rounded-[5px] space-y-3.5">
            <div class="flex items-center justify-between">
              <div>
                <div class="text-xs font-semibold text-[#f1f5f9]">Available Files</div>
                <div class="text-[11px] text-[#828fa3]">
                  Choose which file to install directly into Deadlock's <code style="color: var(--accent-light);" class="font-mono">game/citadel/addons/</code>
                </div>
              </div>
              <Badge variant="packer" text="{mod.files.length} file{mod.files.length === 1 ? '' : 's'}" />
            </div>

            <!-- Files list -->
            {#if mod.files && mod.files.length > 0}
              <div class="space-y-2">
                {#each mod.files as file, fIdx}
                  <label
                    style={selectedFileIndex === fIdx ? 'border-color: var(--accent-primary);' : undefined}
                    class="flex items-center justify-between p-3 rounded-[5px] border transition cursor-pointer select-none
                      {selectedFileIndex === fIdx
                        ? 'bg-[#1b2536] text-[#f1f5f9]'
                        : 'bg-[#131720] border-[#252f40] hover:border-[#384863] text-[#94a3b8]'}"
                  >
                    <div class="flex items-center gap-3 min-w-0 pr-3">
                      <input
                        type="radio"
                        name="mod_file_choice"
                        checked={selectedFileIndex === fIdx}
                        onchange={() => selectedFileIndex = fIdx}
                        style="accent-color: var(--accent-primary);"
                        class="cursor-pointer"
                      />
                      <div class="min-w-0">
                        <div class="text-xs font-semibold truncate text-[#e2e8f0]">
                          {file.filename}
                        </div>
                        {#if file.description}
                          <div class="text-[11px] text-[#828fa3] truncate">
                            {file.description}
                          </div>
                        {/if}
                      </div>
                    </div>

                    <div class="flex items-center gap-2 shrink-0">
                      {#if file.version}
                        <Badge variant="neutral" mono={true} text="v{file.version}" />
                      {/if}
                      <span style="color: var(--accent-light);" class="text-xs font-mono font-medium">
                        {file.filesize_formatted}
                      </span>
                    </div>
                  </label>
                {/each}
              </div>
            {:else}
              <div class="p-3 bg-[#1e2533] text-xs text-[#828fa3] rounded">
                No files available for direct download.
              </div>
            {/if}

            <!-- Target Hero Assignment -->
            <div class="p-3 bg-[#131720] border border-[#252f40] rounded-[5px] flex items-center justify-between gap-3">
              <div>
                <div class="text-xs font-semibold text-[#f1f5f9]">Hero Assignment</div>
                <div class="text-[10px] text-[#828fa3]">This VPK will be linked to this hero and show in Characters loadout</div>
              </div>
              <select
                bind:value={detectedHero}
                class="h-8 px-2.5 bg-[#1b222f] text-xs text-[#f1f5f9] rounded border border-[#2d3a4e] focus:border-[var(--accent-primary)] cursor-pointer"
              >
                {#each DEADLOCK_HEROES as h}
                  <option value={h}>{h}</option>
                {/each}
              </select>
            </div>

            <!-- Installation Progress & Controls -->
            {#if installState === 'downloading' || installState === 'extracting'}
              <div class="pt-2 space-y-2">
                <ProgressBar
                  value={installProgress}
                  status={installState}
                  message={installMsg}
                  label="Mod install progress"
                />
              </div>
            {:else if installState === 'done'}
              <div style="background-color: var(--accent-badge-bg); border-color: var(--accent-border);" class="p-3 border rounded-[5px] flex items-center justify-between gap-3">
                <div style="color: var(--accent-badge-text);" class="flex items-center gap-2 text-xs font-medium">
                  <span class="font-bold">●</span>
                  <span>Mod installed successfully into <code class="font-mono">game/citadel/addons/</code></span>
                </div>
                <div class="flex items-center gap-2">
                  <Button variant="tonal" size="sm" onclick={openAddonsFolder}>
                    Addons Folder
                  </Button>
                  <Button variant="filled" size="sm" onclick={launchGame}>
                    Launch Deadlock
                  </Button>
                </div>
              </div>
            {:else if installState === 'error'}
              <div class="p-3 bg-[#3b171c] border border-[#ef4444]/40 rounded-[5px] space-y-2">
                <div class="text-xs text-[#fca5a5] font-semibold flex items-center gap-2">
                  <span>{installMsg}</span>
                </div>
                {#if installError}
                  <div class="text-[11px] font-mono text-[#fca5a5]/80">{installError}</div>
                {/if}
                <div class="pt-1">
                  <Button variant="filled" size="sm" onclick={startInstall}>
                    Try Again
                  </Button>
                </div>
              </div>
            {:else}
              <div class="pt-2 flex items-center justify-between gap-3">
                <div class="text-[11px] text-[#828fa3]">
                  Selected: <span class="text-[#f1f5f9] font-medium">{mod.files[selectedFileIndex]?.filename || 'None'}</span> ({mod.files[selectedFileIndex]?.filesize_formatted || '0 B'})
                </div>

                <Button
                  variant="filled"
                  size="md"
                  disabled={!mod.files || mod.files.length === 0}
                  onclick={startInstall}
                >
                  Install to Deadlock
                </Button>
              </div>
            {/if}
          </div>
        {/if}
      </div>

      <!-- Modal Footer -->
      <footer class="px-5 py-3 bg-[#171c27] border-t border-[#252f42] flex items-center justify-between shrink-0">
        <div class="text-[11px] text-[#828fa3]">
          GameBanana API · Deadlock ID 20948
        </div>
        <Button variant="tonal" size="sm" onclick={onClose}>
          Close
        </Button>
      </footer>
    </div>
  </div>
{/if}
