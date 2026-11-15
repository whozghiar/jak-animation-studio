// node --experimental-strip-types src/lib/variants.check.ts: the tags of real variants of the games.

import assert from "node:assert/strict";
import { variantTags } from "./variants.ts";

const tags = (names: string[], folders?: string[]) =>
  variantTags(names, folders).map((t) =>
    t.map((tag) => ("kind" in tag ? tag.kind : tag.text)),
  );

assert.deepEqual(tags(["bird-lady-lod0", "bird-lady-beach-lod0"]), [
  ["main"],
  ["beach"],
]);
assert.deepEqual(tags(["babak-lod0", "babak-lod1"]), [["main"], ["LOD 1"]]);
assert.deepEqual(
  tags(["crimson-guard", "crimson-guard-highres", "crimson-guard-lowres"]),
  [["main"], ["highres"], ["lowres"]],
);
// no shared start: each keeps its own name
assert.deepEqual(
  tags(["jak-highres", "jak-highres-prison", "jakb", "jakone-highres"]),
  [
    ["highres", "jak"],
    ["highres", "jak-prison"],
    ["jakb"],
    ["highres", "jakone"],
  ],
);
// Jak 3's arena fighters (bt-sig) are no minigame's: the name says nothing of it
assert.deepEqual(tags(["bt-sig", "sig-highres", "sig-rider"]), [
  ["bt-sig"],
  ["highres", "sig"],
  ["rider", "sig"],
]);
// copies of one name, each with its level's animations (Jak II's grunt-lod0): their level tells them
// apart
assert.deepEqual(tags(["grunt-lod0", "grunt-lod0"], ["atoll", "sewer"]), [
  ["main", "atoll"],
  ["main", "sewer"],
]);
assert.deepEqual(tags(["bird-lady-lod0"], ["beach"]), [["main"]]);
console.log("variants: the tags check out");
