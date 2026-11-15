// node --experimental-strip-types src/lib/masks.check.ts: the looks the games' code gives a model.

import assert from "node:assert/strict";
import { masksOf, outfits } from "./masks.ts";

const looks = (game: string, model: string) => outfits(masksOf(game, model)!);

// Jak 3's guard: segments 1 to 4 hidden, then a weapon (8 or 16) and an armor (3 or 5) shown: blue
// with a rifle or a baton, then yellow
assert.deepEqual(looks("jak3", "crimson-guard-lod0"), [20, 12, 18, 10]);
// Jak 3's Jak: his armor stages with the goatee (target-update-segs, the beard setting), from all
// of it to none, then the same without it
const jak = looks("jak3", "jakb-c-lod0");
assert.equal(jak.length, 10);
assert.equal(jak[0], 324 & ~4);
assert.equal(jak[4], 726 & ~4);
assert.equal(jak[5], 324);
// the same look twice counts once (Jak II's dark guard with or without its rider's weapon)
assert.equal(new Set(looks("jak2", "crimson-guard-lod0")).size, 5);
assert.equal(looks("jak3", "marauder-male-lod0").length, 80);
assert.equal(masksOf("jak1", "babak-lod0"), undefined);
console.log("masks: the outfits check out");
