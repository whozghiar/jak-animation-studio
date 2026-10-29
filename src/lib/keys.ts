// The animation editor's keys: rpc's Channels (one per joint and path, times going up), changed
// key by key. Keys sit on frames of 1/30 s.

import type { Channel } from "./rpc";

/** what the editor starts from: nothing, a video's result, or one of the model's animations */
export type EditStart = { name: string; channels: Channel[]; duration: number };

export const FPS = 30;
const same = (a: number, b: number) => Math.abs(a - b) < 1e-4;
const size = (path: Channel["path"]) => (path === "rotation" ? 4 : 3);

/** `time` on the nearest frame. */
export const snap = (time: number) => Math.max(0, Math.round(time * FPS) / FPS);

/** `channels` with the key of `joint`'s `path` at `time` set to `value` (added, or replaced). */
export function withKey(
  channels: Channel[],
  joint: number,
  path: Channel["path"],
  time: number,
  value: number[],
): Channel[] {
  const n = size(path);
  const old = channels.find((c) => c.joint === joint && c.path === path);
  const c = old
    ? { ...old, times: [...old.times], values: [...old.values] }
    : { joint, path, times: [], values: [] };
  const i = c.times.findIndex((t) => same(t, time) || t > time);
  if (i >= 0 && same(c.times[i], time)) c.values.splice(i * n, n, ...value);
  else {
    const at = i < 0 ? c.times.length : i;
    c.times.splice(at, 0, time);
    c.values.splice(at * n, 0, ...value);
  }
  return [...channels.filter((x) => x !== old), c];
}

/** `channels` without `joint`'s keys at `time`. */
export function withoutKey(
  channels: Channel[],
  joint: number,
  time: number,
): Channel[] {
  return channels.flatMap((c) => {
    const i = c.joint === joint ? c.times.findIndex((t) => same(t, time)) : -1;
    if (i < 0) return [c];
    if (c.times.length === 1) return [];
    const n = size(c.path);
    return [
      {
        ...c,
        times: c.times.filter((_, k) => k !== i),
        values: [...c.values.slice(0, i * n), ...c.values.slice((i + 1) * n)],
      },
    ];
  });
}

/** The keys as they are saved: each rotation on the short way from the key before (the game blends
 * key to key, and q and -q are the same turn), every channel held up to `duration`. */
export function finished(channels: Channel[], duration: number): Channel[] {
  return channels.map((c) => {
    const n = size(c.path);
    const times = [...c.times];
    const values = [...c.values];
    if (c.path === "rotation")
      for (let k = 4; k < values.length; k += 4) {
        let dot = 0;
        for (let j = 0; j < 4; j++) dot += values[k + j] * values[k - 4 + j];
        if (dot < 0) for (let j = 0; j < 4; j++) values[k + j] = -values[k + j];
      }
    if (times[times.length - 1] < duration - 1e-4) {
      times.push(duration);
      values.push(...values.slice(-n));
    }
    return { ...c, times, values };
  });
}
