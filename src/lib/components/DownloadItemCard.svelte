<!-- DownloadItemCard.svelte — Single download card in Downloads tab -->
<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import type { DownloadItem } from '$lib/stores/downloads';
  import { computeEta, cancelDownload } from '$lib/stores/downloads';
  import { t } from '$lib/i18n';
  import HeroIcon from './HeroIcon.svelte';

  interface Props {
    item: DownloadItem;
  }
  let { item }: Props = $props();

  const statusColors: Record<string, string> = {
    downloading: 'text-[#38bdf8]',
    extracting: 'text-[#fbbf24]',
    done: 'text-[#34d399]',
    error: 'text-[#ef4444]',
    cancelled: 'text-[#64748b]',
  };

  const isActive = $derived(item.status === 'downloading' || item.status === 'extracting');
  const eta = $derived(computeEta(item));
  const color = $derived(statusColors[item.status] ?? 'text-[#94a3b8]');
  const pct = $derived(Math.min(100, Math.max(0, Math.round(item.progress * 100))));

  async function openFolder() {
    try {
      await invoke('open_game_addons_folder');
    } catch {}
  }
</script>

<div class="bg-[#171c26] border border-[#252f40] rounded-[6px] p-4 flex gap-4 items-start">
  <!-- Thumbnail -->
  {#if item.thumbnail_url}
    <img src={item.thumbnail_url} alt="" class="w-14 h-14 rounded-[5px] object-cover border border-[#2b374a] shrink-0" />
  {:else if item.hero_name}
    <div class="w-14 h-14 rounded-[5px] bg-[#11141c] border border-[#242e40] flex items-center justify-center shrink-0">
      <HeroIcon name={item.hero_name} size={40} />
    </div>
  {:else}
    <div class="w-14 h-14 rounded-[5px] bg-[#11141c] border border-[#242e40] flex items-center justify-center shrink-0">
      <svg class="w-6 h-6 text-[#5d6b82]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5" />
      </svg>
    </div>
  {/if}

  <div class="flex-1 min-w-0">
    <!-- Title row -->
    <div class="flex items-center justify-between gap-2 mb-1">
      <div class="flex items-center gap-2 min-w-0">
        <span class="font-semibold text-sm text-[#f1f5f9] truncate">
          {item.mod_name ?? item.filename}
        </span>
        {#if item.hero_name}
          <span class="px-1.5 py-0.5 text-[10px] font-mono rounded bg-[#1b2536] text-[#94a3b8] border border-[#2a364a] shrink-0">
            {item.hero_name}
          </span>
        {:else}
          <span class="px-1.5 py-0.5 text-[10px] font-mono rounded bg-[#1b222f] text-[#828fa3] border border-[#263143] shrink-0">
            {t('common.global_mod')}
          </span>
        {/if}
      </div>
      <span class="text-[11px] font-mono {color} shrink-0 font-bold">
        {item.status.toUpperCase()} {isActive ? `${pct}%` : ''}
      </span>
    </div>

    <!-- Filename -->
    <p class="text-[11px] font-mono text-[#828fa3] truncate mb-2">{item.filename}</p>

    <!-- Progress bar -->
    {#if isActive}
      <div class="h-1.5 bg-[#0e1117] rounded-full overflow-hidden mb-2 border border-[#232d3d]">
        <div
          style="background-color: var(--accent-primary); width:{pct}%"
          class="h-full rounded-full transition-all duration-200"
        ></div>
      </div>
      <div class="flex justify-between text-[11px] font-mono text-[#828fa3]">
        <span class="truncate">{item.message}</span>
        <span class="shrink-0 ml-2">{t('downloads.eta')}: {eta}</span>
      </div>
    {:else}
      <p class="text-xs text-[#94a3b8]">{item.message}</p>
    {/if}
  </div>

  <!-- Action buttons -->
  <div class="flex items-center gap-2 shrink-0">
    {#if isActive}
      <button
        type="button"
        onclick={() => cancelDownload(item.mod_id, item.filename)}
        class="text-[11px] text-[#94a3b8] hover:text-[#ef4444] border border-[#2b374a] rounded-[4px] px-2.5 py-1 transition cursor-pointer"
      >
        {t('downloads.cancel')}
      </button>
    {:else if item.status === 'done'}
      <button
        type="button"
        onclick={openFolder}
        class="text-[11px] text-[#94a3b8] hover:text-[#f1f5f9] bg-[#1e2533] hover:bg-[#283245] border border-[#2b374a] rounded-[4px] px-2.5 py-1 transition cursor-pointer"
      >
        {t('downloads.show_folder')}
      </button>
    {/if}
  </div>
</div>
