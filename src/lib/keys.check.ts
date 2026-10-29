// node --experimental-strip-types src/lib/keys.check.ts: the editor's key operations.

import assert from "node:assert/strict";
import { finished, snap, withKey, withoutKey } from "./keys.ts";

const q = (w: number) => [0, 0, Math.sqrt(1 - w * w), w];

let keys = withKey([], 3, "rotation", 0.5, q(1));
keys = withKey(keys, 3, "rotation", 0, q(0.8));
keys = withKey(keys, 3, "rotation", 0.5, q(0.6)); // replaces the key at 0.5
keys = withKey(keys, 3, "translation", 0.2, [1, 2, 3]);
const rot = keys.find((c) => c.path === "rotation")!;
assert.deepEqual(rot.times, [0, 0.5], "times go up, one key per time");
assert.deepEqual(rot.values, [...q(0.8), ...q(0.6)]);

keys = withoutKey(keys, 3, 0.2);
assert.equal(keys.length, 1, "a channel without keys is gone");

// a key on the long way round is turned to the short way; the last key is held to the end
const negated = q(0.6).map((v) => -v);
const [saved] = finished(withKey(keys, 3, "rotation", 1, negated), 2);
assert.deepEqual(saved.times, [0, 0.5, 1, 2]);
assert.deepEqual(saved.values.slice(8, 12), q(0.6));
assert.deepEqual(saved.values.slice(12), saved.values.slice(8, 12));

assert.equal(snap(0.51), 0.5);
assert.equal(snap(-1), 0);
console.log("keys: the key operations check out");
