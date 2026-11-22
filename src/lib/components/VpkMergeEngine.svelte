<script>
  import { invoke } from '@tauri-apps/api/core';
  import Button from './Button.svelte';
  import Badge from './Badge.svelte';

  let {
    installedAddons = [],
    onMergeCompleted = undefined,
  } = $props();

  let vpkA = $state('');
  let vpkB = $state('');
  let outputName = $state('fused_custom_mod.vpk');

  let takeModels = $state('a'); // 'a' | 'b' | 'both'
  let takeSounds = $state('b'); // 'a' | 'b' | 'both'

  let isMerging = $state(false);
  let mergeResult = $state('');
  let mergeError = $state('');

  let manifestA = $state(null);
  let manifestB = $state(null);

  $effect(() => {
    if (vpkA) {
      invoke('inspect_vpk', { filename: vpkA })
        .then(m => { manifestA = m; })
        .catch(() => { manifestA = null; });
    } else {
      manifestA = null;
    }
  });

  $effect(() => {
    if (vpkB) {
      invoke('inspect_vpk', { filename: vpkB })
        .then(m => { manifestB = m; })
        .catch(() => { manifestB = null; });
    } else {
      manifestB = null;
    }
  });

  async function executeMerge() {
    if (!vpkA || !vpkB) {
      mergeError = 'Please select both source VPK files to merge.';
      return;
    }
    if (vpkA === vpkB) {
      mergeError = 'Source VPK A and VPK B must be different packages.';
      return;
    }

    isMerging = true;
    mergeError = '';
    mergeResult = '';

    try {
      await invoke('merge_addon_vpks', {
        vpkA,
        vpkB,
        outputFilename: outputName,
        takeModelsFrom: takeModels,
        takeSoundsFrom: takeSounds,
        heroName: null,
      });
      mergeResult = `Successfully fused into ${outputName}! Deployed to citadel/addons.`;
      onMergeCompleted?.();
    } catch (e) {
      mergeError = String(e);
    } finally {
      isMerging = false;
    }
  }
</script>

<div class="max-w-4xl mx-auto p-6 bg-[#131720] border border-[#232d3d] rounded-[8px] space-y-6 text-[#f1f5f9] select-none shadow-xl">
  <!-- Header -->
  <div class="border-b border-[#212938] pb-4">
    <div class="flex items-center gap-2">
      <span style="color: var(--accent-light);" class="text-sm font-bold">VPK Enhanced</span>
      <Badge variant="theme" text="Fusion Engine" />
    </div>
    <h2 class="text-base font-bold text-[#f1f5f9] mt-1">Multi-VPK Component Merger</h2>
    <p class="text-xs text-[#828fa3] mt-1">
      Merge models, materials, textures, and sounds from different VPK mods into a single consolidated package. For example, take the 3D model from Mod A and custom ultimate audio from Mod B!
    </p>
  </div>

  <!-- Dual Selector Grid -->
  <div class="grid grid-cols-1 md:grid-cols-2 gap-5">
    <!-- Source VPK A -->
    <div class="p-4 bg-[#0d1016] border border-[#20293a] rounded-[6px] space-y-3">
      <div class="flex items-center justify-between">
        <span class="text-xs font-bold text-[#38bdf8]">Source VPK A (Visuals / Base)</span>
        <Badge variant="neutral" text="Package A" />
      </div>

      <select
        bind:value={vpkA}
        class="h-9 w-full px-3 bg-[#151a26] text-xs text-[#f1f5f9] rounded border border-[#263143] focus:border-[var(--accent-primary)] cursor-pointer"
      >
        <option value="">-- Select Installed VPK --</option>
        {#each installedAddons as addon}
          <option value={addon.filename}>{addon.filename} ({addon.detected_hero || 'Universal'})</option>
        {/each}
      </select>

      {#if manifestA}
        <div class="p-3 bg-[#121622] border border-[#232d3e] rounded-[5px] space-y-1 text-[11px] font-mono text-[#828fa3]">
          <div class="flex justify-between text-[#cbd5e1]">
            <span>Files: {manifestA.total_files}</span>
            <span>{(manifestA.total_size / 1048576).toFixed(1)} MB</span>
          </div>
          <div class="flex gap-2 pt-1">
            {#if manifestA.has_models}<span class="text-[#38bdf8]">✓ Models</span>{/if}
            {#if manifestA.has_materials}<span class="text-[#34d399]">✓ Textures</span>{/if}
            {#if manifestA.has_sounds}<span class="text-[#c084fc]">✓ Sounds</span>{/if}
          </div>
        </div>
      {/if}
    </div>

    <!-- Source VPK B -->
    <div class="p-4 bg-[#0d1016] border border-[#20293a] rounded-[6px] space-y-3">
      <div class="flex items-center justify-between">
        <span class="text-xs font-bold text-[#c084fc]">Source VPK B (Audio / Extensions)</span>
        <Badge variant="neutral" text="Package B" />
      </div>

      <select
        bind:value={vpkB}
        class="h-9 w-full px-3 bg-[#151a26] text-xs text-[#f1f5f9] rounded border border-[#263143] focus:border-[var(--accent-primary)] cursor-pointer"
      >
        <option value="">-- Select Installed VPK --</option>
        {#each installedAddons as addon}
          <option value={addon.filename}>{addon.filename} ({addon.detected_hero || 'Universal'})</option>
        {/each}
      </select>

      {#if manifestB}
        <div class="p-3 bg-[#121622] border border-[#232d3e] rounded-[5px] space-y-1 text-[11px] font-mono text-[#828fa3]">
          <div class="flex justify-between text-[#cbd5e1]">
            <span>Files: {manifestB.total_files}</span>
            <span>{(manifestB.total_size / 1048576).toFixed(1)} MB</span>
          </div>
          <div class="flex gap-2 pt-1">
            {#if manifestB.has_models}<span class="text-[#38bdf8]">✓ Models</span>{/if}
            {#if manifestB.has_materials}<span class="text-[#34d399]">✓ Textures</span>{/if}
            {#if manifestB.has_sounds}<span class="text-[#c084fc]">✓ Sounds</span>{/if}
          </div>
        </div>
      {/if}
    </div>
  </div>

  <!-- Component Routing Matrix -->
  <div class="p-4 bg-[#0d1016] border border-[#20293a] rounded-[6px] space-y-4">
    <div class="text-xs font-bold text-[#f1f5f9]">Fusion Composition Rules</div>

    <div class="grid grid-cols-1 sm:grid-cols-2 gap-4 text-xs">
      <!-- Models Rule -->
      <div class="p-3 bg-[#141926] border border-[#232d3e] rounded-[5px] space-y-2">
        <div class="font-semibold text-[#38bdf8]">3D Models & Materials</div>
        <div class="space-y-1">
          <label class="flex items-center gap-2 cursor-pointer">
            <input type="radio" bind:group={takeModels} value="a" style="accent-color: var(--accent-primary);" />
            <span>Take from VPK A</span>
          </label>
          <label class="flex items-center gap-2 cursor-pointer">
            <input type="radio" bind:group={takeModels} value="b" style="accent-color: var(--accent-primary);" />
            <span>Take from VPK B</span>
          </label>
          <label class="flex items-center gap-2 cursor-pointer">
            <input type="radio" bind:group={takeModels} value="both" style="accent-color: var(--accent-primary);" />
            <span>Merge Both (Non-conflicting)</span>
          </label>
        </div>
      </div>

      <!-- Sounds Rule -->
      <div class="p-3 bg-[#141926] border border-[#232d3e] rounded-[5px] space-y-2">
        <div class="font-semibold text-[#c084fc]">Audio, SFX & Voicelines</div>
        <div class="space-y-1">
          <label class="flex items-center gap-2 cursor-pointer">
            <input type="radio" bind:group={takeSounds} value="b" style="accent-color: var(--accent-primary);" />
            <span>Take from VPK B (Recommended)</span>
          </label>
          <label class="flex items-center gap-2 cursor-pointer">
            <input type="radio" bind:group={takeSounds} value="a" style="accent-color: var(--accent-primary);" />
            <span>Take from VPK A</span>
          </label>
          <label class="flex items-center gap-2 cursor-pointer">
            <input type="radio" bind:group={takeSounds} value="both" style="accent-color: var(--accent-primary);" />
            <span>Merge Both</span>
          </label>
        </div>
      </div>
    </div>

    <!-- Output VPK Name -->
    <div class="space-y-1.5 pt-2 border-t border-[#1e2535]">
      <label class="text-[11px] font-mono text-[#828fa3]">Output Merged VPK Filename</label>
      <input
        type="text"
        bind:value={outputName}
        class="h-9 w-full max-w-md px-3 bg-[#141926] text-xs font-mono text-[#f1f5f9] rounded border border-[#2a364a] focus:border-[var(--accent-primary)] transition"
      />
    </div>
  </div>

  <!-- Status / Errors -->
  {#if isMerging}
    <div class="p-4 bg-[#171e2b] border border-[#28364b] rounded-[6px] flex items-center justify-center gap-3">
      <div style="border-top-color: var(--accent-primary);" class="w-4 h-4 rounded-full border-2 border-white/20 animate-spin"></div>
      <span class="text-xs font-mono text-[#cbd5e1]">Decompressing, fusing directory trees and packaging VPK…</span>
    </div>
  {:else if mergeResult}
    <div class="p-3 bg-[#0d2216] border border-[#10b981]/40 rounded-[6px] text-xs font-mono text-[#34d399]">
      {mergeResult}
    </div>
  {:else if mergeError}
    <div class="p-3 bg-[#331114] border border-[#ef4444]/40 rounded-[6px] text-xs font-mono text-[#fca5a5]">
      {mergeError}
    </div>
  {/if}

  <!-- Action -->
  <Button
    variant="filled"
    size="md"
    isFullWidth={true}
    disabled={isMerging || !vpkA || !vpkB}
    onclick={executeMerge}
  >
    Execute Enhanced VPK Fusion
  </Button>
</div>
