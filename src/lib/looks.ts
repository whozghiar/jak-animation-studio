// What the games do to Jak's looks, from jak-project's goal_src (a blend target t is the game's
// `skel override` t + 1, merc-blend-shape.gc):
// - jak2/engine/target/target-darkjak.gc: Dark Jak sets overrides 1 to 5, blend targets 0 to 4.
// - jak3/engine/common-obs/powerups.gc (target-draw-process): Dark Jak sets overrides 2, 4, 6 and 7,
//   the goggles 5, the scarf 8.
// - jak3/engine/target/lightjak-wings.gc (wings-init): Light Jak's wings, the model
//   jakb-wings-wings, follow Jak's node 6, the joint chest; their joint wingmain carries them.
// - jak2|jak3/engine/target/target-gun.gc (target-gun-compute-pos): the Morph Gun, the model gun,
//   is put on the matrix of Jak's joint gun (node 36 in Jak 3, joint 35 and the root before it);
//   each mod's still pose (gun-idle-red, -yellow, -blue, -dark) folds the other barrels away.

export type Known = {
  dark: number[];
  shapes?: Record<number, "goggles" | "scarf">;
  wings?: { model: string; joint: string; anchor: string };
  gun?: { model: string; joint: string };
  /** the joints target-darkjak-process scales every frame, Dark Jak or not (joint numbers: the
   * waist, the neck, the horns), and the neck's own scale (Jak 3's jakc: 0.95 wide, 0.92 high) */
  scaled: {
    waist: number;
    neck: number;
    horns: number;
    head: [number, number];
  };
};

const GUN = { model: "gun-lod0", joint: "gun" };

/** the Morph Gun's mods, by the still pose that shows each */
export const GUN_MODS = ["red", "yellow", "blue", "dark"] as const;

const KNOWN: Record<string, Record<string, Known>> = {
  jak2: {
    jakb: {
      dark: [0, 1, 2, 3, 4],
      gun: GUN,
      scaled: { waist: 3, neck: 7, horns: 41, head: [1, 1] },
    },
  },
  jak3: {
    "jakb-c": {
      dark: [1, 3, 5, 6],
      shapes: { 4: "goggles", 7: "scarf" },
      wings: {
        model: "jakb-wings-wings-lod0",
        joint: "chest",
        anchor: "wingmain",
      },
      gun: GUN,
      scaled: { waist: 3, neck: 7, horns: 14, head: [0.95, 0.92] },
    },
  },
};

/** What the game does to this model, when it is one of Jak's. */
export const known = (game: string, model: string) =>
  KNOWN[game]?.[model.replace(/-lod\d+$/, "")];

/** A part whose textures are armor pieces: Jak 3's chest plate, shoulder guards, gauntlets, greaves. */
export const isArmor = (materials: string[]) =>
  materials.some((m) =>
    /chestplate|shoulderarmor|forearm-arm|shoeplate|precarmor/.test(m),
  );

/** A material's name without the model's prefix and the renderer's suffix: "chestplate". */
export const pieceName = (material: string) =>
  material
    .replace(/^(jakc|jakbsmall|jakchires|jakb)-/, "")
    .replace(/-envmap$/, "") || "?";

/** The scales target-darkjak-process (jak2|jak3 engine/target/target-darkjak.gc) puts on Jak's
 * joints every frame at Dark Jak's `d` (0 to 1), by joint number, -1 for the whole of him: him
 * 1.05 times, his waist 1.2 times wider (upper-body), his neck its own scale over the waist's
 * width (so normal Jak's head is 1 / 1.1 as wide as the model's), his horns `d` (none for normal
 * Jak). */
export function jakScales(
  k: Known,
  d: number,
): [number, [number, number, number]][] {
  const { waist, neck, horns, head } = k.scaled;
  const body = 1 + 0.05 * d;
  const wide = 1 + 0.2 * d;
  const x = head[0] / (1.1 * wide);
  return [
    [-1, [body, body, body]],
    [waist, [wide, 1, wide]],
    [neck, [x, head[1], x]],
    [horns, [d, d, d]],
  ];
}
