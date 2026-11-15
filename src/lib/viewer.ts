import * as THREE from "three";
import { GLTFLoader } from "three/addons/loaders/GLTFLoader.js";
import { OrbitControls } from "three/addons/controls/OrbitControls.js";
import { TransformControls } from "three/addons/controls/TransformControls.js";
import { FIGURE, mid, type PoseFrame } from "./frames.ts";
import type { Channel, Joint } from "./rpc";
import type { Rig } from "./retarget";

type Rest = [THREE.Vector3, THREE.Quaternion, THREE.Vector3];

/** The key times of the animation shown: all of them, and the picked joint's. */
export type Keys = { all: number[]; joint: number[] };

// the game's colors are computed and shown as they are (no linear light): textures, vertex colors
// and the screen in the same space, as the PS2 and OpenGOAL's merc shaders do
THREE.ColorManagement.enabled = false;

const PROPERTY = {
  translation: "position",
  rotation: "quaternion",
  scale: "scale",
} as const;

/** Weights on joints the skin does not have (some extracted models carry them) moved off, so that
 * three.js does not fail on them; the game check reports them. */
function dropMissingJoints(mesh: THREE.SkinnedMesh) {
  const joints = mesh.skeleton.bones.length;
  const index = mesh.geometry.getAttribute("skinIndex");
  const weight = mesh.geometry.getAttribute("skinWeight");
  if (!index || !weight) return;
  for (let v = 0; v < index.count; v++) {
    let dropped = false;
    let sum = 0;
    for (let k = 0; k < 4; k++) {
      if (index.getComponent(v, k) >= joints) {
        index.setComponent(v, k, 0);
        weight.setComponent(v, k, 0);
        dropped = true;
      }
      sum += weight.getComponent(v, k);
    }
    if (!dropped) continue;
    if (sum === 0) weight.setComponent(v, 0, 1);
    else
      for (let k = 0; k < 4; k++)
        weight.setComponent(v, k, weight.getComponent(v, k) / sum);
  }
}

export class Viewer {
  private renderer: THREE.WebGLRenderer;
  private scene = new THREE.Scene();
  private camera = new THREE.PerspectiveCamera(40, 1, 0.01, 1000);
  private controls: OrbitControls;
  private timer = new THREE.Timer();
  private observer: ResizeObserver;
  private mixer?: THREE.AnimationMixer;
  private action?: THREE.AnimationAction;
  private model?: THREE.Object3D;
  private helper?: THREE.SkeletonHelper;
  private rest = new Map<THREE.Object3D, Rest>();
  private figure?: {
    lines: THREE.LineSegments;
    frames: PoseFrame[];
    fps: number;
    at: (p: THREE.Vector3) => THREE.Vector3;
  };
  /** each primitive of the skinned meshes, "mesh:primitive" as rpc's Part numbers them */
  private pieces: { object: THREE.Mesh; key: string }[] = [];
  /** the look shown: blend weights kept over what the animations set */
  private weights: number[] = [];
  /** the models carried by joints: Light Jak's wings, the Morph Gun */
  private carried = new Map<string, THREE.Object3D>();
  /** the model's materials as the game draws them, with their own texture and color */
  private materials: {
    material: THREE.MeshLambertMaterial;
    map: THREE.Texture | null;
    color: THREE.Color;
  }[] = [];
  /** textures put in place of a material's own, by its name without "-envmap" */
  private swaps = new Map<string, THREE.Texture>();
  /** Light Jak: the model untextured, lit by the rim lights */
  private lit = false;
  /** scales the game puts on joints every frame, over the animation's own (joint number, -1 for
   * the whole model): only what is drawn has them */
  private jointScales: [number, THREE.Vector3][] = [];
  private lights = new THREE.Group();
  private rim = new THREE.Group();
  /** the editor's: a dot on each joint to pick it, the picked one's handles */
  private handles?: THREE.InstancedMesh;
  private transform?: TransformControls;
  private down?: { x: number; y: number };
  /** what the camera frames: the model's joints, with a margin */
  private box = new THREE.Box3(
    new THREE.Vector3(-1, 0, -1),
    new THREE.Vector3(1, 2, 1),
  );

  joints: THREE.Object3D[] = [];
  clips: THREE.AnimationClip[] = [];
  paused = false;
  speed = 1;
  skeleton = false;
  /** the joint picked in the editor */
  selected: number | null = null;
  /** called every frame with the playing animation's time and length */
  onTime?: (time: number, duration: number) => void;
  /** called every frame before drawing, with the playing animation's name, time and length (the
   * eyes follow the animation's eye frames) */
  onFrame?: (clip: string | undefined, time: number, duration: number) => void;
  /** called when the animation shown or the picked joint changes */
  onKeys?: (keys: Keys) => void;
  /** a joint's dot clicked */
  onPick?: (joint: number) => void;
  /** the picked joint turned or moved with the mouse, once the button is released */
  onJointMoved?: (joint: number) => void;

  constructor(private canvas: HTMLCanvasElement) {
    this.renderer = new THREE.WebGLRenderer({ canvas, antialias: true });
    this.renderer.setPixelRatio(window.devicePixelRatio);
    this.renderer.outputColorSpace = THREE.LinearSRGBColorSpace;
    this.scene.background = new THREE.Color(0x1b1b1b);
    // three's lambert divides by pi: these are the game's light values
    // (goal_src/<game>/engine/gfx/mood: an ambient and directional lights)
    const light = (
      color: THREE.ColorRepresentation,
      intensity: number,
      x = 0,
      y = 0,
      z = 0,
    ) => {
      const l = new THREE.DirectionalLight(color, intensity * Math.PI);
      l.position.set(x, y, z);
      return l;
    };
    this.lights.add(new THREE.AmbientLight(0xffffff, 0.45 * Math.PI));
    this.lights.add(light(0xffffff, 0.8, 3, 6, 4));
    this.lights.add(light(0xffffff, 0.3, -4, 2, -3));
    this.scene.add(this.lights);
    // Light Jak's rim lights (time-of-day.gc, drawable.gc): three cyan lights from behind the
    // model as the camera sees it, and a blue ambient; they turn with the camera
    // the game's colors are linear: as hex, three.js would read them as sRGB and darken them (the
    // ambient 0.3 blue would light 0.075)
    const linear = (r: number, g: number, b: number) =>
      new THREE.Color().setRGB(r, g, b, THREE.LinearSRGBColorSpace);
    this.rim.add(new THREE.AmbientLight(linear(0, 0, 0.3), Math.PI));
    for (const [x, y, z] of [
      [0.7746, 0.4473, 0.4473],
      [-0.7746, 0.4473, 0.4473],
      [0, -0.8944, 0.4472],
    ]) {
      const l = light(linear(0, 0.95, 1), 2, x, y, -z - 1);
      l.target.position.set(0, 0, -1);
      this.rim.add(l, l.target);
    }
    this.rim.visible = false;
    this.camera.add(this.rim);
    this.scene.add(this.camera);
    this.scene.add(new THREE.GridHelper(20, 40, 0x52525b, 0x27272a));
    this.controls = new OrbitControls(this.camera, canvas);
    this.controls.enableDamping = true;
    this.camera.position.set(0, 1.5, 5);
    this.observer = new ResizeObserver(() => this.resize());
    this.observer.observe(canvas.parentElement ?? canvas);
    this.renderer.setAnimationLoop((t) => this.tick(t));
    // a click (not a drag of the view) on a joint's dot picks it
    canvas.addEventListener("pointerdown", (e) => {
      this.down = { x: e.clientX, y: e.clientY };
    });
    canvas.addEventListener("pointerup", (e) => {
      const down = this.down;
      this.down = undefined;
      if (
        down &&
        this.handles &&
        !this.transform?.axis &&
        Math.hypot(e.clientX - down.x, e.clientY - down.y) < 4
      )
        this.pick(e);
    });
  }

  private resize() {
    const box = (
      this.canvas.parentElement ?? this.canvas
    ).getBoundingClientRect();
    if (!box.width || !box.height) return;
    this.renderer.setSize(box.width, box.height, false);
    this.camera.aspect = box.width / box.height;
    this.camera.updateProjectionMatrix();
  }

  private tick(t: number) {
    this.timer.update(t);
    // hidden (another page shown): nothing to draw, the animation waits
    if (!this.canvas.offsetParent) return;
    if (this.mixer && !this.paused && !this.transform?.dragging)
      this.mixer.update(this.timer.getDelta() * this.speed);
    if (this.helper) this.helper.visible = this.skeleton;
    this.applyWeights(false);
    this.updateFigure();
    this.updateHandles();
    this.controls.update();
    this.onFrame?.(this.action?.getClip().name, this.time, this.duration);
    this.followAlign();
    const scaled = this.jointScales.flatMap(([j, s]) => {
      const o = j < 0 ? this.model : this.joints[j];
      if (!o) return [];
      const was = o.scale.clone();
      o.scale.multiply(s);
      return [[o, was] as const];
    });
    this.renderer.render(this.scene, this.camera);
    for (const [o, was] of scaled) o.scale.copy(was);
    this.onTime?.(this.time, this.duration);
  }

  /** The entity's travel: the game moves the whole character by its `align` joint (align-control),
   * which no other joint hangs from in the extracted models, so the model is moved by align's
   * offset from its rest position, back to the start when the animation loops. */
  private followAlign() {
    const align = this.joints[0];
    const rest = align && this.rest.get(align);
    if (!this.model || !rest || align.name !== "align") return;
    if (this.joints.some((j) => j.parent === align)) return;
    this.model.position.copy(align.position).sub(rest[0]);
  }

  get time() {
    // a clip of no length loops into NaN
    return this.duration ? (this.action?.time ?? 0) : 0;
  }

  /** the playing animation's name */
  get clipName() {
    return this.action?.getClip().name;
  }

  get duration() {
    return this.action?.getClip().duration ?? 0;
  }

  seek(time: number) {
    if (!this.action || !this.mixer) return;
    this.action.time = time;
    this.mixer.update(0);
  }

  async load(bytes: ArrayBuffer, joints: Joint[]) {
    const gltf = await new GLTFLoader().parseAsync(bytes, "");
    this.unload();
    const byNode = new Map<number, THREE.Object3D>();
    gltf.scene.traverse((o) => {
      const node = gltf.parser.associations.get(o)?.nodes;
      if (node !== undefined) byNode.set(node, o);
    });
    this.joints = joints.map((j) => {
      const o = byNode.get(j.node);
      if (!o) throw new Error(`joint ${j.name} is not in the loaded scene`);
      return o;
    });
    for (const j of this.joints)
      this.rest.set(j, [
        j.position.clone(),
        j.quaternion.clone(),
        j.scale.clone(),
      ]);
    gltf.scene.traverse((o) => {
      if ((o as THREE.SkinnedMesh).isSkinnedMesh)
        dropMissingJoints(o as THREE.SkinnedMesh);
      // the loader records the primitive too, which its types do not say
      const at = gltf.parser.associations.get(o) as
        { meshes?: number; primitives?: number } | undefined;
      if ((o as THREE.Mesh).isMesh && at?.meshes !== undefined)
        this.pieces.push({
          object: o as THREE.Mesh,
          key: `${at.meshes}:${at.primitives ?? 0}`,
        });
    });
    this.materials = this.toGame(gltf.scene);
    this.applyMaps();
    this.model = gltf.scene;
    this.clips = gltf.animations;
    this.scene.add(gltf.scene);
    this.mixer = new THREE.AnimationMixer(gltf.scene);
    this.helper = new THREE.SkeletonHelper(gltf.scene);
    (this.helper.material as THREE.LineBasicMaterial).depthTest = false;
    this.helper.renderOrder = 1;
    this.scene.add(this.helper);

    // framed on the joints: the bind-pose geometry of extracted models can lie far off
    gltf.scene.updateMatrixWorld(true);
    const box = new THREE.Box3().setFromPoints(
      this.joints.map((j) => j.getWorldPosition(new THREE.Vector3())),
    );
    box.expandByScalar(box.getSize(new THREE.Vector3()).length() * 0.12 || 0.5);
    this.box = box;
    this.frame(box);
  }

  /** The camera looking at `box` from the front, a little from the right. */
  private frame(box: THREE.Box3) {
    const size = box.getSize(new THREE.Vector3());
    const center = box.getCenter(new THREE.Vector3());
    const far = Math.max(size.x, size.y, size.z) * 2.2 || 3;
    this.camera.position.set(
      center.x + far * 0.3,
      center.y + size.y * 0.15,
      center.z + far,
    );
    this.controls.target.copy(center);
  }

  unload() {
    this.setEditing(false);
    this.swaps.clear();
    this.jointScales = [];
    this.lit = false;
    this.rim.visible = false;
    this.lights.visible = true;
    for (const slot of [...this.carried.keys()]) this.setCarried(slot, null);
    this.pieces = [];
    this.weights = [];
    this.mixer?.stopAllAction();
    this.action = undefined;
    if (this.helper) {
      this.scene.remove(this.helper);
      this.helper.dispose();
    }
    if (this.model) {
      this.scene.remove(this.model);
      this.model.traverse((o) => {
        const mesh = o as THREE.Mesh;
        mesh.geometry?.dispose();
        for (const m of [mesh.material ?? []].flat()) m.dispose();
      });
    }
    this.rest.clear();
    this.setFigure(null);
  }

  private toRest() {
    this.mixer?.stopAllAction();
    for (const [j, [p, q, s]] of this.rest) {
      j.position.copy(p);
      j.quaternion.copy(q);
      j.scale.copy(s);
    }
  }

  play(clip: THREE.AnimationClip | null) {
    this.toRest();
    this.action = undefined;
    if (clip && this.mixer) {
      this.action = this.mixer.clipAction(clip);
      this.action.reset().play();
      this.paused = false;
    }
    this.onKeys?.(this.keyTimes());
  }

  /** `clip` at `time`, playing or paused as before (the editor's animation, rebuilt at each key). */
  show(clip: THREE.AnimationClip, time: number) {
    const old = this.action?.getClip();
    this.toRest();
    if (old && old !== clip && !this.clips.includes(old))
      this.mixer?.uncacheClip(old);
    if (!this.mixer) return;
    this.action = this.mixer.clipAction(clip);
    this.action.play();
    this.action.time = time;
    this.mixer.update(0);
    this.onKeys?.(this.keyTimes());
  }

  keyTimes(): Keys {
    const clip = this.action?.getClip();
    const picked = this.selected === null ? null : this.joints[this.selected];
    const all = new Set<number>();
    const joint = new Set<number>();
    for (const track of clip?.tracks ?? []) {
      const node = THREE.PropertyBinding.parseTrackName(track.name).nodeName;
      const mine = !!picked && (node === picked.uuid || node === picked.name);
      for (const t of track.times) {
        const ms = Math.round(t * 1000) / 1000;
        all.add(ms);
        if (mine) joint.add(ms);
      }
    }
    const sorted = (s: Set<number>) => [...s].sort((a, b) => a - b);
    return { all: sorted(all), joint: sorted(joint) };
  }

  /** An animation of the joints from their keys, to show. */
  clipOf(channels: Channel[], duration: number, name = "edit") {
    const tracks = channels.map((c) => {
      const track = `${this.joints[c.joint].uuid}.${PROPERTY[c.path]}`;
      return c.path === "rotation"
        ? new THREE.QuaternionKeyframeTrack(track, c.times, c.values)
        : new THREE.VectorKeyframeTrack(track, c.times, c.values);
    });
    return new THREE.AnimationClip(name, duration, tracks);
  }

  /** The keys of one of the model's animations, joint by joint. */
  channelsOf(clip: THREE.AnimationClip): Channel[] {
    const out: Channel[] = [];
    for (const track of clip.tracks) {
      const { nodeName, propertyName } = THREE.PropertyBinding.parseTrackName(
        track.name,
      );
      const joint = this.joints.findIndex(
        (j) => j.uuid === nodeName || j.name === nodeName,
      );
      const path = (Object.keys(PROPERTY) as (keyof typeof PROPERTY)[]).find(
        (p) => PROPERTY[p] === propertyName,
      );
      // cubic spline keys hold tangents too: not keys the editor can move
      if (
        joint < 0 ||
        !path ||
        track.values.length !== track.times.length * track.getValueSize()
      )
        continue;
      out.push({
        joint,
        path,
        times: [...track.times],
        values: [...track.values],
      });
    }
    return out;
  }

  /** The look: the `hidden` primitives ("mesh:primitive") not drawn, the blend shapes at `weights`. */
  setLook(hidden: Set<string>, weights: number[]) {
    for (const { object, key } of this.pieces)
      object.visible = !hidden.has(key);
    this.weights = weights;
    this.applyWeights(true);
  }

  /** the blend weights on every primitive; `all` false keeps the animations' own where the look has none */
  private applyWeights(all: boolean) {
    for (const { object } of this.pieces) {
      const influences = object.morphTargetInfluences;
      if (!influences) continue;
      this.weights.forEach((w, t) => {
        if (t < influences.length && (all || w)) influences[t] = w;
      });
    }
  }

  /** A model carried by one of the joints in `slot` (Light Jak's wings on his chest, the Morph Gun
   * in his hand), or none: its `anchor` joint (else its origin) on `joint`, without `joint`'s
   * scale; `glow`, a model the game does not light; `pose`, the animation whose first keys it takes
   * (the gun's gun-idle-red folds the barrels of the other mods away). */
  async setCarried(
    slot: string,
    bytes: ArrayBuffer | null,
    joint = "",
    anchor = "",
    glow = false,
    pose = "",
  ) {
    const old = this.carried.get(slot);
    if (old) {
      old.removeFromParent();
      old.traverse((o) => {
        const mesh = o as THREE.Mesh;
        mesh.geometry?.dispose();
        for (const m of [mesh.material ?? []].flat()) m.dispose();
      });
      this.carried.delete(slot);
    }
    const at = this.joints.find((j) => j.name === joint);
    if (!bytes || !at) return;
    const gltf = await new GLTFLoader().parseAsync(bytes, "");
    gltf.scene.traverse((o) => {
      if ((o as THREE.SkinnedMesh).isSkinnedMesh)
        dropMissingJoints(o as THREE.SkinnedMesh);
    });
    // In game (lightjak-wings.gc) the wings' root follows the joint without its scale, and their own
    // animations (open, flap, none of them extracted) bring their main joint there. ponytail: at
    // rest, the anchor is put on the joint instead; the wings' animations if they are ever extracted.
    // The gun's root is the joint's matrix, scaled as Jak (target-gun.gc, target-gun-compute-pos).
    const posed = gltf.animations.find((c) => c.name === pose);
    if (posed) {
      const mixer = new THREE.AnimationMixer(gltf.scene);
      mixer.clipAction(posed).play();
      mixer.setTime(0);
    }
    gltf.scene.updateMatrixWorld(true);
    const anchored = gltf.scene.getObjectByName(anchor);
    const scale = 1 / at.getWorldScale(new THREE.Vector3()).x;
    if (anchored)
      gltf.scene.position
        .copy(anchored.getWorldPosition(new THREE.Vector3()))
        .multiplyScalar(-scale);
    gltf.scene.scale.setScalar(scale);
    this.toGame(gltf.scene, glow);
    this.carried.set(slot, gltf.scene);
    at.add(gltf.scene);
  }

  /** The extraction's materials made the game's: its texture times its vertex colors (the game
   * lights them) times 2, lit without shine (merc2.frag). The "-envmap" ones add the game's
   * environment texture (KHR_materials_specular's, fr3_to_gltf.cpp), looked up by the normal as
   * the camera sees it and as strong as their strength texture (the blue of their metalness one):
   * emerc.vert, emerc.frag. glTF's own reading makes them metallic: dark, with black lenses.
   * `glow`: Light Jak's wings, which the game does not light (lightjak-wings.gc: color-mult 0,
   * color-emissive 1); the glb keeps no blend mode, and only added to the picture are they the
   * game's glow. */
  private toGame(root: THREE.Object3D, glow = false) {
    const made = new Map<THREE.Material, THREE.MeshLambertMaterial>();
    root.traverse((o) => {
      const mesh = o as THREE.Mesh;
      if (!mesh.isMesh) return;
      const convert = (m: THREE.Material) => {
        let game = made.get(m);
        if (game) return game;
        const g = m as THREE.MeshPhysicalMaterial;
        const plain = (t: THREE.Texture | null) => {
          if (t) t.colorSpace = THREE.NoColorSpace;
          return t;
        };
        game = new THREE.MeshLambertMaterial({
          name: g.name,
          map: plain(g.map),
          color: g.color,
          vertexColors: g.vertexColors,
          transparent: glow || g.transparent,
          opacity: g.opacity,
          alphaTest: g.alphaTest,
          side: g.side,
          depthWrite: !glow && g.depthWrite,
          blending: glow ? THREE.AdditiveBlending : THREE.NormalBlending,
        });
        const env = plain(g.specularColorMap ?? null);
        const strength = plain(g.metalnessMap ?? null);
        // three reads any material's defines; its types give them to shader materials only
        Object.assign(game, {
          defines: {
            ...(glow && { GAME_GLOW: "" }),
            ...(env && strength && { GAME_ENV: "" }),
          },
        });
        game.onBeforeCompile = (shader) => {
          shader.uniforms.gameEnv = { value: env };
          shader.uniforms.gameStrength = { value: strength };
          shader.fragmentShader = shader.fragmentShader
            .replace(
              "#include <common>",
              `#include <common>
uniform sampler2D gameEnv;
uniform sampler2D gameStrength;`,
            )
            .replace(
              "#include <opaque_fragment>",
              `#ifdef GAME_GLOW
  outgoingLight = diffuseColor.rgb;
#endif
#if defined(GAME_ENV) && defined(USE_MAP)
  outgoingLight += texture2D(gameEnv, normal.xy * 0.5 + 0.5).rgb * texture2D(gameStrength, vMapUv).b;
#endif
#include <opaque_fragment>`,
            );
        };
        made.set(m, game);
        return game;
      };
      mesh.material = Array.isArray(mesh.material)
        ? mesh.material.map(convert)
        : convert(mesh.material);
    });
    return [...made.values()].map((material) => ({
      material,
      map: material.map,
      color: material.color.clone(),
    }));
  }

  /** each material's texture: its own, the one put in its place, or none for Light Jak (without
   * one, the environment texture is not drawn either) */
  private applyMaps() {
    for (const { material, map, color } of this.materials) {
      const swap = this.swaps.get(material.name.replace(/-envmap$/, ""));
      material.map = this.lit ? null : (swap ?? map);
      // untextured, the game draws the vertex colors alone, not doubled
      material.color.copy(color).multiplyScalar(this.lit ? 0.5 : 1);
      material.needsUpdate = true;
    }
  }

  /** Textures in place of the materials' own, by material name (without "-envmap"): the eyes
   * drawn as the game does, Dark Jak's skin. One of the same size is updated in place (the eyes
   * change with the animation's frames). */
  setTextures(textures: Map<string, ImageData>) {
    let changed = false;
    for (const [name, t] of this.swaps)
      if (!textures.has(name)) {
        t.dispose();
        this.swaps.delete(name);
        changed = true;
      }
    for (const [name, image] of textures) {
      const old = this.swaps.get(name);
      if (
        old &&
        old.image.width === image.width &&
        old.image.height === image.height
      ) {
        (old.image.data as Uint8ClampedArray).set(image.data);
        old.needsUpdate = true;
        continue;
      }
      old?.dispose();
      const t = new THREE.DataTexture(
        new Uint8ClampedArray(image.data),
        image.width,
        image.height,
      );
      t.magFilter = t.minFilter = THREE.LinearFilter;
      t.wrapS = t.wrapT = THREE.RepeatWrapping;
      t.needsUpdate = true;
      this.swaps.set(name, t);
      changed = true;
    }
    if (changed) this.applyMaps();
  }

  /** The scales the game puts on joints every frame (`joint-mod`s: Jak's neck, waist and horns,
   * target-darkjak.gc), by joint number, -1 for the whole model. */
  setJointScales(scales: [number, [number, number, number]][]) {
    this.jointScales = scales.map(([j, s]) => [j, new THREE.Vector3(...s)]);
  }

  /** Light Jak: the model untextured under the game's rim lights. */
  setLit(on: boolean) {
    this.lit = on;
    this.rim.visible = on;
    this.lights.visible = !on;
    this.applyMaps();
  }

  /** A joint's rest translation and rotation. */
  restOf(joint: number) {
    const [p, q] = this.rest.get(this.joints[joint])!;
    return { translation: p.toArray(), rotation: q.toArray() };
  }

  /** The editor: a dot on each joint to pick it, and the picked one turned or moved with the mouse. */
  setEditing(on: boolean) {
    if (!on) {
      this.select(null);
      if (this.transform) {
        this.scene.remove(this.transform.getHelper());
        this.transform.dispose();
        this.transform = undefined;
      }
      if (this.handles) {
        this.scene.remove(this.handles);
        this.handles.geometry.dispose();
        (this.handles.material as THREE.Material).dispose();
        this.handles.dispose();
        this.handles = undefined;
      }
      return;
    }
    if (this.handles || !this.joints.length) return;
    const size = this.box.getSize(new THREE.Vector3()).length() * 0.008;
    this.handles = new THREE.InstancedMesh(
      new THREE.SphereGeometry(size, 12, 8),
      new THREE.MeshBasicMaterial({ depthTest: false, transparent: true }),
      this.joints.length,
    );
    this.handles.renderOrder = 2;
    this.handles.frustumCulled = false;
    this.scene.add(this.handles);
    this.transform = new TransformControls(this.camera, this.canvas);
    this.transform.setSpace("local");
    this.transform.setSize(0.8);
    this.transform.addEventListener("dragging-changed", (e) => {
      this.controls.enabled = !e.value;
      if (!e.value && this.selected !== null)
        this.onJointMoved?.(this.selected);
    });
    this.scene.add(this.transform.getHelper());
    this.select(this.selected);
  }

  /** The joint the editor works on, turned (rotate) or moved (translate) by its handles. */
  select(joint: number | null, mode: "rotate" | "translate" = "rotate") {
    this.selected = joint;
    if (this.transform) {
      if (joint === null) this.transform.detach();
      else {
        this.transform.attach(this.joints[joint]);
        this.transform.setMode(mode);
      }
    }
    if (this.handles) {
      const color = new THREE.Color();
      for (let i = 0; i < this.joints.length; i++)
        this.handles.setColorAt(
          i,
          color.set(i === joint ? 0xf97316 : 0xe5e5e5),
        );
      this.handles.instanceColor!.needsUpdate = true;
    }
    this.onKeys?.(this.keyTimes());
  }

  private updateHandles() {
    if (!this.handles) return;
    const m = new THREE.Matrix4();
    const p = new THREE.Vector3();
    this.joints.forEach((j, i) =>
      this.handles!.setMatrixAt(i, m.makeTranslation(j.getWorldPosition(p))),
    );
    this.handles.instanceMatrix.needsUpdate = true;
  }

  /** The joint nearest the click on the screen, within a few pixels: the dots are small. */
  private pick(e: PointerEvent) {
    const r = this.canvas.getBoundingClientRect();
    const p = new THREE.Vector3();
    let best = -1;
    let nearest = 14;
    this.joints.forEach((j, i) => {
      j.getWorldPosition(p).project(this.camera);
      if (p.z > 1) return;
      const d = Math.hypot(
        ((p.x + 1) / 2) * r.width + r.left - e.clientX,
        ((1 - p.y) / 2) * r.height + r.top - e.clientY,
      );
      if (d < nearest) {
        nearest = d;
        best = i;
      }
    });
    if (best >= 0) this.onPick?.(best);
  }

  /** The skeleton at rest, to retarget onto (stops what plays). */
  rig(): Rig {
    this.play(null);
    this.model!.updateMatrixWorld(true);
    return { root: this.model!, joints: this.joints };
  }

  /** The captured body as a stick figure beside the model, following the playing animation: its
   * size the model's, its feet on the floor at the first frame. */
  setFigure(frames: PoseFrame[] | null, fps = 30) {
    if (this.figure) {
      this.scene.remove(this.figure.lines);
      this.figure.lines.geometry.dispose();
      (this.figure.lines.material as THREE.Material).dispose();
      this.figure = undefined;
      this.frame(this.box);
    }
    if (!frames?.length) return;
    const first = Object.values(frames[0]);
    const lowest = Math.min(...first.map((p) => p.y));
    const tallest = Math.max(...first.map((p) => p.y));
    const scale =
      this.box.getSize(new THREE.Vector3()).y /
      Math.max(tallest - lowest, 1e-6);
    const pelvis = mid(frames[0].hipL, frames[0].hipR) ?? new THREE.Vector3();
    const place = (p: THREE.Vector3) =>
      new THREE.Vector3(
        p.x - pelvis.x,
        p.y - lowest,
        p.z - pelvis.z,
      ).multiplyScalar(scale);
    const shape = new THREE.Box3().setFromPoints(first.map(place));
    const offset = new THREE.Vector3(
      this.box.min.x - shape.max.x - shape.getSize(new THREE.Vector3()).x * 0.2,
      this.box.min.y,
      this.box.getCenter(new THREE.Vector3()).z,
    );
    const geometry = new THREE.BufferGeometry();
    geometry.setAttribute(
      "position",
      new THREE.BufferAttribute(new Float32Array(FIGURE.length * 6), 3),
    );
    const lines = new THREE.LineSegments(
      geometry,
      new THREE.LineBasicMaterial({ color: 0xf97316 }),
    );
    lines.frustumCulled = false;
    this.scene.add(lines);
    this.figure = { lines, frames, fps, at: (p) => place(p).add(offset) };
    this.frame(this.box.clone().union(shape.translate(offset)));
  }

  private updateFigure() {
    if (!this.figure) return;
    const { frames, fps, lines, at } = this.figure;
    const frame =
      frames[Math.min(frames.length - 1, Math.floor(this.time * fps))];
    const positions = lines.geometry.getAttribute(
      "position",
    ) as THREE.BufferAttribute;
    FIGURE.forEach(([a, b], k) => {
      const pa = frame[a];
      const pb = frame[b];
      // a missing point draws nothing: both ends of its line at the first one
      const va = pa && pb ? at(pa) : at(frames[0].hipL ?? new THREE.Vector3());
      const vb = pa && pb ? at(pb) : va;
      positions.setXYZ(k * 2, va.x, va.y, va.z);
      positions.setXYZ(k * 2 + 1, vb.x, vb.y, vb.z);
    });
    positions.needsUpdate = true;
  }

  dispose() {
    this.renderer.setAnimationLoop(null);
    this.observer.disconnect();
    this.unload();
    this.controls.dispose();
    this.renderer.dispose();
  }
}
