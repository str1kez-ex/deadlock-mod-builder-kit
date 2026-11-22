<script>
  import { onMount, onDestroy } from 'svelte';
  import * as THREE from 'three';

  let {
    hero = 'Abrams',
    skinName = 'Default Hero Model',
  } = $props();

  let containerEl = $state(null);
  let isWireframe = $state(false);
  let isAutoRotating = $state(true);
  let isInteracting = $state(false);

  let renderer = null;
  let scene = null;
  let camera = null;
  let animFrameId = null;
  let modelGroup = null;
  let pedestalGroup = null;

  // Pointer interaction state
  let isPointerDown = false;
  let prevPointerX = 0;
  let prevPointerY = 0;
  let targetRotationY = 0;
  let targetRotationX = 0.1;
  let currentRotationY = 0;
  let currentRotationX = 0.1;
  let cameraDistance = 4.2;

  // Hero color accents mapping for stylized materials
  const heroColorMap = {
    'Abrams': 0x3b82f6,
    'Bebop': 0xeab308,
    'Calico': 0xec4899,
    'Dynamo': 0x06b6d4,
    'Grey Talon': 0x10b981,
    'Haze': 0x8b5cf6,
    'Infernus': 0xef4444,
    'Ivy': 0x84cc16,
    'Kelvin': 0x38bdf8,
    'Lady Geist': 0xa855f7,
    'Lash': 0xf97316,
    'McGinnis': 0xf59e0b,
    'Mirage': 0x14b8a6,
    'Mo & Krill': 0x64748b,
    'Paradox': 0x6366f1,
    'Pocket': 0xd946ef,
    'Seven': 0x0ea5e9,
    'Shiv': 0xdc2626,
    'Vindicta': 0x9333ea,
    'Viscous': 0x22c55e,
    'Warden': 0x2563eb,
    'Wraith': 0x7c3aed,
    'Yamato': 0xe11d48,
  };

  function getHeroColor(name) {
    return heroColorMap[name] || 0x4ade80;
  }

  function initThree() {
    if (!containerEl) return;

    const width = containerEl.clientWidth || 360;
    const height = containerEl.clientHeight || 480;

    // 1. Scene
    scene = new THREE.Scene();
    scene.background = new THREE.Color(0x0e1117);
    scene.fog = new THREE.FogExp2(0x0e1117, 0.15);

    // 2. Camera
    camera = new THREE.PerspectiveCamera(45, width / height, 0.1, 100);
    camera.position.set(0, 1.2, cameraDistance);

    // 3. Renderer
    renderer = new THREE.WebGLRenderer({ antialias: true, alpha: true, powerPreference: 'high-performance' });
    renderer.setSize(width, height);
    renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
    renderer.shadowMap.enabled = true;
    renderer.shadowMap.type = THREE.PCFSoftShadowMap;
    renderer.toneMapping = THREE.ACESFilmicToneMapping;
    renderer.toneMappingExposure = 1.1;

    containerEl.appendChild(renderer.domElement);

    // 4. Lights
    const ambientLight = new THREE.AmbientLight(0xffffff, 0.6);
    scene.add(ambientLight);

    const dirLight1 = new THREE.DirectionalLight(0xffffff, 1.8);
    dirLight1.position.set(3, 6, 4);
    dirLight1.castShadow = true;
    scene.add(dirLight1);

    const rimLight = new THREE.DirectionalLight(getHeroColor(hero), 2.2);
    rimLight.position.set(-4, 3, -4);
    scene.add(rimLight);

    // 5. Build Hero Stand & Stylized Mannequin Mesh
    buildSceneObjects();

    // 6. Start Loop
    animate();
  }

  function buildSceneObjects() {
    if (modelGroup) scene.remove(modelGroup);
    if (pedestalGroup) scene.remove(pedestalGroup);

    modelGroup = new THREE.Group();
    pedestalGroup = new THREE.Group();

    const heroColor = getHeroColor(hero);

    // Pedestal Base
    const baseGeo = new THREE.CylinderGeometry(1.4, 1.6, 0.15, 32);
    const baseMat = new THREE.MeshStandardMaterial({
      color: 0x141822,
      roughness: 0.7,
      metalness: 0.3,
      wireframe: isWireframe,
    });
    const baseMesh = new THREE.Mesh(baseGeo, baseMat);
    baseMesh.position.y = -0.075;
    baseMesh.receiveShadow = true;
    pedestalGroup.add(baseMesh);

    // Glowing Neon Ring on Pedestal
    const ringGeo = new THREE.TorusGeometry(1.42, 0.025, 16, 64);
    const ringMat = new THREE.MeshBasicMaterial({
      color: heroColor,
      wireframe: isWireframe,
    });
    const ringMesh = new THREE.Mesh(ringGeo, ringMat);
    ringMesh.rotation.x = Math.PI / 2;
    ringMesh.position.y = 0.01;
    pedestalGroup.add(ringMesh);

    // Pedestal Floor Grid
    const grid = new THREE.GridHelper(5, 10, heroColor, 0x1f2937);
    grid.position.y = -0.08;
    pedestalGroup.add(grid);

    scene.add(pedestalGroup);

    // Stylized Hero Anatomy Mesh (Source 2 / Valve Mannequin silhouette)
    const bodyMat = new THREE.MeshStandardMaterial({
      color: 0x222938,
      roughness: 0.45,
      metalness: 0.55,
      wireframe: isWireframe,
    });

    const armorMat = new THREE.MeshStandardMaterial({
      color: heroColor,
      roughness: 0.3,
      metalness: 0.8,
      wireframe: isWireframe,
    });

    // Torso / Chest
    const torsoGeo = new THREE.CylinderGeometry(0.38, 0.28, 0.8, 16);
    const torso = new THREE.Mesh(torsoGeo, bodyMat);
    torso.position.y = 1.35;
    torso.castShadow = true;
    modelGroup.add(torso);

    // Armor Chestplate
    const plateGeo = new THREE.BoxGeometry(0.48, 0.5, 0.32);
    const plate = new THREE.Mesh(plateGeo, armorMat);
    plate.position.set(0, 1.42, 0.05);
    plate.castShadow = true;
    modelGroup.add(plate);

    // Head / Helmet
    const headGeo = new THREE.SphereGeometry(0.22, 24, 24);
    const head = new THREE.Mesh(headGeo, bodyMat);
    head.position.y = 1.95;
    head.castShadow = true;
    modelGroup.add(head);

    // Visor
    const visorGeo = new THREE.BoxGeometry(0.28, 0.08, 0.2);
    const visorMat = new THREE.MeshBasicMaterial({ color: heroColor });
    const visor = new THREE.Mesh(visorGeo, visorMat);
    visor.position.set(0, 1.95, 0.12);
    modelGroup.add(visor);

    // Pelvis & Legs
    const pelvisGeo = new THREE.CylinderGeometry(0.28, 0.24, 0.25, 16);
    const pelvis = new THREE.Mesh(pelvisGeo, bodyMat);
    pelvis.position.y = 0.85;
    modelGroup.add(pelvis);

    // Left Leg
    const legL = new THREE.Mesh(new THREE.CylinderGeometry(0.12, 0.08, 0.85, 12), bodyMat);
    legL.position.set(-0.2, 0.42, 0);
    legL.castShadow = true;
    modelGroup.add(legL);

    // Right Leg
    const legR = new THREE.Mesh(new THREE.CylinderGeometry(0.12, 0.08, 0.85, 12), bodyMat);
    legR.position.set(0.2, 0.42, 0);
    legR.castShadow = true;
    modelGroup.add(legR);

    // Arms
    const armL = new THREE.Mesh(new THREE.CylinderGeometry(0.09, 0.07, 0.75, 12), bodyMat);
    armL.position.set(-0.48, 1.3, 0);
    armL.rotation.z = 0.2;
    armL.castShadow = true;
    modelGroup.add(armL);

    const armR = new THREE.Mesh(new THREE.CylinderGeometry(0.09, 0.07, 0.75, 12), bodyMat);
    armR.position.set(0.48, 1.3, 0);
    armR.rotation.z = -0.2;
    armR.castShadow = true;
    modelGroup.add(armR);

    // Shoulder Pauldrons
    const pauldronL = new THREE.Mesh(new THREE.SphereGeometry(0.16, 12, 12), armorMat);
    pauldronL.position.set(-0.48, 1.62, 0);
    modelGroup.add(pauldronL);

    const pauldronR = new THREE.Mesh(new THREE.SphereGeometry(0.16, 12, 12), armorMat);
    pauldronR.position.set(0.48, 1.62, 0);
    modelGroup.add(pauldronR);

    scene.add(modelGroup);
  }

  $effect(() => {
    // Re-build or update colors when hero or wireframe changes
    if (scene && modelGroup) {
      buildSceneObjects();
    }
  });

  function animate() {
    animFrameId = requestAnimationFrame(animate);

    if (isAutoRotating && !isPointerDown) {
      targetRotationY += 0.007;
    }

    // Smooth dampening
    currentRotationY += (targetRotationY - currentRotationY) * 0.1;
    currentRotationX += (targetRotationX - currentRotationX) * 0.1;

    if (modelGroup) {
      modelGroup.rotation.y = currentRotationY;
      modelGroup.rotation.x = currentRotationX;
    }
    if (pedestalGroup) {
      pedestalGroup.rotation.y = currentRotationY * 0.5;
    }

    // Update camera distance
    camera.position.z = cameraDistance;

    if (renderer && scene && camera) {
      renderer.render(scene, camera);
    }
  }

  // Pointer event listeners
  function onPointerDown(e) {
    isPointerDown = true;
    isInteracting = true;
    prevPointerX = e.clientX;
    prevPointerY = e.clientY;
  }

  function onPointerMove(e) {
    if (!isPointerDown) return;
    const deltaX = e.clientX - prevPointerX;
    const deltaY = e.clientY - prevPointerY;
    prevPointerX = e.clientX;
    prevPointerY = e.clientY;

    targetRotationY += deltaX * 0.012;
    targetRotationX = Math.max(-0.4, Math.min(0.6, targetRotationX + deltaY * 0.012));
  }

  function onPointerUp() {
    isPointerDown = false;
    setTimeout(() => {
      isInteracting = false;
    }, 2000);
  }

  function onWheel(e) {
    e.preventDefault();
    cameraDistance = Math.max(2.0, Math.min(7.0, cameraDistance + e.deltaY * 0.003));
  }

  function resetView() {
    targetRotationY = 0;
    targetRotationX = 0.1;
    cameraDistance = 4.2;
  }

  function handleResize() {
    if (!containerEl || !camera || !renderer) return;
    const width = containerEl.clientWidth;
    const height = containerEl.clientHeight;
    camera.aspect = width / height;
    camera.updateProjectionMatrix();
    renderer.setSize(width, height);
  }

  onMount(() => {
    initThree();
    window.addEventListener('resize', handleResize);
  });

  onDestroy(() => {
    window.removeEventListener('resize', handleResize);
    if (animFrameId) cancelAnimationFrame(animFrameId);
    if (renderer && renderer.domElement) {
      renderer.dispose();
      renderer.domElement.remove();
    }
  });
</script>

<div
  class="relative w-full h-full min-h-[460px] bg-[#0c0f16] border border-[#232d3e] rounded-[6px] overflow-hidden flex flex-col select-none"
>
  <!-- ThreeJS Canvas Container -->
  <div
    bind:this={containerEl}
    class="w-full flex-1 cursor-grab active:cursor-grabbing touch-none"
    onpointerdown={onPointerDown}
    onpointermove={onPointerMove}
    onpointerup={onPointerUp}
    onwheel={onWheel}
    role="region"
    aria-label="3D Model Viewer"
  ></div>

  <!-- Top Floating HUD Overlay -->
  <div class="absolute top-3 left-3 right-3 flex items-start justify-between pointer-events-none">
    <div class="px-3 py-1.5 bg-[#121622]/90 border border-[#293548] backdrop-blur-xs rounded-[4px] pointer-events-auto shadow-md">
      <div class="text-[10px] font-mono uppercase tracking-wider text-[#828fa3]">Model Inspector</div>
      <div class="text-sm font-semibold text-[#f1f5f9] leading-tight flex items-center gap-1.5">
        <span>{hero}</span>
        <span class="w-1.5 h-1.5 rounded-full" style="background-color: var(--accent-light);"></span>
      </div>
      <div class="text-[11px] font-mono text-[#cbd5e1] truncate max-w-[190px]">
        {skinName}
      </div>
    </div>

    <!-- Orbit Controls HUD Buttons -->
    <div class="flex items-center gap-1.5 pointer-events-auto bg-[#121622]/90 border border-[#293548] backdrop-blur-xs p-1 rounded-[4px] shadow-md">
      <button
        type="button"
        onclick={resetView}
        class="px-2 py-1 text-[10px] font-mono text-[#94a3b8] hover:text-[#f1f5f9] hover:bg-[#1f2838] rounded transition cursor-pointer"
        title="Reset camera angles and zoom"
      >
        Reset
      </button>

      <button
        type="button"
        onclick={() => (isWireframe = !isWireframe)}
        class="px-2 py-1 text-[10px] font-mono rounded transition cursor-pointer {isWireframe ? 'bg-[#2b394f] text-[#38bdf8]' : 'text-[#94a3b8] hover:text-[#f1f5f9] hover:bg-[#1f2838]'}"
        title="Toggle wireframe rendering"
      >
        Wireframe
      </button>

      <button
        type="button"
        onclick={() => (isAutoRotating = !isAutoRotating)}
        class="px-2 py-1 text-[10px] font-mono rounded transition cursor-pointer {isAutoRotating ? 'text-[#34d399] bg-[#132a1e]' : 'text-[#94a3b8] hover:text-[#f1f5f9] hover:bg-[#1f2838]'}"
        title="Toggle turntable auto-rotation"
      >
        {isAutoRotating ? 'Rotating' : 'Paused'}
      </button>
    </div>
  </div>

  <!-- Bottom Helper Prompt -->
  <div class="absolute bottom-2.5 left-3 right-3 flex items-center justify-between text-[10px] font-mono text-[#64748b] pointer-events-none">
    <span>Left Click + Drag: Rotate · Scroll: Zoom</span>
    <span class="text-[9px] uppercase tracking-wider text-[#475569]">Source 2 Model Rig</span>
  </div>
</div>
