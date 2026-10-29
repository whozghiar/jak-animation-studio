// yarn check:retarget — a humanoid rig whose bones have random axes is posed from captured points:
// every mapped bone must point as the captured limb does, the hips face as the captured hips, and
// the travel of a mocap lands on `align`, scaled from the captured legs to the model's.

import assert from "node:assert/strict";
import * as THREE from "three";
import { retarget } from "./retarget.ts";
import type { PoseFrame } from "./frames.ts";

const v = (x: number, y: number, z: number) => new THREE.Vector3(x, y, z);

// name, parent, world position at rest (a T pose)
const REST: [string, string | null, THREE.Vector3][] = [
  ["align", null, v(0, 0, 0)],
  ["hips", "align", v(0, 1, 0)],
  ["chest", "hips", v(0, 1.4, 0)],
  ["neck", "chest", v(0, 1.6, 0)],
  ["head", "neck", v(0, 1.7, 0)],
  ["headTop", "head", v(0, 1.9, 0)],
  ["armL", "chest", v(0.2, 1.55, 0)],
  ["foreL", "armL", v(0.5, 1.55, 0)],
  ["handL", "foreL", v(0.75, 1.55, 0)],
  ["fingerL", "handL", v(0.85, 1.55, 0)],
  ["armR", "chest", v(-0.2, 1.55, 0)],
  ["foreR", "armR", v(-0.5, 1.55, 0)],
  ["handR", "foreR", v(-0.75, 1.55, 0)],
  ["fingerR", "handR", v(-0.85, 1.55, 0)],
  ["thighL", "hips", v(0.1, 1, 0)],
  ["shinL", "thighL", v(0.1, 0.55, 0)],
  ["footL", "shinL", v(0.1, 0.1, 0)],
  ["toeL", "footL", v(0.1, 0.05, 0.15)],
  ["thighR", "hips", v(-0.1, 1, 0)],
  ["shinR", "thighR", v(-0.1, 0.55, 0)],
  ["footR", "shinR", v(-0.1, 0.1, 0)],
  ["toeR", "footR", v(-0.1, 0.05, 0.15)],
];

let seed = 7;
function random() {
  seed = (seed + 0x6d2b79f5) | 0;
  let t = Math.imul(seed ^ (seed >>> 15), 1 | seed);
  t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
  return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
}

function rig() {
  const root = new THREE.Group();
  const joints: THREE.Object3D[] = [];
  for (const [name, parentName, pos] of REST) {
    const parent = parentName
      ? joints.find((j) => j.name === parentName)!
      : root;
    const joint = new THREE.Object3D();
    joint.name = name;
    parent.add(joint);
    root.updateMatrixWorld(true);
    const axes = new THREE.Quaternion()
      .setFromEuler(new THREE.Euler(random() * 6, random() * 6, random() * 6))
      .normalize();
    joint.quaternion.copy(
      parent.getWorldQuaternion(new THREE.Quaternion()).invert().multiply(axes),
    );
    joint.position.copy(parent.worldToLocal(pos.clone()));
    joints.push(joint);
  }
  root.updateMatrixWorld(true);
  return { root, joints };
}

const at = (name: string) => REST.findIndex(([n]) => n === name);
const mapping = {
  hips: at("hips"),
  chest: at("chest"),
  neck: at("neck"),
  head: at("head"),
  upperArmL: at("armL"),
  foreArmL: at("foreL"),
  handL: at("handL"),
  upperArmR: at("armR"),
  foreArmR: at("foreR"),
  handR: at("handR"),
  thighL: at("thighL"),
  shinL: at("shinL"),
  footL: at("footL"),
  thighR: at("thighR"),
  shinR: at("shinR"),
  footR: at("footR"),
};

// hips turned 30 degrees, left arm bent forward, right arm raised, left knee up
const turn = new THREE.Quaternion().setFromAxisAngle(v(0, 1, 0), Math.PI / 6);
const body = (p: THREE.Vector3) => p.applyQuaternion(turn);
function pose(shift = 0): PoseFrame {
  const f: PoseFrame = {
    hipL: body(v(0.1, 0, 0)),
    hipR: body(v(-0.1, 0, 0)),
    shoulderL: body(v(0.2, 0.55, 0)),
    shoulderR: body(v(-0.2, 0.55, 0)),
  };
  f.elbowL = f.shoulderL!.clone().add(v(0.05, -0.3, 0.1));
  f.wristL = f.elbowL.clone().add(v(0, -0.1, 0.25));
  f.handL = f.wristL.clone().add(v(0, 0, 0.1));
  f.elbowR = f.shoulderR!.clone().add(v(0, 0.3, 0));
  f.wristR = f.elbowR.clone().add(v(-0.1, 0.25, 0));
  f.handR = f.wristR.clone().add(v(0, 0.1, 0));
  f.kneeL = f.hipL!.clone().add(v(0, -0.3, 0.3));
  f.ankleL = f.kneeL.clone().add(v(0, -0.45, 0));
  f.toeL = f.ankleL.clone().add(v(0, -0.02, 0.15));
  f.kneeR = f.hipR!.clone().add(v(0, -0.45, 0));
  f.ankleR = f.kneeR.clone().add(v(0, -0.4, -0.15));
  f.toeR = f.ankleR.clone().add(v(0, 0, 0.15));
  f.head = f
    .shoulderL!.clone()
    .add(f.shoulderR!)
    .multiplyScalar(0.5)
    .add(v(0, 0.25, 0.05));
  for (const p of Object.values(f)) p.x += shift;
  return f;
}

const options = { smoothing: 0, inPlace: true, faceForward: false };
const r = rig();
const out = retarget(
  r,
  mapping,
  { fps: 30, frames: [pose(), pose()], absolute: false },
  options,
);
for (const c of out.channels)
  r.joints[c.joint].quaternion.fromArray(c.values, 0);
r.root.updateMatrixWorld(true);

const world = (name: string) =>
  r.joints[at(name)].getWorldPosition(new THREE.Vector3());
const f = pose();
const neckBase = f.shoulderL!.clone().add(f.shoulderR!).multiplyScalar(0.5);
const pairs: [string, string, THREE.Vector3, THREE.Vector3][] = [
  ["armL", "foreL", f.shoulderL!, f.elbowL!],
  ["foreL", "handL", f.elbowL!, f.wristL!],
  ["handL", "fingerL", f.wristL!, f.handL!],
  ["armR", "foreR", f.shoulderR!, f.elbowR!],
  ["foreR", "handR", f.elbowR!, f.wristR!],
  ["handR", "fingerR", f.wristR!, f.handR!],
  ["thighL", "shinL", f.hipL!, f.kneeL!],
  ["shinL", "footL", f.kneeL!, f.ankleL!],
  ["footL", "toeL", f.ankleL!, f.toeL!],
  ["thighR", "shinR", f.hipR!, f.kneeR!],
  ["shinR", "footR", f.kneeR!, f.ankleR!],
  ["footR", "toeR", f.ankleR!, f.toeR!],
  ["neck", "head", neckBase, f.head!],
  ["thighR", "thighL", f.hipR!, f.hipL!],
];
for (const [a, b, from, to] of pairs) {
  const posed = world(b).sub(world(a)).normalize();
  const wanted = to.clone().sub(from).normalize();
  const degrees = THREE.MathUtils.radToDeg(posed.angleTo(wanted));
  assert.ok(degrees < 0.5, `${a} -> ${b} is ${degrees.toFixed(2)} degrees off`);
}
assert.equal(out.clip.tracks.length, Object.keys(mapping).length);

// a mocap moving 1 unit along X: the travel goes to align, scaled by 0.9 / 0.874 (model / captured legs)
const travel = retarget(
  rig(),
  mapping,
  { fps: 30, frames: [pose(0), pose(0.5), pose(1)], absolute: true },
  { ...options, inPlace: false },
);
const align = travel.channels.find(
  (c) => c.joint === at("align") && c.path === "translation",
);
assert.ok(align, "the travel is on align");
const legs = 0.9 / (Math.hypot(0.3, 0.3) + 0.45);
assert.ok(
  Math.abs(align.values[6] - legs) < 1e-4,
  `align ends at x=${align.values[6]}`,
);
console.log(
  `retarget: ${pairs.length} bone directions and the travel check out`,
);
