<script>
  let {
    id,
    label = '',
    checked = false,
    disabled = false,
    onchange = undefined,
  } = $props();

  function toggle() {
    if (disabled) return;
    onchange?.(!checked);
  }

  function handleKeydown(event) {
    if (disabled) return;
    if (event.key === ' ' || event.key === 'Enter') {
      event.preventDefault();
      toggle();
    }
  }
</script>

<div class="inline-flex items-center gap-2.5 select-none">
  <button
    {id}
    type="button"
    role="switch"
    aria-checked={checked}
    aria-label={label || 'Toggle switch'}
    disabled={disabled}
    onclick={toggle}
    onkeydown={handleKeydown}
    style={checked ? 'background-color: var(--accent-primary); border-color: var(--accent-primary);' : undefined}
    class="relative inline-flex h-5 w-9 shrink-0 cursor-pointer rounded-full border transition-colors duration-150 ease-in-out
      {checked ? '' : 'bg-[#1e2533] border-[#364257]'}
      {disabled ? 'opacity-40 cursor-not-allowed pointer-events-none' : ''}"
  >
    <span
      aria-hidden="true"
      style={checked ? 'background-color: var(--accent-on-primary);' : undefined}
      class="pointer-events-none inline-flex items-center justify-center transform rounded-full shadow-xs transition-all duration-150 ease-in-out
        {checked
          ? 'translate-x-[17px] w-3.5 h-3.5 mt-[2px]'
          : 'translate-x-[2px] bg-[#828fa3] w-3.5 h-3.5 mt-[2px]'}"
    >
    </span>
  </button>

  {#if label}
    <label for={id} class="text-xs font-medium text-[#e2e8f0] cursor-pointer">
      {label}
    </label>
  {/if}
</div>
