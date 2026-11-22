<script>
  let {
    id,
    label = '',
    value = '',
    placeholder = '',
    readonly = false,
    isMonospace = false,
    helperText = '',
    errorMessage = '',
    status = 'default',
    actionLabel = '',
    onaction = undefined,
    oninput = undefined,
  } = $props();

  let hasError = $derived(status === 'error' || Boolean(errorMessage));
  let describedBy = $derived(
    hasError ? `${id}-error` : helperText ? `${id}-helper` : undefined
  );
</script>

<div class="space-y-1 w-full">
  {#if label}
    <label for={id} class="text-xs font-medium text-[#94a3b8] tracking-wide block">
      {label}
    </label>
  {/if}

  <div class="flex items-center gap-2">
    <div class="relative flex-1 flex items-center">
      <input
        {id}
        type="text"
        {value}
        {placeholder}
        {readonly}
        aria-invalid={hasError ? 'true' : 'false'}
        aria-describedby={describedBy}
        oninput={oninput}
        class="h-8 w-full px-3 py-1.5 bg-[#101319] text-xs text-[#f1f5f9] placeholder-[#55637a] rounded-[5px] border transition-colors
          {isMonospace ? 'font-mono' : 'font-sans'}
          {hasError
            ? 'border-[#ef4444] text-[#fca5a5]'
            : 'border-[#263143] hover:border-[#384863] focus:border-[#2ea069]'}
          {readonly ? 'cursor-default select-all' : ''}"
      />
    </div>

    {#if actionLabel && onaction}
      <button
        type="button"
        onclick={onaction}
        class="h-8 px-3 text-xs font-medium rounded-[5px] bg-[#202735] hover:bg-[#283142] border border-[#313c4f] text-[#e2e8f0] transition cursor-pointer active:scale-95"
      >
        {actionLabel}
      </button>
    {/if}
  </div>

  {#if hasError && errorMessage}
    <p id="{id}-error" role="alert" class="text-[11px] font-mono text-[#f87171] flex items-center gap-1.5 pt-0.5">
      <span aria-hidden="true">⚠️</span>
      <span>{errorMessage}</span>
    </p>
  {:else if helperText}
    <p id="{id}-helper" class="text-[11px] text-[#64748b] pt-0.5">
      {helperText}
    </p>
  {/if}
</div>
