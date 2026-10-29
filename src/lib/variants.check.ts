// node --experimental-strip-types src/lib/variants.check.ts: the tags of real variants of the games.

import assert from "node:assert/strict";
import { variantTags } from "./variants.ts";

const tags = (names: string[]) =>
  variantTags(names).map((t) =>
    t.map((tag) => ("kind" in tag ? tag.kind : tag.text)),
  );

assert.deepEqual(tags(["bird-lady-lod0", "bird-lady-beach-lod0"]), [
  ["main"],
  ["beach"],
]);
assert.deepEqual(tags(["babak-lod0", "babak-lod1"]), [["main"], ["LOD 1"]]);
assert.deepEqual(
  tags(["crimson-guard", "crimson-guard-highres", "crimson-guard-lowres"]),
  [["main"], ["cutscene"], ["lowres"]],
);
// no shared start: each keeps its own name
assert.deepEqual(
  tags(["jak-highres", "jak-highres-prison", "jakb", "jakone-highres"]),
  [
    ["cutscene", "jak"],
    ["cutscene", "jak-prison"],
    ["jakb"],
    ["cutscene", "jakone"],
  ],
);
assert.deepEqual(tags(["bt-sig", "sig-highres", "sig-rider"]), [
  ["minigame", "sig"],
  ["cutscene", "sig"],
  ["rider", "sig"],
]);
console.log("variants: the tags check out");
