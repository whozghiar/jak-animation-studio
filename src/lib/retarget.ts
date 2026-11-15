// Puts a captured movement on a model's skeleton. Each mapped bone is turned, from its parent's new
// pose, by the smallest rotation that points it as the captured limb points (upper arm from the
// shoulder to the elbow...); the hips, the chest and the head take the whole orientation of the
// captured body. Only directions are copied, never lengths: any proportions, any rest pose (T or A),
// any bone axes. Unmapped bones keep their rest rotation, or, given a `base` animation, the pose they
// have at its start when no mapped bone hangs from them. A prop a hand holds at that start (a guard's
// gun, whose rest pose lies on the floor: the extraction's bind pose) follows that hand; with
// `holdWeapon`, the arms keep the base animation's pose too, so both hands stay on it.

import * as THREE from "three";
import {
  faceForward,
  mid,
  smooth,
  type Capture,
  type Point,
  type PoseFrame,
} from "./frames.ts";
import type { Mapping, Slot } from "./mapping";
import type { Channel } from "./rpc";

export type RetargetOptions = {
  /** frames averaged on each side of a frame */
  smoothing: number;
  /** no travel across the floor (the hips still go up and down) */
  inPlace: boolean;
  /** the body turned to face +Z on the first frame */
  faceForward: boolean;
  /** a prop held in the hands (a gun): the arms keep the base animation's pose, holding it */
  holdWeapon: boolean;
};

/** The New animation panel's options at first, and those of the MCP server's movements. */
export const DEFAULTS: RetargetOptions = {
  smoothing: 2,
  inPlace: true,
  faceForward: true,
  holdWeapon: true,
};

/** The skeleton to animate at its rest pose, its joints in the order of the skin. */
export type Rig = { root: THREE.Object3D; joints: THREE.Object3D[] };

export type Retarget = {
  clip: THREE.AnimationClip;
  channels: Channel[];
  /** the captured points as used, for the stick figure */
  frames: PoseFrame[];
  /** a hand holds a prop at the base animation's start */
  weapon: boolean;
};

type From = Point | "neckBase";

/** bone, from, to (in the capture), the slot whose joint it points at in the model */
const AIMS: [Slot, From, Point, Slot | null][] = [
  ["upperArmL", "shoulderL", "elbowL", "foreArmL"],
  ["foreArmL", "elbowL", "wristL", "handL"],
  ["handL", "wristL", "handL", null],
  ["upperArmR", "shoulderR", "elbowR", "foreArmR"],
  ["foreArmR", "elbowR", "wristR", "handR"],
  ["handR", "wristR", "handR", null],
  ["thighL", "hipL", "kneeL", "shinL"],
  ["shinL", "kneeL", "ankleL", "footL"],
  ["footL", "ankleL", "toeL", null],
  ["thighR", "hipR", "kneeR", "shinR"],
  ["shinR", "kneeR", "ankleR", "footR"],
  ["footR", "ankleR", "toeR", null],
  ["neck", "neckBase", "head", "head"],
];

type Body = "hips" | "chest" | "head";

function point(f: PoseFrame, p: From) {
  const v = p === "neckBase" ? mid(f.shoulderL, f.shoulderR) : f[p];
  return v?.clone();
}

/** The rotation whose X axis is `side` (towards the body's left) and Y axis `up`, Z forward. */
function basis(side: THREE.Vector3, up: THREE.Vector3) {
  const x = side.clone().normalize();
  const z = new THREE.Vector3().crossVectors(x, up).normalize();
  const y = new THREE.Vector3().crossVectors(z, x);
  return new THREE.Quaternion().setFromRotationMatrix(
    new THREE.Matrix4().makeBasis(x, y, z),
  );
}

function bodyBases(f: PoseFrame): Partial<Record<Body, THREE.Quaternion>> {
  const pelvis = mid(f.hipL, f.hipR);
  const top = mid(f.shoulderL, f.shoulderR);
  const out: Partial<Record<Body, THREE.Quaternion>> = {};
  if (!pelvis || !top) return out;
  const up = top.clone().sub(pelvis);
  if (f.hipL && f.hipR) out.hips = basis(f.hipL.clone().sub(f.hipR), up);
  if (f.shoulderL && f.shoulderR)
    out.chest = basis(f.shoulderL.clone().sub(f.shoulderR), up);
  const eyes = mid(f.eyeL, f.eyeR);
  if (f.earL && f.earR && eyes && f.mouth)
    out.head = basis(f.earL.clone().sub(f.earR), eyes.sub(f.mouth));
  return out;
}

export function retarget(
  rig: Rig,
  mapping: Mapping,
  capture: Capture,
  options: RetargetOptions,
  base?: THREE.AnimationClip,
): Retarget {
  let frames = smooth(capture.frames, options.smoothing);
  if (options.faceForward) frames = faceForward(frames);
  const { joints, root } = rig;
  root.updateMatrixWorld(true);

  const index = new Map(joints.map((j, i) => [j, i]));
  const parent = joints.map((j) =>
    j.parent ? (index.get(j.parent) ?? -1) : -1,
  );
  const depth = (i: number): number =>
    parent[i] < 0 ? 0 : 1 + depth(parent[i]);
  const order = joints.map((_, i) => i).sort((a, b) => depth(a) - depth(b));
  const restPos = joints.map((j) => j.getWorldPosition(new THREE.Vector3()));
  const restQ = joints.map((j) => j.getWorldQuaternion(new THREE.Quaternion()));
  const restLocal = joints.map((j) => j.quaternion.clone());
  const outside = joints.map((j) =>
    j.parent
      ? j.parent.getWorldQuaternion(new THREE.Quaternion())
      : new THREE.Quaternion(),
  );
  const slot = (s: Slot) => {
    const i = mapping[s];
    return i !== undefined && i < joints.length ? i : undefined;
  };
  const pos = (s: Slot) => {
    const i = slot(s);
    return i === undefined ? undefined : restPos[i];
  };

  // where a bone points at rest: at the next mapped joint, else at its children
  const children = joints.map((_, i) =>
    parent.flatMap((p, c) => (p === i ? [c] : [])),
  );
  const descendants = (i: number): number =>
    children[i].reduce((n, c) => n + 1 + descendants(c), 0);
  const childrenEnd = (i: number) => {
    const kids = children[i];
    if (!kids.length) return undefined;
    const most = Math.max(...kids.map(descendants));
    const chosen = kids.filter((c) => descendants(c) === most);
    return chosen
      .reduce((sum, c) => sum.add(restPos[c]), new THREE.Vector3())
      .divideScalar(chosen.length);
  };
  const aims = new Map<
    number,
    { from: From; to: Point; local: THREE.Vector3 }
  >();
  for (const [s, from, to, next] of AIMS) {
    const i = slot(s);
    if (i === undefined) continue;
    const end = (next && pos(next)) || childrenEnd(i);
    const dir = end?.clone().sub(restPos[i]);
    if (!dir || dir.lengthSq() < 1e-12) continue;
    aims.set(i, {
      from,
      to,
      local: dir.normalize().applyQuaternion(restQ[i].clone().invert()),
    });
  }

  // the body's orientation at rest, from the model's hips and shoulders
  const thighs = mid(pos("thighL"), pos("thighR"));
  const shoulders = mid(pos("upperArmL"), pos("upperArmR"));
  const up = thighs && shoulders ? shoulders.clone().sub(thighs) : undefined;
  const hipsSide = pos("thighL")?.clone().sub(pos("thighR")!);
  const chestSide =
    pos("upperArmL")?.clone().sub(pos("upperArmR")!) ?? hipsSide;
  const bodies = new Map<number, { body: Body; restInv: THREE.Quaternion }>();
  if (up && hipsSide) {
    const restBody = basis(hipsSide, up).invert();
    for (const [s, body, restInv] of [
      ["hips", "hips", restBody],
      ["chest", "chest", chestSide ? basis(chestSide, up).invert() : restBody],
      // the head looks forward at rest
      ["head", "head", restBody],
    ] as [Slot, Body, THREE.Quaternion][]) {
      const i = slot(s);
      if (i !== undefined) bodies.set(i, { body, restInv });
    }
  }

  // the base animation's first keys, else the rest pose: each joint's local position, rotation, scale
  type Local = { p: THREE.Vector3; q: THREE.Quaternion; s: THREE.Vector3 };
  const first = (i: number, property: string) => {
    const track = base?.tracks.find(
      (t) =>
        t.name === `${joints[i].uuid}.${property}` ||
        t.name === `${joints[i].name}.${property}`,
    );
    return track && [...track.values.slice(0, track.getValueSize())];
  };
  const rest: Local[] = joints.map((j) => ({
    p: j.position.clone(),
    q: j.quaternion.clone(),
    s: j.scale.clone(),
  }));
  const baseLocal: Local[] = joints.map((_, i) => {
    const [p, q, sc] = ["position", "quaternion", "scale"].map((k) =>
      first(i, k),
    );
    return {
      p: p ? new THREE.Vector3().fromArray(p) : rest[i].p,
      q: q ? new THREE.Quaternion().fromArray(q) : rest[i].q,
      s: sc ? new THREE.Vector3().fromArray(sc) : rest[i].s,
    };
  });
  const outsideM = joints.map((j) =>
    j.parent ? j.parent.matrixWorld.clone() : new THREE.Matrix4(),
  );
  const worldOf = (local: (i: number) => Local) => {
    const m: THREE.Matrix4[] = [];
    for (const i of order) {
      const { p, q, s } = local(i);
      m[i] = (parent[i] >= 0 ? m[parent[i]] : outsideM[i])
        .clone()
        .multiply(new THREE.Matrix4().compose(p, q, s));
    }
    return m;
  };

  // the props: joints no mapped bone hangs from, under a mapped one, outside the hands, that a hand
  // holds at the base animation's start (within its forearm's length): each keeps its place in it.
  // A prop hangs from the body (the guard's gun from swivelMe), never from a limb, the neck or the
  // head, nor from hips apart from the upper body: Daxter's first animation starts with the balls of
  // his feet (Lball, Rball) by his hands, and Babak, who holds nothing, has joints there too.
  const reaches = (i: number, set: Set<number>): boolean =>
    set.has(i) || children[i].some((c) => reaches(c, set));
  const mapped = new Set([...aims.keys(), ...bodies.keys()]);
  const hipsJoint = slot("hips");
  const chestJoint = slot("chest");
  const limbs = new Set(
    (
      [
        "thighL",
        "shinL",
        "footL",
        "thighR",
        "shinR",
        "footR",
        "upperArmL",
        "foreArmL",
        "upperArmR",
        "foreArmR",
        "neck",
        "head",
      ] as const
    )
      .map(slot)
      .filter((i) => i !== undefined),
  );
  if (
    hipsJoint !== undefined &&
    !(chestJoint !== undefined && reaches(hipsJoint, new Set([chestJoint])))
  )
    limbs.add(hipsJoint);
  const onLimb = (i: number): boolean =>
    i >= 0 && (limbs.has(i) || onLimb(parent[i]));
  const props: { joint: number; hand: number; inHand: THREE.Matrix4 }[] = [];
  if (base) {
    const baseWorld = worldOf((i) => baseLocal[i]);
    const at = (i: number) =>
      new THREE.Vector3().setFromMatrixPosition(baseWorld[i]);
    const hands = (["L", "R"] as const).flatMap((side) => {
      const hand = slot(`hand${side}`);
      const fore = slot(`foreArm${side}`);
      return hand === undefined || fore === undefined
        ? []
        : [{ hand, reach: at(fore).distanceTo(at(hand)) }];
    });
    const inHands = (i: number): boolean =>
      i >= 0 && (hands.some((h) => h.hand === i) || inHands(parent[i]));
    for (const [i] of joints.entries()) {
      if (parent[i] < 0 || reaches(i, mapped) || !reaches(parent[i], mapped))
        continue;
      if (inHands(i) || onLimb(i)) continue;
      const holder = hands
        .map((h) => ({ ...h, d: at(i).distanceTo(at(h.hand)) }))
        .filter((h) => h.d < h.reach)
        .sort((a, b) => a.d - b.d)[0];
      if (holder)
        props.push({
          joint: i,
          hand: holder.hand,
          inHand: baseWorld[holder.hand]
            .clone()
            .invert()
            .multiply(baseWorld[i]),
        });
    }
  }
  if (props.length && options.holdWeapon)
    for (const side of ["L", "R"] as const)
      for (const s of [
        `upperArm${side}`,
        `foreArm${side}`,
        `hand${side}`,
      ] as const) {
        const i = slot(s);
        if (i !== undefined) aims.delete(i);
      }

  const n = frames.length;
  const times = frames.map((_, f) => f / capture.fps);
  const rotations = new Map<number, number[]>();
  for (const f of frames) {
    const world: THREE.Quaternion[] = [];
    const captured = bodyBases(f);
    for (const i of order) {
      const parentQ = parent[i] >= 0 ? world[parent[i]] : outside[i];
      const inherited = parentQ.clone().multiply(restLocal[i]);
      let q = inherited;
      const body = bodies.get(i);
      const aim = aims.get(i);
      const target = body && captured[body.body];
      if (target) {
        q = target.clone().multiply(body.restInv).multiply(restQ[i]);
      } else if (aim) {
        const from = point(f, aim.from);
        const want = point(f, aim.to)?.sub(from ?? new THREE.Vector3());
        if (from && want && want.lengthSq() > 1e-12) {
          const now = aim.local.clone().applyQuaternion(inherited);
          q = new THREE.Quaternion()
            .setFromUnitVectors(now.normalize(), want.normalize())
            .multiply(inherited);
        }
      }
      world[i] = q;
      if (!body && !aim) continue;
      const local = parentQ.clone().invert().multiply(q);
      const keys = rotations.get(i) ?? [];
      const prev = keys.length
        ? new THREE.Quaternion().fromArray(keys, keys.length - 4)
        : null;
      if (prev && prev.dot(local) < 0)
        local.set(-local.x, -local.y, -local.z, -local.w);
      keys.push(local.x, local.y, local.z, local.w);
      rotations.set(i, keys);
    }
  }

  // the hips' travel, scaled from the captured legs to the model's
  const translations = new Map<number, number[]>();
  const hips = slot("hips");
  const f0 = frames[0];
  const pelvis0 = mid(f0.hipL, f0.hipR);
  if (capture.absolute && hips !== undefined && pelvis0) {
    const leg = (a?: THREE.Vector3, b?: THREE.Vector3, c?: THREE.Vector3) =>
      a && b && c ? a.distanceTo(b) + b.distanceTo(c) : 0;
    const model = leg(pos("thighL"), pos("shinL"), pos("footL"));
    const captured = leg(f0.hipL, f0.kneeL, f0.ankleL);
    const scale = model > 0 && captured > 0 ? model / captured : 1;
    // travel goes to `align` (it moves the entity in game), else to the hips
    const travel = joints[0].name === "align" && hips !== 0 ? 0 : hips;
    const toLocal = (i: number, world: THREE.Vector3) =>
      joints[i].parent
        ? joints[i].parent!.worldToLocal(world.clone())
        : world.clone();
    for (const f of frames) {
      const delta = (mid(f.hipL, f.hipR) ?? pelvis0)
        .clone()
        .sub(pelvis0)
        .multiplyScalar(scale);
      const moves = new Map<number, THREE.Vector3>([
        [hips, new THREE.Vector3(0, delta.y, 0)],
      ]);
      if (!options.inPlace) {
        const across = new THREE.Vector3(delta.x, 0, delta.z);
        moves.set(
          travel,
          (moves.get(travel) ?? new THREE.Vector3()).add(across),
        );
      }
      for (const [i, d] of moves) {
        const keys = translations.get(i) ?? [];
        keys.push(...toLocal(i, restPos[i].clone().add(d)).toArray());
        translations.set(i, keys);
      }
    }
  }

  // the joints no mapped bone hangs from: the first keys of the base animation (one key: held)
  const driven = new Set([...aims.keys(), ...bodies.keys()]);
  const isProp = new Set(props.map((p) => p.joint));
  const held: Channel[] = [];
  for (const [i] of joints.entries()) {
    if (reaches(i, driven) || translations.has(i) || isProp.has(i)) continue;
    for (const [path, property] of [
      ["rotation", "quaternion"],
      ["translation", "position"],
      ["scale", "scale"],
    ] as const) {
      const values = first(i, property);
      if (values) held.push({ joint: i, path, times: [0], values });
    }
  }

  // the props, frame by frame, where their hand holds them
  const heldAt = (i: number, path: Channel["path"]) =>
    held.find((c) => c.joint === i && c.path === path)?.values;
  const propKeys = props.map(() => ({ q: [] as number[], p: [] as number[] }));
  for (let f = 0; f < n; f++) {
    const world = worldOf((i) => {
      const r = rotations.get(i);
      const t = translations.get(i);
      const hq = heldAt(i, "rotation");
      const hp = heldAt(i, "translation");
      const hs = heldAt(i, "scale");
      return {
        q: r
          ? new THREE.Quaternion().fromArray(r, f * 4)
          : hq
            ? new THREE.Quaternion().fromArray(hq)
            : rest[i].q,
        p: t
          ? new THREE.Vector3().fromArray(t, f * 3)
          : hp
            ? new THREE.Vector3().fromArray(hp)
            : rest[i].p,
        s: hs ? new THREE.Vector3().fromArray(hs) : rest[i].s,
      };
    });
    for (const [k, prop] of props.entries()) {
      const local = world[parent[prop.joint]]
        .clone()
        .invert()
        .multiply(world[prop.hand])
        .multiply(prop.inHand);
      const p = new THREE.Vector3();
      const q = new THREE.Quaternion();
      local.decompose(p, q, new THREE.Vector3());
      const keys = propKeys[k];
      const prev = keys.q.length
        ? new THREE.Quaternion().fromArray(keys.q, keys.q.length - 4)
        : null;
      if (prev && prev.dot(q) < 0) q.set(-q.x, -q.y, -q.z, -q.w);
      keys.q.push(q.x, q.y, q.z, q.w);
      keys.p.push(p.x, p.y, p.z);
    }
  }
  for (const [k, prop] of props.entries()) {
    rotations.set(prop.joint, propKeys[k].q);
    translations.set(prop.joint, propKeys[k].p);
  }

  const tracks: THREE.KeyframeTrack[] = [];
  const channels: Channel[] = [];
  for (const c of held) {
    const property = {
      rotation: "quaternion",
      translation: "position",
      scale: "scale",
    }[c.path];
    tracks.push(
      new (c.path === "rotation"
        ? THREE.QuaternionKeyframeTrack
        : THREE.VectorKeyframeTrack)(
        `${joints[c.joint].uuid}.${property}`,
        c.times,
        c.values,
      ),
    );
    channels.push(c);
  }
  for (const [i, values] of rotations) {
    tracks.push(
      new THREE.QuaternionKeyframeTrack(
        `${joints[i].uuid}.quaternion`,
        times,
        values,
      ),
    );
    channels.push({ joint: i, path: "rotation", times, values });
  }
  for (const [i, values] of translations) {
    tracks.push(
      new THREE.VectorKeyframeTrack(
        `${joints[i].uuid}.position`,
        times,
        values,
      ),
    );
    channels.push({ joint: i, path: "translation", times, values });
  }
  const duration = n > 1 ? times[n - 1] : 1 / capture.fps;
  return {
    clip: new THREE.AnimationClip("preview", duration, tracks),
    channels,
    frames,
    weapon: props.length > 0,
  };
}
