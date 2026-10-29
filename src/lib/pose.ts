// The body points of frames.ts read from a video (MediaPipe Pose, in the window) or from a motion
// capture file (BVH, FBX, glTF).

import * as THREE from "three";
import { PoseLandmarker, type Landmark } from "@mediapipe/tasks-vision";
import wasmLoaderPath from "@mediapipe/tasks-vision/vision_wasm_internal.js?url";
import wasmBinaryPath from "@mediapipe/tasks-vision/vision_wasm_internal.wasm?url";
import { BVHLoader } from "three/addons/loaders/BVHLoader.js";
import { FBXLoader } from "three/addons/loaders/FBXLoader.js";
import { GLTFLoader } from "three/addons/loaders/GLTFLoader.js";
import { autoMap, type Slot } from "./mapping";
import { t } from "./i18n.svelte";
import type { Capture, Point, PoseFrame } from "./frames.ts";

// ponytail: the model is fetched from Google's servers the first time a video is read (then cached
// by the webview); ship it with the app when the studio must work offline.
const MODEL_URL =
  "https://storage.googleapis.com/mediapipe-models/pose_landmarker/pose_landmarker_full/float16/latest/pose_landmarker_full.task";

let landmarker: Promise<PoseLandmarker> | undefined;
/** MediaPipe wants video timestamps that always go up, from one video to the next too. */
let clock = 0;

function getLandmarker() {
  const create = (delegate: "GPU" | "CPU") =>
    PoseLandmarker.createFromOptions(
      { wasmLoaderPath, wasmBinaryPath },
      {
        baseOptions: { modelAssetPath: MODEL_URL, delegate },
        runningMode: "VIDEO",
        numPoses: 1,
      },
    );
  if (!landmarker) {
    landmarker = create("GPU").catch(() => create("CPU"));
    landmarker.catch(() => (landmarker = undefined));
  }
  return landmarker;
}

// MediaPipe's 33 landmarks: the mean of several where a point lies between them
const LANDMARKS: [Point, number[]][] = [
  ["nose", [0]],
  ["eyeL", [2]],
  ["eyeR", [5]],
  ["earL", [7]],
  ["earR", [8]],
  ["mouth", [9, 10]],
  ["shoulderL", [11]],
  ["shoulderR", [12]],
  ["elbowL", [13]],
  ["elbowR", [14]],
  ["wristL", [15]],
  ["wristR", [16]],
  ["handL", [17, 19]],
  ["handR", [18, 20]],
  ["hipL", [23]],
  ["hipR", [24]],
  ["kneeL", [25]],
  ["kneeR", [26]],
  ["ankleL", [27]],
  ["ankleR", [28]],
  ["toeL", [31]],
  ["toeR", [32]],
];

/** MediaPipe's world landmarks (x right in the picture, y down, z away from the camera) turned to
 * Y up, the filmed person facing +Z (towards the camera), their left on +X: glTF's convention. */
function videoFrame(world: Landmark[]): PoseFrame {
  const frame: PoseFrame = {};
  for (const [point, ids] of LANDMARKS) {
    const v = new THREE.Vector3();
    for (const i of ids)
      v.add(new THREE.Vector3(world[i].x, -world[i].y, -world[i].z));
    frame[point] = v.divideScalar(ids.length);
  }
  frame.head = frame.earL!.clone().add(frame.earR!).multiplyScalar(0.5);
  return frame;
}

export async function captureVideo(
  file: File,
  fps: number,
  onProgress: (done: number) => void,
  signal: AbortSignal,
): Promise<Capture> {
  const pose = await getLandmarker();
  const video = document.createElement("video");
  video.muted = true;
  video.preload = "auto";
  const url = URL.createObjectURL(file);
  try {
    video.src = url;
    await new Promise<void>((ok, fail) => {
      video.onloadeddata = () => ok();
      video.onerror = () =>
        fail(new Error(t("videoUnreadable", { file: file.name })));
    });
    const count = Math.max(1, Math.floor(video.duration * fps));
    const frames: PoseFrame[] = [];
    for (let i = 0; i < count; i++) {
      if (signal.aborted) throw new DOMException("Stopped", "AbortError");
      const seeked = new Promise((ok) => (video.onseeked = ok));
      video.currentTime = i / fps;
      await seeked;
      clock += 1000 / fps;
      const world = pose.detectForVideo(video, clock).worldLandmarks[0];
      frames.push(world ? videoFrame(world) : (frames.at(-1) ?? {}));
      onProgress((i + 1) / count);
    }
    const first = frames.findIndex((f) => f.hipL);
    if (first < 0) throw new Error(t("noBody", { file: file.name }));
    frames.fill(frames[first], 0, first);
    return { fps, frames, absolute: false };
  } finally {
    URL.revokeObjectURL(url);
    video.removeAttribute("src");
  }
}

export type Mocap = {
  /** what the clips' tracks are bound to */
  root: THREE.Object3D;
  clips: THREE.AnimationClip[];
  bones: THREE.Object3D[];
};

function bonesOf(root: THREE.Object3D) {
  const bones: THREE.Object3D[] = [];
  root.traverse((o) => {
    if ((o as THREE.Bone).isBone) bones.push(o);
  });
  return bones;
}

export async function loadMocap(file: File): Promise<Mocap> {
  const ext = file.name.split(".").pop()?.toLowerCase();
  if (ext === "bvh") {
    const { skeleton, clip } = new BVHLoader().parse(await file.text());
    const root = new THREE.Group();
    root.add(skeleton.bones[0]);
    return { root, clips: [clip], bones: skeleton.bones };
  }
  if (ext === "fbx") {
    const group = new FBXLoader().parse(await file.arrayBuffer(), "");
    return {
      root: group,
      clips: group.animations,
      bones: bonesOf(group),
    };
  }
  if (ext === "glb" || ext === "gltf") {
    const data = ext === "glb" ? await file.arrayBuffer() : await file.text();
    const gltf = await new GLTFLoader().parseAsync(data, "");
    return {
      root: gltf.scene,
      clips: gltf.animations,
      bones: bonesOf(gltf.scene),
    };
  }
  throw new Error(t("notMocap", { file: file.name }));
}

export function captureMocap(
  mocap: Mocap,
  clip: THREE.AnimationClip,
  fps: number,
): Capture {
  const { bones } = mocap;
  const map = autoMap(
    bones.map((b) => {
      const parent = bones.indexOf(b.parent as THREE.Object3D);
      return { name: b.name, parent: parent < 0 ? null : parent };
    }),
  );
  const bone = (s: Slot) => (map[s] === undefined ? undefined : bones[map[s]!]);
  const at = (s: Slot) => bone(s)?.getWorldPosition(new THREE.Vector3());
  /** past the end of a bone: the middle of its children, else its own length on */
  const tip = (s: Slot, from: Slot) => {
    const b = bone(s);
    if (!b) return undefined;
    const kids = b.children.filter((c) => bones.includes(c));
    if (kids.length)
      return kids
        .reduce(
          (sum, k) => sum.add(k.getWorldPosition(new THREE.Vector3())),
          new THREE.Vector3(),
        )
        .divideScalar(kids.length);
    const end = at(s)!;
    const start = at(from);
    return start && end.clone().add(end.clone().sub(start).multiplyScalar(0.3));
  };

  const mixer = new THREE.AnimationMixer(mocap.root);
  mixer.clipAction(clip).play();
  const count = Math.max(1, Math.round(clip.duration * fps));
  const frames: PoseFrame[] = [];
  for (let i = 0; i < count; i++) {
    mixer.setTime(i / fps);
    mocap.root.updateMatrixWorld(true);
    const head = at("head");
    const headTip = tip("head", "neck");
    const frame: PoseFrame = {
      hipL: at("thighL"),
      hipR: at("thighR"),
      kneeL: at("shinL"),
      kneeR: at("shinR"),
      ankleL: at("footL"),
      ankleR: at("footR"),
      toeL: tip("footL", "shinL"),
      toeR: tip("footR", "shinR"),
      shoulderL: at("upperArmL"),
      shoulderR: at("upperArmR"),
      elbowL: at("foreArmL"),
      elbowR: at("foreArmR"),
      wristL: at("handL"),
      wristR: at("handR"),
      handL: tip("handL", "foreArmL"),
      handR: tip("handR", "foreArmR"),
      head:
        head && headTip ? head.clone().add(headTip).multiplyScalar(0.5) : head,
    };
    for (const key of Object.keys(frame) as Point[])
      if (!frame[key]) delete frame[key];
    frames.push(frame);
  }
  mixer.stopAllAction();
  mixer.uncacheRoot(mocap.root);
  return { fps, frames, absolute: true };
}
