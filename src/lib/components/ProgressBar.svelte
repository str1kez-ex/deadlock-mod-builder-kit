<script>
  let {
    value = 0,
    status = 'downloading',
    message = '',
    label = 'Progress',
  } = $props();

  let percent = $derived(Math.min(100, Math.max(0, Math.round(value))));
</script>

<div class="w-full space-y-1.5" role="region" aria-label={label}>
  {#if message || percent !== undefined}
    <div class="flex items-center justify-between text-[11px] font-mono leading-tight" aria-live="polite">
      <span class="text-[#94a3b8] truncate pr-2">{message}</span>
      <span style="color: var(--accent-light);" class="font-bold shrink-0">{percent}%</span>
    </div>
  {/if}

  <div
    role="progressbar"
    aria-valuenow={percent}
    aria-valuemin="0"
    aria-valuemax="100"
    aria-label={label}
    class="h-2 w-full bg-[#141822] rounded overflow-hidden border border-[#263042]"
  >
    <div
      class="h-full transition-all duration-200 ease-out rounded
        {status === 'error'
          ? 'bg-[#ef4444]'
          : status === 'extracting'
            ? 'animate-pulse'
            : ''}"
      style="width: {Math.max(2, percent)}%; background-color: {status === 'error' ? '#ef4444' : 'var(--accent-primary)'};"
    ></div>
  </div>
</div>
