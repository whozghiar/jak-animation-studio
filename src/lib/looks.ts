// What the games do to Jak's looks, from jak-project's goal_src (a blend target t is the game's
// `skel override` t + 1, merc-blend-shape.gc):
// - jak2/engine/target/target-darkjak.gc: Dark Jak sets overrides 1 to 5, blend targets 0 to 4.
// - jak3/engine/common-obs/powerups.gc (target-draw-process): Dark Jak sets overrides 2, 4, 6 and 7,
//   the goggles 5, the scarf 8.
// - jak3/engine/target/lightjak-wings.gc (wings-init): Light Jak's wings, the model
//   jakb-wings-wings, follow Jak's node 6, the joint chest; their joint wingmain carries them.

export type Known = {
  dark: number[];
  shapes?: Record<number, "goggles" | "scarf">;
  wings?: { model: string; joint: string; anchor: string };
};

const KNOWN: Record<string, Record<string, Known>> = {
  jak2: { jakb: { dark: [0, 1, 2, 3, 4] } },
  jak3: {
    "jakb-c": {
      dark: [1, 3, 5, 6],
      shapes: { 4: "goggles", 7: "scarf" },
      wings: {
        model: "jakb-wings-wings-lod0",
        joint: "chest",
        anchor: "wingmain",
      },
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
