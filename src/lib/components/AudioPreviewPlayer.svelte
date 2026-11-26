<!-- AudioPreviewPlayer.svelte — In-app audio preview for sound mods -->
<script lang="ts">
  interface Props {
    src: string;
    label?: string;
  }
  let { src, label = 'Preview Audio' }: Props = $props();

  let audio: HTMLAudioElement | undefined = $state(undefined);
  let paused = $state(true);
  let progress = $state(0);
  let currentTime = $state(0);
  let duration = $state(0);
  let volume = $state(0.8);
  let loadError = $state(false);

  const playing = $derived(!paused);

  function toggle() {
    if (!audio) return;
    loadError = false;
    if (paused) {
      audio.play().catch(() => {
        loadError = true;
      });
    } else {
      audio.pause();
    }
  }

  function onTimeUpdate() {
    if (!audio) return;
    currentTime = audio.currentTime;
    progress = audio.duration ? audio.currentTime / audio.duration : 0;
  }

  function onLoadedMetadata() {
    duration = audio?.duration ?? 0;
    loadError = false;
  }

  function onEnded() {
    paused = true;
    progress = 0;
    currentTime = 0;
  }

  function onError() {
    paused = true;
    loadError = true;
  }

  function seek(e: MouseEvent) {
    if (!audio || !audio.duration) return;
    const bar = e.currentTarget as HTMLElement;
    const rect = bar.getBoundingClientRect();
    const pct = Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width));
    audio.currentTime = pct * audio.duration;
  }

  function formatTime(s: number): string {
    if (!isFinite(s) || s < 0) return '0:00';
    const m = Math.floor(s / 60);
    const sec = Math.floor(s % 60);
    return `${m}:${sec.toString().padStart(2, '0')}`;
  }

  $effect(() => {
    if (audio) audio.volume = volume;
  });
</script>

<div class="flex flex-col gap-2 bg-[#131720] border border-[#252f40] rounded-[6px] p-3 w-full">
  <div class="flex items-center gap-3 w-full">
    <!-- Play/Pause button -->
    <button
      type="button"
      onclick={toggle}
      style="background-color: var(--accent-primary); color: var(--accent-on-primary);"
      class="w-9 h-9 rounded-full flex items-center justify-center hover:opacity-90 transition shrink-0 cursor-pointer"
      title={playing ? 'Pause' : label}
      aria-label={playing ? 'Pause audio' : label}
    >
      {#if playing}
        <svg class="w-4 h-4" viewBox="0 0 24 24" fill="currentColor">
          <rect x="6" y="4" width="4" height="16" /><rect x="14" y="4" width="4" height="16" />
        </svg>
      {:else}
        <svg class="w-4 h-4 translate-x-0.5" viewBox="0 0 24 24" fill="currentColor">
          <polygon points="5,3 19,12 5,21" />
        </svg>
      {/if}
    </button>

    <!-- Progress bar -->
    <div class="flex-1 flex flex-col gap-1 min-w-0">
      <div class="text-[11px] font-medium text-[#cbd5e1] truncate">{label}</div>
      <div
        class="h-1.5 bg-[#242e3f] rounded-full cursor-pointer overflow-hidden"
        onclick={seek}
        onkeydown={(e) => {
          if (!audio || !audio.duration) return;
          if (e.key === 'ArrowRight') audio.currentTime = Math.min(audio.duration, audio.currentTime + 5);
          if (e.key === 'ArrowLeft') audio.currentTime = Math.max(0, audio.currentTime - 5);
        }}
        role="slider"
        aria-label="Audio seekbar"
        aria-valuenow={Math.round(progress * 100)}
        aria-valuemin={0}
        aria-valuemax={100}
        tabindex="0"
      >
        <div
          style="background-color: var(--accent-primary); width:{progress * 100}%"
          class="h-full rounded-full transition-all duration-100"
        ></div>
      </div>
      <div class="flex justify-between text-[10px] font-mono text-[#828fa3]">
        <span>{formatTime(currentTime)}</span>
        <span>{formatTime(duration)}</span>
      </div>
    </div>

    <!-- Volume -->
    <input
      type="range"
      min="0"
      max="1"
      step="0.05"
      bind:value={volume}
      style="accent-color: var(--accent-primary);"
      class="w-16 cursor-pointer"
      title="Volume"
      aria-label="Volume"
    />

    <!-- Hidden audio element -->
    <audio
      bind:this={audio}
      {src}
      bind:paused
      ontimeupdate={onTimeUpdate}
      onloadedmetadata={onLoadedMetadata}
      onended={onEnded}
      onerror={onError}
      class="hidden"
    ></audio>
  </div>

  {#if loadError}
    <p class="text-[11px] text-[#f59e0b]">
      Audio stream preview unavailable for this mirror.
    </p>
  {/if}
</div>
