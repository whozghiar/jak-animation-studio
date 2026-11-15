// What the games' code shows of a model: its segments (crates/core looks.rs, game_looks) turned on
// and off by `setup-masks draw on off` (process-drawable.gc: the mask of the hidden segments, `on`
// cleared then `off` set; foreground.gc: a hidden segment's effects are not drawn). Paths are under
// jak-project's goal_src/<game>/; the game's #x.. numbers are written 0x.. .
// - Jak's goatee (Jak II's segment 1, Jak 3's 2): the beard setting (jak2|jak3
//   engine/target/logic-target.gc), on but in the Japanese and Korean versions (settings.gc), the
//   toggle-beard secret turning it off. Jak 3's armor (jak3
//   engine/common-obs/powerups.gc, target-update-segs): segments 1, 4 and 7 one by one, then 9 for
//   8; his scarf over the mouth (segment 6, for 5) once its shape is all the way up
//   (target-draw-process).
// - Metal Heads (engine/ai/enemy.gc): their enemy-info's gem-seg (0x2) shown, gem-no-seg (0x4)
//   hidden; in Jak II also the other way round, an enemy without its gem (enemy-init-by-other,
//   init-from-entity!), in Jak 3 always the gem (enemy-setup-gem makes the same call either way).
// - Characters drawn at random (citizens, wastelanders, Marauders, monks): each `one` is one of the
//   code's draws (rand-vu-int-count, rnd-int-count), a spawn's data or the game's progress.
// Cutscenes set their own masks from the scenes' data (scene.gc 'segment): not listed.

/** From the hidden segments `hide`, a number shows its segments (bit i: segment i), an array
 * plays each of its rules in turn, `one` one of its rules. */
export type Rule = number | Rule[] | { one: Rule[] };

export type Masks = {
  hide: number;
  rule: Rule;
  /** Jak 3's scarf over the mouth: shown when the scarf shape is up, for the other */
  scarf?: { up: number; down: number };
  names?: Record<number, "goatee" | "scarfUp">;
};

/** an enemy-info with gem-seg 0x2 and gem-no-seg 0x4 (Jak II: with or without its gem) */
const GEM2: Masks = { hide: 6, rule: { one: [2, 4] } };
/** Jak 3: always with its gem */
const GEM3: Masks = { hide: 4, rule: 0 };
const ALL: Masks = { hide: 0, rule: 0 };

/** citizen-norm-rider, vehicle-rider.gc:291 (vehicle-rider-method-33) */
const RIDER: Rule = [
  32,
  { one: [0x40000, 0x20000] },
  { one: [2, 4096] },
  { one: [8, 512, 0x4000] },
  { one: [4, 256, 8192, 0] },
  { one: [64, 2048, 0x10000] },
  16,
  { one: [128, 0] },
];

/** citizen-norm, citizen-norm.gc:831 (Jak II) and :1054 (Jak 3), citizen-init! */
const NORM: Rule = [
  32,
  { one: [64, 2048, 0x10000, 0x200000] },
  { one: [2, 8192, 0x40000] },
  { one: [8, 1024, 0x8000, 0x100000] },
  { one: [4, 512, 0x4000, 0x80000, 0] },
  { one: [128, 4096, 0x20000, 0x400000] },
  16,
  { one: [256, 0x800000, 0] },
];

/** citizen-fat's two bodies, citizen-fat.gc:686 (Jak II) and :905 (Jak 3), citizen-init! */
const FAT: Rule = {
  one: [
    [
      8,
      { one: [4, 256, 0x8000] },
      { one: [2, 128, 130, 0] },
      512,
      { one: [64, 4096, 8192] },
      { one: [32, 2048, 0x20000] },
      { one: [16, 1024, 0x10000] },
    ],
    [
      0x400000,
      0x200000,
      { one: [4096, 0x80000] },
      { one: [256, 0x100000] },
      { one: [2, 128, 130, 0] },
      0x20000,
      { one: [16, 1024, 0x10000] },
    ],
  ],
};

/** citizen-chick.gc:652 (Jak II) and :907 (Jak 3), citizen-init! */
const CHICK_TOP = { one: [128, 8192] };
const CHICK: Rule = [
  16,
  8,
  { one: [32, 0x4000, 0x8000] },
  { one: [64, 1024, 0x10000] },
  { one: [4, 512, 4096] },
  { one: [[2, CHICK_TOP], 256, [2048, CHICK_TOP]] },
];

/** a guard: one weapon (8: rifle, 16: tazer and grenade) and one armor (3: the guard, blue; 5: the
 * dark guard, yellow, scaled 1.1), the blue one with a rifle first */
const GUARD: Rule = [{ one: [8, 16] }, { one: [3, 5] }];
const HATS = { one: [40, 2056, 544, 2560] };

const MASKS: Record<string, Record<string, Masks>> = {
  jak2: {
    // city/traffic/citizen/guard.gc:3522 crimson-guard-method-225, crimson-guard-level.gc:4093;
    // 3: crimson-guard-rider without a weapon (city/traffic/vehicle/vehicle-rider.gc:386)
    "crimson-guard": { hide: 30, rule: { one: [GUARD, 3] } },
    jakb: { hide: 2, rule: { one: [2, 0] }, names: { 1: "goatee" } },
    "citizen-chick": { hide: ~0, rule: CHICK },
    "citizen-fat": { hide: ~0, rule: FAT },
    "citizen-norm": { hide: ~0, rule: NORM },
    "citizen-norm-rider": { hide: ~0, rule: RIDER },
    // city/shuttle/shuttle.gc:890 citizen-init!
    "citizen-rebel": {
      hide: ~0,
      rule: [4, 16, 64, { one: [2, 8, 128] }, { one: [32, 256] }],
    },
    // sewer/escort/jinx.gc:331 (2786, the bomb 4096 planted or not), mog.gc:287, grim.gc:94
    jinx: {
      hide: 0x3fffe,
      rule: { one: [[2786, { one: [4096, 0] }], 0x26002, 654] },
    },
    // forest/fish.gc:35 minnow-init-by-other: red, blue, green or yellow
    minnow: { hide: ~0, rule: { one: [4, 1, 2, 8] } },
    // forest/predator.gc:791, metalhead-predator.gc:327: 4 the body, 8 the cloak (cross-faded)
    predator: { hide: 12, rule: { one: [4, 8, 12] } },
    // common/enemy/spyder.gc:1346, :345: the gem, and 8 the body, 16 the cloak
    spyder: { hide: 30, rule: [{ one: [2, 4] }, { one: [8, 16, 24] }] },
    // tomb/tomb-beetle.gc:307: 12, the key beetle
    "tomb-beetle": { hide: ~0, rule: [2, { one: [12, 0] }] },
    // forest/wren.gc:433: flying (8) or pecking (2)
    wren: { hide: ~0, rule: { one: [8, 2] } },
    // ruins/rapid-gunner.gc:209: gem-seg 0x2 without gem-no-seg
    "rapid-gunner": { hide: 2, rule: { one: [2, 0] } },
    // sewer/hosehead-fake.gc:281: the cutscene's fake draws both
    hosehead: { hide: 6, rule: { one: [2, 4, 6] } },
    centurion: GEM2,
    flamer: GEM2,
    flitter: GEM2,
    "flying-spider": GEM2,
    grenadier: GEM2,
    grunt: GEM2,
    hopper: GEM2,
    jellyfish: GEM2,
    juicer: GEM2,
    mantis: GEM2,
    pegasus: GEM2,
    rhino: GEM2,
  },
  jak3: {
    // levels/city/traffic/citizen/guard.gc:1610 crimson-guard-method-267
    "crimson-guard": { hide: 30, rule: GUARD },
    // armor from all of it to none, with or without the goatee
    "jakb-c": {
      hide: 982,
      rule: [{ one: [658, 402, 274, 258, 256] }, { one: [4, 0] }],
      scarf: { up: 64, down: 32 },
      names: { 2: "goatee", 6: "scarfUp" },
    },
    "citizen-chick": { hide: ~0, rule: CHICK },
    "citizen-fat": { hide: ~0, rule: FAT },
    "citizen-norm": { hide: ~0, rule: NORM },
    "citizen-norm-rider": { hide: ~0, rule: RIDER },
    // levels/desert/chase/marauder.gc:1871: a body, one head of three with one of its hats, a
    // belt, sleeves, and a weapon (4096: club, 8192: gun)
    "marauder-male": {
      hide: ~0,
      rule: [
        64,
        { one: [[2, HATS], [128, { one: [40, 544] }], HATS] },
        { one: [16, 1024] },
        { one: [4, 256] },
        { one: [4096, 8192] },
      ],
    },
    // levels/wascity/wlander-male.gc:1624 citizen-method-194
    "wlander-male": {
      hide: ~0,
      rule: [
        { one: [16, 256, 0x200000] },
        { one: [[4, { one: [32, 0] }], 4096, 0x40000] },
        { one: [[2, { one: [2048, 0x10000, 0] }], 0x20000] },
        { one: [8, 128, 0x100000] },
        { one: [512, 0x4000, 0x400000] },
        { one: [1024, 0x8000, 0x1000000, 0] },
        { one: [0x800000, 0] },
        { one: [64, 0x80000, 0] },
        { one: [0x2000000, 0x4000000] },
      ],
    },
    // levels/wascity/wlander-male.gc:2543 citizen-method-194 (wlander-female)
    "wlander-female": {
      hide: ~0,
      rule: [
        288,
        { one: [2, 512] },
        { one: [4, 1024] },
        { one: [8, 2048] },
        { one: [16, 4096] },
        { one: [64, 8192] },
        { one: [128, 0x4000] },
        { one: [0x8000, 0x10000] },
      ],
    },
    // levels/wascity/waswide-obs.gc:827, engine/target/flut/flut-racer.gc:637: six monks
    monk: {
      hide: ~0,
      rule: { one: [4171, 8357, 0x4113, 0x8225, 0x10407, 0x20825] },
    },
    // metalhead-predator.gc:1125, :353: the cloak (8), the body (4), or both while fading
    predator: { hide: 12, rule: { one: [8, 4, 12] } },
    // levels/common/enemy/flitter.gc:550 (the gem); Vin's power-game-chaser shows everything
    flitter: { hide: 4, rule: { one: [0, 4] } },
    // levels/volcano/flamer-lava.gc:192; temple's flamer-hover shows everything
    "flamer-lava": { hide: 4, rule: { one: [0, 4] } },
    // levels/common/enemy/prebot-eco-creature.gc:2567, :3361: the medium and small ones hide 2
    "cav-eco-lg": { hide: 2, rule: { one: [2, 0] } },
    grunt: GEM3,
    "city-grunt": GEM3,
    "city-flitter": GEM3,
    mantis: GEM3,
    "neo-grenadier": GEM3,
    "neo-juicer": GEM3,
    spyder: GEM3,
    saberfish: ALL,
  },
};

/** What the game's code shows of a model, when its code sets masks. */
export const masksOf = (game: string, model: string): Masks | undefined =>
  MASKS[game]?.[model.replace(/-lod\d+$/, "")];

/** Every look `masks` can give, as hidden segments, in the code's order: each choice's first
 * branch first. */
export function outfits(masks: Masks): number[] {
  const all = (rule: Rule, hidden: number[]): number[] => {
    if (typeof rule === "number") return hidden.map((h) => h & ~rule);
    if (Array.isArray(rule))
      return rule.reduce<number[]>((hs, r) => all(r, hs), hidden);
    return rule.one.flatMap((r) => all(r, hidden));
  };
  return [...new Set(all(masks.rule, [masks.hide]))];
}
