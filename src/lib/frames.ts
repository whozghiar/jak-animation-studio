// A body's movement as points over time (hips, knees, wrists...), Y up, whatever it was read from:
// retargeting reads only these points, so every source animates every skeleton the same way.

import * as THREE from "three";

export type Point =
  | "hipL"
  | "hipR"
  | "shoulderL"
  | "shoulderR"
  | "elbowL"
  | "elbowR"
  | "wristL"
  | "wristR"
  | "handL"
  | "handR"
  | "kneeL"
  | "kneeR"
  | "ankleL"
  | "ankleR"
  | "toeL"
  | "toeR"
  | "head"
  | "nose"
  | "earL"
  | "earR"
  | "eyeL"
  | "eyeR"
  | "mouth";

export type PoseFrame = Partial<Record<Point, THREE.Vector3>>;

export type Capture = {
  fps: number;
  frames: PoseFrame[];
  /** the points move through space with the body (mocap); else they stay around the hips (video) */
  absolute: boolean;
};

/** The lines of the stick figure drawn next to the model. */
export const FIGURE: [Point, Point][] = [
  ["shoulderL", "shoulderR"],
  ["shoulderL", "elbowL"],
  ["elbowL", "wristL"],
  ["wristL", "handL"],
  ["shoulderR", "elbowR"],
  ["elbowR", "wristR"],
  ["wristR", "handR"],
  ["hipL", "hipR"],
  ["hipL", "shoulderL"],
  ["hipR", "shoulderR"],
  ["hipL", "kneeL"],
  ["kneeL", "ankleL"],
  ["ankleL", "toeL"],
  ["hipR", "kneeR"],
  ["kneeR", "ankleR"],
  ["ankleR", "toeR"],
  ["earL", "earR"],
  ["nose", "head"],
];

export function mid(a?: THREE.Vector3, b?: THREE.Vector3) {
  return a && b ? a.clone().add(b).multiplyScalar(0.5) : undefined;
}

/** Each point the mean of itself and its `radius` neighbours on each side: less shaking. */
export function smooth(frames: PoseFrame[], radius: number): PoseFrame[] {
  if (radius < 1) return frames;
  return frames.map((_, i) => {
    const out: PoseFrame = {};
    for (const key of Object.keys(frames[i]) as Point[]) {
      const sum = new THREE.Vector3();
      let n = 0;
      for (
        let j = Math.max(0, i - radius);
        j <= Math.min(frames.length - 1, i + radius);
        j++
      ) {
        const p = frames[j][key];
        if (p) {
          sum.add(p);
          n++;
        }
      }
      out[key] = sum.divideScalar(n);
    }
    return out;
  });
}

/** Turned about the vertical so that the body faces +Z on the first frame. */
export function faceForward(frames: PoseFrame[]): PoseFrame[] {
  const f = frames[0];
  const pelvis = mid(f.hipL, f.hipR);
  const top = mid(f.shoulderL, f.shoulderR);
  if (!f.hipL || !f.hipR || !pelvis || !top) return frames;
  const forward = new THREE.Vector3().crossVectors(
    f.hipL.clone().sub(f.hipR),
    top.clone().sub(pelvis),
  );
  const turn = new THREE.Quaternion().setFromAxisAngle(
    new THREE.Vector3(0, 1, 0),
    -Math.atan2(forward.x, forward.z),
  );
  return frames.map((frame) => {
    const out: PoseFrame = {};
    for (const [key, p] of Object.entries(frame) as [Point, THREE.Vector3][])
      out[key] = p.clone().sub(pelvis).applyQuaternion(turn).add(pelvis);
    return out;
  });
}
