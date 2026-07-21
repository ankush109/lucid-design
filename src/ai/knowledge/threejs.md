When the design calls for a signature 3D moment — a hero with an animated
gradient orb, a floating geometry, a particle field, a wireframe object —
reach for three.js. Guidance below.

CDN (ES module, always latest stable):
<script type="importmap">
{"imports": {"three": "https://cdn.jsdelivr.net/npm/three@0.170.0/build/three.module.js"}}
</script>
<script type="module">
  import * as THREE from "three";
  // ... your scene here ...
</script>

WHEN to use three.js:
- Exactly ONE moment per page. Never two 3D scenes.
- Decorative only — behind the hero, in an aside, as an ambient background.
- If the REFERENCE SITE uses three.js/WebGL (block will call this out), you
  MUST include a matching 3D element in the new design.
- If the subject has depth-suggestive semantics (space, physics, VR, 3D
  modelling tool, immersive audio), consider it even without reference cue.

WHEN NOT:
- Never for navigation, content, or anything a screen reader must access.
- Never in a bento cell — the tiling breaks the visual metaphor.
- Never over key text without an opacity < 0.5 layer between.

MINIMAL BOILERPLATE (adapt, don't copy verbatim):
<div id="scene" style="position:absolute;inset:0;pointer-events:none;z-index:0"></div>
<script type="module">
import * as THREE from "three";
const host = document.getElementById("scene");
const renderer = new THREE.WebGLRenderer({ alpha:true, antialias:true, powerPreference:"high-performance" });
renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
const resize = () => { renderer.setSize(host.clientWidth, host.clientHeight); camera.aspect = host.clientWidth/host.clientHeight; camera.updateProjectionMatrix(); };
host.appendChild(renderer.domElement);
const scene = new THREE.Scene();
const camera = new THREE.PerspectiveCamera(50, host.clientWidth/host.clientHeight, 0.1, 100);
camera.position.z = 4;
// ── your geometry / material here ──
window.addEventListener("resize", resize); resize();
const reduce = matchMedia("(prefers-reduced-motion: reduce)").matches;
const loop = (t) => { /* animate here — but skip transform updates if `reduce` */ renderer.render(scene, camera); requestAnimationFrame(loop); };
requestAnimationFrame(loop);
</script>

COMMON PATTERNS (write one, not all):
1. FLOATING GRADIENT ORB — icosahedron with MeshBasicMaterial + emissive
   glow, slow rotation + subtle scale pulse. Behind hero copy. Colours from
   the design's palette. Compose with a subtle blur backdrop-filter.
2. PARTICLE DRIFT — BufferGeometry with 800-1500 Points, additive blending,
   opacity 0.3-0.5. Wind-like drift on x-axis. Use for atmosphere, not focus.
3. WIREFRAME OBJECT — TorusKnot or Dodecahedron with a wireframe material
   whose line weight roughly matches the design's body stroke. Slow yaw
   rotation. Feels editorial + technical.
4. GRADIENT PLANE — ShaderMaterial on a plane, mixing two palette colours
   with slow noise. Reads as an animated background wash.

PERFORMANCE + RESPONSIBILITY:
- Cap devicePixelRatio at 2. Higher wastes battery.
- Pause `requestAnimationFrame` when `document.hidden` becomes true.
- Honour `prefers-reduced-motion` — hold the still frame, don't remove the visual entirely (the composition still needs it).
- Never depend on user interaction to render — the initial state must look intentional at t=0.
- The scene must survive removal — no critical content lives inside the WebGL canvas.
