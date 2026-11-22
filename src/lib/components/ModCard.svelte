<script>
  import Badge from './Badge.svelte';

  let {
    mod,
    onclick = undefined,
  } = $props();

  let imgError = $state(false);

  function getCategoryVariant(cat) {
    if (!cat) return 'neutral';
    const c = cat.toLowerCase();
    if (c.includes('skin') || c.includes('model')) return 'modeling';
    if (c.includes('sound') || c.includes('vo') || c.includes('music')) return 'inspector';
    if (c.includes('hud') || c.includes('ui')) return 'sdk';
    return 'packer';
  }
</script>

<button
  type="button"
  onclick={onclick}
  class="group flex flex-col w-full bg-[#161b25] hover:bg-[#1a212e] border border-[#252f40] hover:border-[var(--accent-primary)] rounded-[6px] overflow-hidden transition-all duration-150 cursor-pointer text-left select-none shadow-xs hover:shadow-md"
  aria-label="{mod.name} by {mod.submitter_name}"
>
  <!-- Thumbnail Cover -->
  <div class="relative w-full aspect-[16/10] bg-[#0d1017] overflow-hidden shrink-0 border-b border-[#212938]">
    {#if mod.thumbnail_url && !imgError}
      <img
        src={mod.thumbnail_url}
        alt={mod.name}
        loading="lazy"
        onerror={() => { imgError = true; }}
        class="w-full h-full object-cover group-hover:scale-105 transition-transform duration-300"
      />
    {:else}
      <div class="w-full h-full flex flex-col items-center justify-center gap-1.5 text-[#55637a] bg-[#11141c]">
        <svg class="w-7 h-7 stroke-current fill-none stroke-[1.5]" viewBox="0 0 24 24">
          <polygon points="12 2 2 7 12 12 22 7 12 2" />
          <polyline points="2 17 12 22 22 17" />
          <polyline points="2 12 12 17 22 12" />
        </svg>
        <span class="text-[10px] font-mono uppercase tracking-wider text-[#738299]">Deadlock Mod</span>
      </div>
    {/if}

    <!-- Category Pill Overlay -->
    <div class="absolute top-2 left-2 z-10 pointer-events-none">
      <Badge variant={getCategoryVariant(mod.category)} text={mod.category} />
    </div>

    <!-- Quick action hover overlay -->
    <div class="absolute inset-0 bg-black/45 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center">
      <span
        style="background-color: var(--accent-primary); color: var(--accent-on-primary);"
        class="px-3 py-1.5 rounded-[4px] font-semibold text-xs shadow-md"
      >
        View & Install
      </span>
    </div>
  </div>

  <!-- Content Details -->
  <div class="p-3.5 flex-1 flex flex-col justify-between space-y-3 w-full">
    <div>
      <h3
        style="--hover-color: var(--accent-light);"
        class="text-xs font-semibold text-[#f1f5f9] group-hover:text-[var(--accent-light)] transition-colors leading-snug line-clamp-2"
      >
        {mod.name}
      </h3>

      <div class="flex items-center gap-2 mt-2">
        {#if mod.submitter_avatar}
          <img
            src={mod.submitter_avatar}
            alt={mod.submitter_name}
            class="w-4 h-4 rounded-full object-cover border border-[#384863] shrink-0"
            onerror={(e) => { e.currentTarget.style.display = 'none'; }}
          />
        {/if}
        <span class="text-[11px] text-[#828fa3] truncate">
          {mod.submitter_name}
        </span>
      </div>
    </div>

    <!-- Footer Stats -->
    <div class="pt-2 border-t border-[#212938] flex items-center justify-between text-[10px] font-mono text-[#828fa3] w-full">
      <div class="flex items-center gap-2.5">
        <span class="flex items-center gap-1">
          <svg class="w-3.5 h-3.5 stroke-current fill-none stroke-[1.8] text-[#828fa3]" viewBox="0 0 24 24">
            <path d="M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z" />
          </svg>
          <span>{mod.likes}</span>
        </span>
        <span class="flex items-center gap-1">
          <svg class="w-3.5 h-3.5 stroke-current fill-none stroke-[1.8] text-[#828fa3]" viewBox="0 0 24 24">
            <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z" />
            <circle cx="12" cy="12" r="3" />
          </svg>
          <span>{mod.views > 999 ? `${Math.round(mod.views / 1000)}k` : mod.views}</span>
        </span>
      </div>

      <span style="color: var(--accent-light);" class="font-sans font-medium text-[11px] group-hover:underline">
        Get Mod →
      </span>
    </div>
  </div>
</button>
