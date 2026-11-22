<script>
  import { invoke } from '@tauri-apps/api/core';
  import Button from './Button.svelte';
  import Badge from './Badge.svelte';

  let {
    selectedMod = null,
    hero = 'Abrams',
    heroIcon = '',
    onClose = undefined,
    onToggleEquip = undefined,
    onDelete = undefined,
  } = $props();

  let vpkManifest = $state(null);
  let loadingManifest = $state(false);
  let manifestError = $state('');

  let fixingCamera = $state(false);
  let cameraFixMsg = $state('');

  $effect(() => {
    if (selectedMod && !selectedMod.isDefault && selectedMod.filename) {
      loadVpkManifest(selectedMod.filename);
    } else {
      vpkManifest = null;
      manifestError = '';
    }
    cameraFixMsg = '';
  });

  async function loadVpkManifest(filename) {
    loadingManifest = true;
    manifestError = '';
    vpkManifest = null;
    try {
      vpkManifest = await invoke('inspect_vpk', { filename });
    } catch (e) {
      manifestError = String(e);
    } finally {
      loadingManifest = false;
    }
  }

  async function handleFixCamera() {
    fixingCamera = true;
    cameraFixMsg = '';
    try {
      const res = await invoke('fix_mods_camera');
      cameraFixMsg = res.message;
    } catch (e) {
      cameraFixMsg = `Error: ${e}`;
    } finally {
      fixingCamera = false;
    }
  }

  function formatBytes(bytes) {
    if (!bytes || bytes === 0) return '0 B';
    const mb = bytes / 1048576;
    if (mb >= 1) return `${mb.toFixed(1)} MB`;
    return `${(bytes / 1024).toFixed(0)} KB`;
  }
</script>

{#if selectedMod}
  <aside
    class="w-full h-full bg-[#121620] border border-[#232d3d] rounded-[6px] flex flex-col overflow-hidden text-[#f1f5f9] animate-in fade-in duration-150 select-none shadow-lg"
    aria-label="VPK Asset Inspector"
  >
    <!-- Header -->
    <header class="px-4 py-3 bg-[#151a26] border-b border-[#212938] flex items-center justify-between shrink-0">
      <div class="flex items-center gap-2 truncate">
        <span class="w-2 h-2 rounded-full" style="background-color: var(--accent-light);"></span>
        <h3 class="text-xs font-bold text-[#f1f5f9] truncate">
          {selectedMod.isDefault ? `${hero} Vanilla Asset` : 'VPK Package Inspector'}
        </h3>
      </div>

      <button
        type="button"
        onclick={onClose}
        class="w-6 h-6 rounded-[3px] bg-[#1e2534] hover:bg-[#2b3548] border border-[#2d384c] text-[#94a3b8] hover:text-[#f1f5f9] flex items-center justify-center text-xs transition cursor-pointer"
        title="Collapse Inspector"
        aria-label="Close Inspector"
      >
        ✕
      </button>
    </header>

    <!-- Scrollable Body -->
    <div class="flex-1 overflow-y-auto p-4 space-y-4">

      <!-- Cover / Artwork Preview -->
      <div class="relative w-full h-44 bg-[#0c0e14] border border-[#242e40] rounded-[5px] overflow-hidden flex items-center justify-center">
        {#if selectedMod.thumbnail_url}
          <img
            src={selectedMod.thumbnail_url}
            alt={selectedMod.mod_title || selectedMod.filename}
            class="w-full h-full object-cover"
          />
        {:else if heroIcon}
          <div class="relative w-full h-full flex flex-col items-center justify-center bg-radial from-[#1e2638] to-[#0c0e14] p-4">
            <img
              src={heroIcon}
              alt={hero}
              class="w-20 h-20 object-contain drop-shadow-xl"
            />
            <div class="text-[11px] font-mono text-[#828fa3] mt-2">Source 2 Package</div>
          </div>
        {:else}
          <div class="text-xs font-mono text-[#5d6b82]">No Preview Thumbnail</div>
        {/if}

        <div class="absolute bottom-2 left-2 right-2 flex items-center justify-between">
          <Badge
            variant={selectedMod.enabled ? 'theme' : 'neutral'}
            text={selectedMod.enabled ? 'Active In-Game' : 'Disabled'}
          />
          {#if selectedMod.size_bytes}
            <span class="px-2 py-0.5 rounded text-[10px] font-mono bg-black/70 text-[#cbd5e1] backdrop-blur-xs">
              {formatBytes(selectedMod.size_bytes)}
            </span>
          {/if}
        </div>
      </div>

      <!-- Title & Origin -->
      <div class="space-y-1">
        <h4 class="text-sm font-bold text-[#f1f5f9] leading-snug break-words">
          {selectedMod.mod_title || selectedMod.filename}
        </h4>
        <div class="text-xs text-[#828fa3] flex items-center gap-2">
          <span>Author: <strong class="text-[#e2e8f0] font-medium">{selectedMod.author || 'Official Valve Asset'}</strong></span>
          {#if selectedMod.gamebanana_id}
            <span>·</span>
            <a
              href="https://gamebanana.com/mods/{selectedMod.gamebanana_id}"
              target="_blank"
              rel="noreferrer"
              style="color: var(--accent-light);"
              class="hover:underline font-mono text-[11px]"
            >
              GB #{selectedMod.gamebanana_id} ↗
            </a>
          {/if}
        </div>
      </div>

      <!-- Technical Meta Specs -->
      <div class="p-3 bg-[#0d1017] border border-[#1f2838] rounded-[5px] space-y-2 text-xs font-mono">
        <div class="flex items-center justify-between">
          <span class="text-[#828fa3]">Assigned Hero</span>
          <span class="text-[#f1f5f9] font-semibold">{hero}</span>
        </div>
        <div class="flex items-center justify-between">
          <span class="text-[#828fa3]">Package Format</span>
          <span class="text-[#cbd5e1]">{selectedMod.isDefault ? 'Source 2 Base VPK' : 'Custom Addon VPK'}</span>
        </div>
        {#if selectedMod.filename}
          <div class="flex items-center justify-between">
            <span class="text-[#828fa3]">Disk File</span>
            <span class="text-[#a0aec0] truncate max-w-[170px]" title={selectedMod.filename}>
              {selectedMod.filename}
            </span>
          </div>
        {/if}
      </div>

      <!-- VPK Internal File Structure (Inspected from VPK bytes) -->
      {#if !selectedMod.isDefault}
        <div class="space-y-2">
          <div class="flex items-center justify-between text-xs">
            <span class="font-bold text-[#f1f5f9]">VPK Internal Files</span>
            {#if vpkManifest}
              <span class="text-[10px] font-mono text-[#828fa3]">
                {vpkManifest.total_files} files · {formatBytes(vpkManifest.total_size)}
              </span>
            {/if}
          </div>

          {#if loadingManifest}
            <div class="p-4 bg-[#0d1017] border border-[#1f2838] rounded-[5px] flex items-center justify-center gap-2 text-xs text-[#828fa3]">
              <div style="border-top-color: var(--accent-primary);" class="w-3.5 h-3.5 rounded-full border-2 border-white/20 animate-spin"></div>
              <span>Reading VPK directory tree…</span>
            </div>
          {:else if manifestError}
            <div class="p-3 bg-[#331114] border border-[#ef4444]/30 rounded text-[11px] font-mono text-[#fca5a5]">
              {manifestError}
            </div>
          {:else if vpkManifest}
            <!-- Component badges -->
            <div class="flex flex-wrap gap-1.5">
              {#if vpkManifest.has_models}
                <span class="px-2 py-0.5 text-[10px] font-mono rounded bg-[#1e293b] text-[#38bdf8] border border-[#0284c7]/30">
                  3D Models
                </span>
              {/if}
              {#if vpkManifest.has_sounds}
                <span class="px-2 py-0.5 text-[10px] font-mono rounded bg-[#2e1d3d] text-[#c084fc] border border-[#9333ea]/30">
                  Audio / SFX
                </span>
              {/if}
              {#if vpkManifest.has_materials}
                <span class="px-2 py-0.5 text-[10px] font-mono rounded bg-[#1f2d24] text-[#34d399] border border-[#10b981]/30">
                  Textures
                </span>
              {/if}
            </div>

            <!-- File tree list -->
            <div class="max-h-40 overflow-y-auto p-2 bg-[#0c0e14] border border-[#1e2535] rounded-[4px] space-y-1 font-mono text-[10px]">
              {#each vpkManifest.files as file}
                <div class="flex items-center justify-between text-[#828fa3] hover:text-[#f1f5f9] truncate py-0.5">
                  <span class="truncate">{file.full_path}</span>
                  <span class="shrink-0 text-[#64748b] ml-2">{(file.size / 1024).toFixed(0)}K</span>
                </div>
              {/each}
            </div>
          {/if}
        </div>
      {/if}

      <!-- Camera Calibration Quick Action -->
      {#if !selectedMod.isDefault}
        <div class="p-3 bg-[#171c26] border border-[#252f40] rounded-[5px] space-y-2">
          <div class="text-[11px] font-semibold text-[#f1f5f9] flex items-center justify-between">
            <span>Violet Camera Offset Calibration</span>
            {#if fixingCamera}
              <div style="border-top-color: var(--accent-primary);" class="w-3 h-3 rounded-full border-2 border-white/20 animate-spin"></div>
            {/if}
          </div>
          <p class="text-[10px] text-[#828fa3] leading-normal">
            Enforces standard 3rd-person camera cvars for older hero models.
          </p>
          <Button
            variant="tonal"
            size="sm"
            isFullWidth={true}
            disabled={fixingCamera}
            onclick={handleFixCamera}
          >
            Apply Camera Fix
          </Button>
          {#if cameraFixMsg}
            <div class="text-[10px] font-mono text-[#34d399] leading-snug">
              {cameraFixMsg}
            </div>
          {/if}
        </div>
      {/if}
    </div>

    <!-- Footer Controls -->
    <footer class="p-3 bg-[#151a26] border-t border-[#212938] flex items-center justify-between gap-2 shrink-0">
      <Button
        variant={selectedMod.enabled ? 'tonal' : 'filled'}
        size="sm"
        onclick={onToggleEquip}
      >
        {selectedMod.enabled ? 'Unequip Skin' : 'Equip this Skin'}
      </Button>

      {#if !selectedMod.isDefault && onDelete}
        <Button variant="destructive" size="sm" onclick={onDelete} title="Delete VPK file from addons">
          Delete VPK
        </Button>
      {/if}
    </footer>
  </aside>
{/if}
