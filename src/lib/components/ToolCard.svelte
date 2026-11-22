<script>
  import Button from './Button.svelte';
  import Badge from './Badge.svelte';
  import ProgressBar from './ProgressBar.svelte';
  import { invoke } from '@tauri-apps/api/core';

  let {
    tool,
    progress = null,
    onLaunch = undefined,
    onOpenFolder = undefined,
    onInstall = undefined,
    onLocate = undefined,
    onRemove = undefined,
  } = $props();

  async function openBrowserUrl() {
    if (tool?.download_url) {
      try {
        await invoke('open_browser_url', { url: tool.download_url });
      } catch (e) {
        window.open(tool.download_url, '_blank');
      }
    }
  }

  let isBusy = $derived(
    progress?.state === 'downloading' ||
    progress?.state === 'extracting' ||
    progress?.state === 'starting'
  );

  function getCategoryVariant(cat) {
    switch (cat) {
      case 'SDK & Compilers': return 'sdk';
      case '3D Modeling': return 'modeling';
      case 'Asset Inspector': return 'inspector';
      case 'Mod Packer & Rigging': return 'packer';
      default: return 'neutral';
    }
  }
</script>

<article
  class="p-4 bg-[#171c26] border border-[#263143] hover:border-[#384863] rounded-[6px] flex flex-col justify-between space-y-3.5 transition-all duration-150"
  aria-labelledby="tool-title-{tool.id}"
>
  <!-- Header row -->
  <div class="space-y-3">
    <div class="flex items-start justify-between gap-3">
      <div class="flex items-center gap-3">
        <!-- Monochrome Tool Icon Box -->
        <div
          class="w-10 h-10 rounded-[5px] bg-[#1e2533] border border-[#2e3b50] flex items-center justify-center text-[#94a3b8] shrink-0 select-none shadow-xs"
          aria-hidden="true"
        >
          {#if tool.id === 'csdk13'}
            <svg class="w-5 h-5 stroke-current fill-none stroke-[1.8]" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" d="M13 10V3L4 14h7v7l9-11h-7z" />
            </svg>
          {:else if tool.id === 'blender'}
            <svg class="w-5 h-5 stroke-current fill-none stroke-[1.8]" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z" />
              <polyline points="3.27 6.96 12 12.01 20.73 6.96" />
              <line x1="12" y1="22.08" x2="12" y2="12" />
            </svg>
          {:else if tool.id === 'source2viewer'}
            <svg class="w-5 h-5 stroke-current fill-none stroke-[1.8]" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z" />
              <circle cx="12" cy="12" r="3" />
            </svg>
          {:else}
            <svg class="w-5 h-5 stroke-current fill-none stroke-[1.8]" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" d="M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.77-3.77a6 6 0 0 1-7.94 7.94l-6.91 6.91a2.12 2.12 0 0 1-3-3l6.91-6.91a6 6 0 0 1 7.94-7.94l-3.76 3.76z" />
            </svg>
          {/if}
        </div>

        <div>
          <div class="flex items-center gap-2">
            <h3 id="tool-title-{tool.id}" class="text-sm font-semibold text-[#f1f5f9] leading-tight">
              {tool.name}
            </h3>
            <Badge variant="neutral" mono={true} text={tool.version_tag} />
          </div>
          <div class="flex items-center gap-2 mt-1">
            <Badge variant={getCategoryVariant(tool.category)} text={tool.category} />
            <span class="text-[10px] font-mono text-[#828fa3]">
              {tool.file_type.toUpperCase()} · {tool.size_estimate}
            </span>
          </div>
        </div>
      </div>

      <!-- State Indicator Pill -->
      <div class="shrink-0">
        {#if tool.installed}
          <span style="background-color: var(--accent-badge-bg); color: var(--accent-badge-text); border-color: var(--accent-border);" class="inline-flex items-center gap-1.5 px-2.5 py-0.5 text-[10px] font-semibold border rounded">
            <span style="background-color: var(--accent-light);" class="w-1.5 h-1.5 rounded-full" aria-hidden="true"></span>
            Ready
          </span>
        {:else if isBusy}
          <span class="inline-flex items-center gap-1.5 px-2.5 py-0.5 text-[10px] font-semibold bg-[#3b2b14] text-[#fcd34d] border border-[#f59e0b]/30 rounded animate-pulse">
            <span class="w-1.5 h-1.5 rounded-full bg-[#f59e0b]" aria-hidden="true"></span>
            {progress.state === 'extracting' ? 'Extracting…' : 'Downloading…'}
          </span>
        {:else}
          <span class="inline-flex items-center gap-1.5 px-2.5 py-0.5 text-[10px] font-medium bg-[#1e2533] text-[#828fa3] border border-[#2d3a4e] rounded">
            Not installed
          </span>
        {/if}
      </div>
    </div>

    <!-- Description -->
    <p class="text-xs text-[#94a3b8] leading-relaxed">
      {tool.description}
    </p>

    <!-- Path box -->
    <div class="px-2.5 py-1.5 bg-[#101319] border border-[#212938] rounded text-[11px] font-mono text-[#828fa3] flex items-center justify-between gap-2">
      <div class="flex items-center gap-1.5 truncate">
        <span style="color: var(--accent-light);" class="font-semibold select-none text-[10px]" aria-hidden="true">PATH:</span>
        <span class="truncate text-[#e2e8f0]">{tool.install_subdir}/{tool.executable}</span>
      </div>
      {#if !tool.installed && onLocate}
        <button
          type="button"
          onclick={onLocate}
          style="color: var(--accent-light);"
          class="shrink-0 text-[10px] hover:underline cursor-pointer font-sans font-medium"
          title="Point to already installed executable on your PC"
        >
          Locate existing
        </button>
      {/if}
    </div>

    <!-- Download Progress View -->
    {#if progress && isBusy}
      <div class="pt-1">
        <ProgressBar
          value={progress.progress * 100}
          status={progress.state}
          message={progress.message}
          label="{tool.name} installation progress"
        />
      </div>
    {:else if progress?.state === 'error'}
      <div role="alert" class="px-3 py-2 bg-[#3b171c] text-[#fca5a5] border border-[#ef4444]/30 text-xs font-mono rounded">
        ERROR: {progress.message}
      </div>
    {/if}
  </div>

  <!-- Actions Toolbar -->
  <div class="pt-2.5 border-t border-[#232c3c] flex items-center justify-between gap-2">
    {#if tool.installed}
      <div class="flex items-center gap-1.5">
        <Button variant="filled" size="sm" onclick={onLaunch}>
          Launch
        </Button>
        <Button variant="tonal" size="sm" onclick={onOpenFolder} title="Open folder in file manager">
          Folder
        </Button>
      </div>

      <div class="flex items-center gap-1.5">
        <Button variant="tonal" size="sm" onclick={onInstall} title="Reinstall tool">
          Reinstall
        </Button>
        <Button variant="destructive" size="sm" onclick={onRemove} title="Remove tool">
          Remove
        </Button>
      </div>
    {:else}
      <div class="flex items-center gap-2 w-full">
        <div class="flex-1">
          <Button
            variant="filled"
            size="md"
            isFullWidth={true}
            onclick={openBrowserUrl}
            title="Open official download site in web browser"
          >
            <span class="inline-flex items-center gap-1.5 justify-center">
              <span>Open in Browser</span>
              <svg class="w-3.5 h-3.5 stroke-current fill-none stroke-[2]" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6" />
                <polyline points="15 3 21 3 21 9" />
                <line x1="10" y1="14" x2="21" y2="3" />
              </svg>
            </span>
          </Button>
        </div>
        {#if onLocate}
          <Button
            variant="tonal"
            size="md"
            title="Locate existing executable on your PC"
            onclick={onLocate}
          >
            Link .exe
          </Button>
        {/if}
      </div>
    {/if}
  </div>
</article>
