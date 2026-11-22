<script>
  import Button from './Button.svelte';

  let {
    isOpen = false,
    title = '',
    description = '',
    confirmLabel = 'Confirm',
    cancelLabel = 'Cancel',
    isDestructive = false,
    onConfirm = undefined,
    onCancel = undefined,
  } = $props();

  let cancelBtnRef = $state(null);

  $effect(() => {
    if (isOpen) {
      setTimeout(() => {
        cancelBtnRef?.focus();
      }, 50);
    }
  });

  function handleKeydown(event) {
    if (!isOpen) return;
    if (event.key === 'Escape') {
      event.preventDefault();
      onCancel?.();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if isOpen}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/70 backdrop-blur-xs"
    role="presentation"
    onclick={(e) => {
      if (e.target === e.currentTarget) onCancel?.();
    }}
  >
    <div
      role="alertdialog"
      aria-modal="true"
      aria-labelledby="dialog-title"
      aria-describedby="dialog-desc"
      class="max-w-md w-full p-5 bg-[#171c26] border border-[#2b374a] rounded-[6px] shadow-2xl space-y-4"
    >
      <div class="space-y-2">
        <h3 id="dialog-title" class="text-sm font-semibold text-[#f1f5f9]">
          {title}
        </h3>
        <p id="dialog-desc" class="text-xs text-[#94a3b8] leading-relaxed">
          {description}
        </p>
      </div>

      <div class="flex items-center justify-end gap-2 pt-3 border-t border-[#232c3c]">
        <button
          bind:this={cancelBtnRef}
          type="button"
          onclick={onCancel}
          class="h-8 px-3 text-xs font-medium rounded-[5px] bg-[#202735] hover:bg-[#283142] border border-[#313c4f] text-[#e2e8f0] transition cursor-pointer"
        >
          {cancelLabel}
        </button>

        <Button
          variant={isDestructive ? 'destructive' : 'filled'}
          size="md"
          onclick={onConfirm}
        >
          {confirmLabel}
        </Button>
      </div>
    </div>
  </div>
{/if}
