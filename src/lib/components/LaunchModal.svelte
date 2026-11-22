<script>
  import { invoke } from '@tauri-apps/api/core';
  import Button from './Button.svelte';

  let {
    isOpen = false,
    onClose = undefined,
    onLaunch = undefined,
  } = $props();

  let isFixing = $state(false);
  let fixResult = $state(null);
  let fixError = $state('');

  function resetState() {
    isFixing = false;
    fixResult = null;
    fixError = '';
  }

  $effect(() => {
    if (!isOpen) {
      resetState();
    }
  });

  async function handleFixCamera() {
    isFixing = true;
    fixError = '';
    fixResult = null;
    try {
      const res = await invoke('fix_mods_camera');
      fixResult = res;
    } catch (err) {
      fixError = String(err);
    } finally {
      isFixing = false;
    }
  }

  async function handleLaunch() {
    if (onLaunch) {
      onLaunch();
    } else {
      try {
        await invoke('launch_game');
        onClose?.();
      } catch (err) {
        fixError = `Failed to launch: ${err}`;
      }
    }
  }

  function handleKeydown(e) {
    if (!isOpen) return;
    if (e.key === 'Escape') {
      e.preventDefault();
      onClose?.();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if isOpen}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/80 backdrop-blur-xs"
    role="presentation"
    onclick={(e) => {
      if (e.target === e.currentTarget) onClose?.();
    }}
  >
    <div
      class="max-w-md w-full bg-[#131720] border border-[#2a3547] rounded-[6px] shadow-2xl overflow-hidden text-[#f1f5f9] animate-in fade-in zoom-in-95 duration-150"
      role="dialog"
      aria-modal="true"
      tabindex="-1"
      aria-labelledby="launch-modal-title"
    >
      <!-- Header -->
      <header class="px-5 py-3.5 bg-[#171c26] border-b border-[#242f40] flex items-center justify-between">
        <div class="flex items-center gap-2.5">
          <div class="w-7 h-7 rounded-[4px] bg-[#291e12] border border-[#d97706]/40 text-[#f59e0b] flex items-center justify-center text-xs shrink-0 font-bold">
            !
          </div>
          <h2 id="launch-modal-title" class="text-sm font-semibold text-[#f1f5f9]">
            Launch Deadlock with Mods
          </h2>
        </div>

        <button
          type="button"
          onclick={onClose}
          class="w-7 h-7 rounded-[4px] bg-[#1e2533] hover:bg-[#283244] border border-[#2e3b50] text-[#94a3b8] hover:text-[#f1f5f9] flex items-center justify-center text-xs transition cursor-pointer"
          title="Close dialog (Esc)"
          aria-label="Close dialog"
        >
          ✕
        </button>
      </header>

      <!-- Content -->
      <div class="p-5 space-y-4">
        <!-- Warning Message Box -->
        <div class="p-3.5 bg-[#1e1915] border border-[#78350f]/60 rounded-[5px] space-y-2">
          <div class="flex items-center gap-2 text-xs font-semibold text-[#fbbf24]">
            <svg class="w-4 h-4 stroke-current fill-none stroke-[2] shrink-0" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
            </svg>
            <span>Violet Update Camera Notice</span>
          </div>
          <p class="text-xs text-[#d1d5db] leading-relaxed">
            In Deadlock's latest engine update (Violet branch), third-person camera attachment offsets broke compatibility with older custom hero models and skins.
          </p>
          <p class="text-[11px] text-[#9ca3af] leading-normal font-mono">
            Execute the camera calibration fix below to enforce standard FOV, shoulder offsets, and cvar values in your Citadel autoexec before launching.
          </p>
        </div>

        <!-- Fix Result / Status -->
        {#if isFixing}
          <div class="p-3 bg-[#171d28] border border-[#2b394e] rounded-[5px] flex items-center gap-3">
            <div style="border-top-color: var(--accent-primary);" class="w-4 h-4 rounded-full border-2 border-white/20 animate-spin shrink-0"></div>
            <span class="text-xs text-[#94a3b8]">Calibrating camera offsets & generating autoexec fix…</span>
          </div>
        {:else if fixResult}
          <div class="p-3 bg-[#0d2216] border border-[#10b981]/40 rounded-[5px] space-y-1">
            <div class="flex items-center gap-2 text-xs font-semibold text-[#34d399]">
              <svg class="w-4 h-4 stroke-current fill-none stroke-[2]" viewBox="0 0 24 24">
                <polyline points="20 6 9 17 4 12" />
              </svg>
              <span>Camera Calibration Applied</span>
            </div>
            <p class="text-[11px] font-mono text-[#a7f3d0] leading-snug">
              {fixResult.message}
            </p>
          </div>
        {:else if fixError}
          <div class="p-3 bg-[#331114] border border-[#ef4444]/40 rounded-[5px] text-xs text-[#fca5a5] font-mono">
            {fixError}
          </div>
        {/if}

        <!-- Calibration Action -->
        <div class="pt-1">
          <Button
            variant="tonal"
            size="md"
            isFullWidth={true}
            disabled={isFixing}
            onclick={handleFixCamera}
            title="Inject camera offset fix into Citadel autoexec"
          >
            <span class="inline-flex items-center justify-center gap-2">
              <svg class="w-3.5 h-3.5 stroke-current fill-none stroke-[2]" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" d="M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.77-3.77a6 6 0 0 1-7.94 7.94l-6.91 6.91a2.12 2.12 0 0 1-3-3l6.91-6.91a6 6 0 0 1 7.94-7.94l-3.76 3.76z" />
              </svg>
              <span>{fixResult ? 'Re-apply Camera Fix' : 'Fix Mods Camera'}</span>
            </span>
          </Button>
        </div>
      </div>

      <!-- Footer Buttons -->
      <footer class="px-5 py-3 bg-[#151923] border-t border-[#232c3d] flex items-center justify-between gap-3">
        <Button variant="ghost" size="sm" onclick={onClose}>
          Cancel
        </Button>
        <Button variant="filled" size="sm" onclick={handleLaunch}>
          <span class="inline-flex items-center gap-1.5">
            <svg class="w-3.5 h-3.5 fill-current" viewBox="0 0 24 24">
              <polygon points="5 3 19 12 5 21 5 3" />
            </svg>
            <span>Proceed & Launch Deadlock</span>
          </span>
        </Button>
      </footer>
    </div>
  </div>
{/if}
