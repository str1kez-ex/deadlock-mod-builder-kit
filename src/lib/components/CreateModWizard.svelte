<script>
  import { invoke } from '@tauri-apps/api/core';
  import Button from './Button.svelte';
  import Badge from './Badge.svelte';

  let {
    heroes = [],
    getHeroIcon = undefined,
    onModCreated = undefined,
  } = $props();

  let step = $state(1); // 1 to 5

  // Form State
  let projectName = $state('my_character');
  let selectedHero = $state('Abrams');
  let category = $state('Character Skin');
  let author = $state('str1kez');

  let modelFile = $state('');
  let gunFile = $state('');

  let textureDiffuse = $state('');
  let textureNormal = $state('');
  let audioUlt = $state('');
  let audioAbility1 = $state('');

  let fusionVpk = $state('');
  let applyCameraFix = $state(true);

  let isBuilding = $state(false);
  let buildResult = $state(null);
  let buildError = $state('');

  async function handleBuild() {
    isBuilding = true;
    buildError = '';
    buildResult = null;
    try {
      const res = await invoke('create_custom_mod', {
        name: projectName,
        heroName: selectedHero,
        category,
        author: author || 'str1kez',
        applyCameraFix,
      });
      buildResult = {
        name: projectName,
        hero: selectedHero,
        addons: res,
      };
      onModCreated?.();
    } catch (e) {
      buildError = String(e);
    } finally {
      isBuilding = false;
    }
  }

  function resetForm() {
    step = 1;
    projectName = 'my_character';
    selectedHero = 'Abrams';
    modelFile = '';
    gunFile = '';
    buildResult = null;
    buildError = '';
  }
</script>

<div class="max-w-4xl mx-auto p-6 bg-[#131720] border border-[#232d3d] rounded-[8px] space-y-6 text-[#f1f5f9] select-none shadow-xl">
  <!-- Stepper Breadcrumb / Progress Bar -->
  <div class="flex items-center justify-between border-b border-[#212938] pb-4">
    <div>
      <div class="text-[10px] font-mono uppercase tracking-wider text-[#828fa3]">
        STEP {step} OF 5
      </div>
      <h2 class="text-base font-bold text-[#f1f5f9] leading-tight">
        {#if step === 1}Character & Project
        {:else if step === 2}3D Model & Attachments
        {:else if step === 3}Textures & Audio Replacements
        {:else if step === 4}VPK Fusion & Camera Calibration
        {:else}Build, Pack & Deploy to Addons
        {/if}
      </h2>
    </div>

    <!-- Step Pills -->
    <div class="flex items-center gap-1.5">
      {#each [1, 2, 3, 4, 5] as s}
        <button
          type="button"
          onclick={() => (step = s)}
          style={step === s ? 'background-color: var(--accent-primary); color: var(--accent-on-primary);' : undefined}
          class="w-7 h-7 rounded-[4px] text-xs font-mono font-bold flex items-center justify-center transition cursor-pointer
            {step === s
              ? 'shadow-xs'
              : step > s
              ? 'bg-[#1e2738] text-[var(--accent-light)] border border-[#2e3b52]'
              : 'bg-[#101319] text-[#55637a] border border-[#1f2735]'}"
        >
          {s}
        </button>
      {/each}
    </div>
  </div>

  <!-- STEP 1: CHARACTER & PROJECT -->
  {#if step === 1}
    <div class="space-y-5">
      <!-- Project Name -->
      <div class="space-y-1.5">
        <label for="project-name-input" class="text-xs font-semibold text-[#cbd5e1]">Project Name</label>
        <input
          id="project-name-input"
          type="text"
          bind:value={projectName}
          class="h-9 w-full max-w-md px-3 bg-[#0d1016] text-xs font-mono text-[#f1f5f9] rounded border border-[#263143] focus:border-[var(--accent-primary)] transition"
          placeholder="e.g. ratking_cyberpunk_overhaul"
        />
        <p class="text-[11px] text-[#828fa3]">Up to 41 letters, numbers, spaces or underscores.</p>
      </div>

      <!-- Mod Category & Author -->
      <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
        <div class="space-y-1.5">
          <label for="cat-select" class="text-xs font-semibold text-[#cbd5e1]">Category</label>
          <select
            id="cat-select"
            bind:value={category}
            class="h-9 w-full px-3 bg-[#0d1016] text-xs text-[#f1f5f9] rounded border border-[#263143] focus:border-[var(--accent-primary)] cursor-pointer"
          >
            <option value="Character Skin">Character Model & Skin</option>
            <option value="Audio / Voicepack">Sound & Voicepack</option>
            <option value="Weapon Mod">Weapon Attachment</option>
            <option value="UI & HUD">UI / HUD Crosshair</option>
            <option value="Hybrid Fusion">Enhanced VPK Fusion</option>
          </select>
        </div>

        <div class="space-y-1.5">
          <label for="author-input" class="text-xs font-semibold text-[#cbd5e1]">Author Tag</label>
          <input
            id="author-input"
            type="text"
            bind:value={author}
            class="h-9 w-full px-3 bg-[#0d1016] text-xs font-mono text-[#f1f5f9] rounded border border-[#263143] focus:border-[var(--accent-primary)] transition"
          />
        </div>
      </div>

      <!-- Hero Selector with User Icons -->
      <div class="space-y-2 pt-2">
        <div class="flex items-center justify-between">
          <span class="text-xs font-semibold text-[#cbd5e1]">Target Deadlock Hero</span>
          <span class="text-xs font-mono" style="color: var(--accent-light);">Selected: {selectedHero}</span>
        </div>

        <div class="grid grid-cols-3 sm:grid-cols-6 md:grid-cols-8 gap-2 max-h-56 overflow-y-auto p-2 bg-[#0c0e14] border border-[#1f2838] rounded-[6px]">
          {#each heroes as h}
            {@const icon = getHeroIcon ? getHeroIcon(h) : ''}
            <button
              type="button"
              onclick={() => (selectedHero = h)}
              style={selectedHero === h ? 'border-color: var(--accent-primary); background-color: var(--accent-active-sidebar);' : undefined}
              class="p-2 rounded-[5px] border flex flex-col items-center justify-center gap-1.5 transition cursor-pointer
                {selectedHero === h
                  ? 'border-2 shadow-xs'
                  : 'bg-[#131722] border-[#222a3a] hover:border-[#33425b]'}"
            >
              {#if icon}
                <img src={icon} alt={h} class="w-8 h-8 object-contain" />
              {:else}
                <div class="w-8 h-8 rounded bg-[#1e2535] text-xs font-bold flex items-center justify-center">{h[0]}</div>
              {/if}
              <span class="text-[10px] font-medium truncate max-w-[70px] text-center">{h}</span>
            </button>
          {/each}
        </div>
      </div>
    </div>

  <!-- STEP 2: 3D MODEL & WEAPONS -->
  {:else if step === 2}
    <div class="space-y-5">
      <div class="p-4 bg-[#0d1016] border border-[#1f2838] rounded-[6px] space-y-3">
        <div class="text-xs font-bold text-[#f1f5f9]">Base 3D Model Replacement</div>
        <p class="text-xs text-[#828fa3]">
          Upload or specify your custom rigged 3D character model. Supported formats: FBX, GLB, glTF, OBJ, SMD, DMX or existing VPK to remix.
        </p>

        <div class="space-y-2">
          <label for="wizard-model-file" class="text-[11px] font-mono text-[#94a3b8]">Model File Path / Name</label>
          <input
            id="wizard-model-file"
            type="text"
            bind:value={modelFile}
            placeholder="C:\Mods\models\custom_hero.fbx"
            class="h-9 w-full px-3 bg-[#151924] text-xs font-mono text-[#f1f5f9] rounded border border-[#2a364a] focus:border-[var(--accent-primary)]"
          />
        </div>
      </div>

      <div class="p-4 bg-[#0d1016] border border-[#1f2838] rounded-[6px] space-y-3">
        <div class="text-xs font-bold text-[#f1f5f9]">Gun / Weapon Model Attachment (Optional)</div>
        <p class="text-xs text-[#828fa3]">
          Optional weapon replacement file. If your hero model uses dual weapons, both hands are rigged automatically.
        </p>

        <div class="space-y-2">
          <label for="wizard-gun-file" class="text-[11px] font-mono text-[#94a3b8]">Weapon Asset Path</label>
          <input
            id="wizard-gun-file"
            type="text"
            bind:value={gunFile}
            placeholder="C:\Mods\weapons\custom_gun.glb"
            class="h-9 w-full px-3 bg-[#151924] text-xs font-mono text-[#f1f5f9] rounded border border-[#2a364a] focus:border-[var(--accent-primary)]"
          />
        </div>
      </div>
    </div>

  <!-- STEP 3: TEXTURES & AUDIO REPLACEMENTS -->
  {:else if step === 3}
    <div class="space-y-5">
      <!-- Material Slots -->
      <div class="p-4 bg-[#0d1016] border border-[#1f2838] rounded-[6px] space-y-3">
        <div class="text-xs font-bold text-[#f1f5f9]">Source 2 PBR Textures</div>
        <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
          <div class="space-y-1">
            <span class="text-[11px] font-mono text-[#828fa3]">Diffuse / Albedo Map (PNG / TGA)</span>
            <input
              type="text"
              bind:value={textureDiffuse}
              placeholder="hero_color.png"
              class="h-8 w-full px-2.5 bg-[#151924] text-xs font-mono text-[#f1f5f9] rounded border border-[#2a364a]"
            />
          </div>
          <div class="space-y-1">
            <span class="text-[11px] font-mono text-[#828fa3]">Normal Map (OpenGL / DirectX)</span>
            <input
              type="text"
              bind:value={textureNormal}
              placeholder="hero_normal.png"
              class="h-8 w-full px-2.5 bg-[#151924] text-xs font-mono text-[#f1f5f9] rounded border border-[#2a364a]"
            />
          </div>
        </div>
      </div>

      <!-- Audio Slots -->
      <div class="p-4 bg-[#0d1016] border border-[#1f2838] rounded-[6px] space-y-3">
        <div class="text-xs font-bold text-[#f1f5f9]">Sound & Ability Audio Replacements (WAV / MP3)</div>
        <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
          <div class="space-y-1">
            <span class="text-[11px] font-mono text-[#828fa3]">Ultimate Cast Voice / SFX</span>
            <input
              type="text"
              bind:value={audioUlt}
              placeholder="ult_cast.wav"
              class="h-8 w-full px-2.5 bg-[#151924] text-xs font-mono text-[#f1f5f9] rounded border border-[#2a364a]"
            />
          </div>
          <div class="space-y-1">
            <span class="text-[11px] font-mono text-[#828fa3]">Primary Ability SFX</span>
            <input
              type="text"
              bind:value={audioAbility1}
              placeholder="ability1_fire.wav"
              class="h-8 w-full px-2.5 bg-[#151924] text-xs font-mono text-[#f1f5f9] rounded border border-[#2a364a]"
            />
          </div>
        </div>
      </div>
    </div>

  <!-- STEP 4: VPK FUSION & CALIBRATION -->
  {:else if step === 4}
    <div class="space-y-5">
      <!-- VPK Fusion Feature -->
      <div class="p-4 bg-[#0d1016] border border-[#1f2838] rounded-[6px] space-y-3">
        <div class="flex items-center gap-2 text-xs font-bold text-[#f1f5f9]">
          <span style="color: var(--accent-light);">⚡</span>
          <span>VPK Enhanced Fusion (Asset Remixing)</span>
        </div>
        <p class="text-xs text-[#828fa3]">
          Borrow assets from an existing VPK package! For example, inject custom ultimate sounds from another mod into this model.
        </p>

        <div class="space-y-1.5">
          <label for="wizard-fusion-vpk" class="text-[11px] font-mono text-[#94a3b8]">External Source VPK (Optional)</label>
          <input
            id="wizard-fusion-vpk"
            type="text"
            bind:value={fusionVpk}
            placeholder="e.g. ratking_custom_sounds.vpk"
            class="h-9 w-full px-3 bg-[#151924] text-xs font-mono text-[#f1f5f9] rounded border border-[#2a364a] focus:border-[var(--accent-primary)]"
          />
        </div>
      </div>

      <!-- Camera Calibration Notice -->
      <div class="p-4 bg-[#0d1016] border border-[#1f2838] rounded-[6px] space-y-3">
        <div class="flex items-center justify-between">
          <div>
            <div class="text-xs font-bold text-[#f1f5f9]">Violet Engine Camera Auto-Calibration</div>
            <p class="text-[11px] text-[#828fa3]">
              Automatically embeds third-person shoulder offset cvars to prevent broken perspectives after the Violet update.
            </p>
          </div>
          <input
            type="checkbox"
            bind:checked={applyCameraFix}
            style="accent-color: var(--accent-primary);"
            class="w-4 h-4 cursor-pointer"
          />
        </div>
      </div>
    </div>

  <!-- STEP 5: BUILD, PACK & DEPLOY -->
  {:else}
    <div class="space-y-5">
      <div class="p-4 bg-[#0d1016] border border-[#1f2838] rounded-[6px] space-y-3">
        <h3 class="text-xs font-bold text-[#f1f5f9]">Mod Build Summary</h3>
        <div class="grid grid-cols-2 gap-3 text-xs font-mono">
          <div class="p-2 bg-[#141924] rounded border border-[#232c3d]">
            <span class="text-[#828fa3]">Project:</span>
            <div class="text-[#f1f5f9] font-bold mt-0.5">{projectName}</div>
          </div>
          <div class="p-2 bg-[#141924] rounded border border-[#232c3d]">
            <span class="text-[#828fa3]">Target Hero:</span>
            <div class="text-[#f1f5f9] font-bold mt-0.5">{selectedHero}</div>
          </div>
          <div class="p-2 bg-[#141924] rounded border border-[#232c3d]">
            <span class="text-[#828fa3]">Category:</span>
            <div class="text-[#f1f5f9] font-bold mt-0.5">{category}</div>
          </div>
          <div class="p-2 bg-[#141924] rounded border border-[#232c3d]">
            <span class="text-[#828fa3]">Camera Fix:</span>
            <div class="text-[#34d399] font-bold mt-0.5">{applyCameraFix ? 'Calibrated' : 'Off'}</div>
          </div>
        </div>
      </div>

      {#if isBuilding}
        <div class="p-5 bg-[#171e2b] border border-[#28364b] rounded-[6px] flex items-center justify-center gap-3">
          <div style="border-top-color: var(--accent-primary);" class="w-5 h-5 rounded-full border-2 border-white/20 animate-spin"></div>
          <span class="text-xs font-mono text-[#cbd5e1]">Compiling Valve v2 VPK package & deploying to citadel/addons…</span>
        </div>
      {:else if buildResult}
        <div class="p-4 bg-[#0d2216] border border-[#10b981]/40 rounded-[6px] space-y-2">
          <div class="flex items-center gap-2 text-xs font-bold text-[#34d399]">
            <svg class="w-4 h-4 stroke-current fill-none stroke-[2]" viewBox="0 0 24 24">
              <polyline points="20 6 9 17 4 12" />
            </svg>
            <span>Mod Successfully Built & Installed!</span>
          </div>
          <p class="text-xs text-[#a7f3d0]">
            Packaged as <code class="font-mono font-bold">{buildResult.name}.vpk</code> and deployed into <code class="font-mono">game/citadel/addons/</code>. It is now active under Characters loadout.
          </p>
          <div class="pt-2">
            <Button variant="filled" size="sm" onclick={resetForm}>
              Create Another Mod
            </Button>
          </div>
        </div>
      {:else if buildError}
        <div class="p-4 bg-[#331114] border border-[#ef4444]/40 rounded-[6px] text-xs font-mono text-[#fca5a5]">
          Build failed: {buildError}
        </div>
      {/if}

      {#if !buildResult}
        <div class="pt-2">
          <Button
            variant="filled"
            size="md"
            isFullWidth={true}
            disabled={isBuilding || !projectName.trim()}
            onclick={handleBuild}
          >
            Build & Install Mod Package (.vpk)
          </Button>
        </div>
      {/if}
    </div>
  {/if}

  <!-- Footer Navigation Buttons -->
  <div class="flex items-center justify-between border-t border-[#212938] pt-4">
    <Button
      variant="tonal"
      size="sm"
      disabled={step === 1 || isBuilding}
      onclick={() => (step = Math.max(1, step - 1))}
    >
      ← Previous Step
    </Button>

    <Button
      variant="filled"
      size="sm"
      disabled={step === 5 || isBuilding}
      onclick={() => (step = Math.min(5, step + 1))}
    >
      Next Step →
    </Button>
  </div>
</div>
