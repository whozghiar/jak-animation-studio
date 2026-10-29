// Which joint of a skeleton plays which part of a body, guessed from the joint names: Jak's rigs
// (Larm, Lforarm, sk_lhand, Lthigh, Lknee...), Mixamo and BVH (LeftArm, LeftUpLeg...), Blender's
// (upper_arm.L, thigh.L...). The user corrects what the guess misses.

export const SLOTS = [
  "hips",
  "chest",
  "neck",
  "head",
  "upperArmL",
  "foreArmL",
  "handL",
  "upperArmR",
  "foreArmR",
  "handR",
  "thighL",
  "shinL",
  "footL",
  "thighR",
  "shinR",
  "footR",
] as const;

export type Slot = (typeof SLOTS)[number];
/** slot -> joint index */
export type Mapping = Partial<Record<Slot, number>>;

type Bone = { name: string; parent: number | null };

const IGNORED =
  /strap|pad|hair|pants|shirt|twist|roll|nub|end$|_end|helper|pole|(^|[^a-z])ik([^a-z]|$)/i;
const LIMB =
  /^(thigh|knee|ankle|ball|toe|foot|leg|upleg|calf|shin|arm|forarm|forearm|hand|elbow|should|clav|wrist|hip|collar)/;

/** The side of a joint (L, R or none) and its name without it, lowercase. */
function side(name: string): ["L" | "R" | null, string] {
  const n = name
    .replace(/^.*[:|]/, "")
    .replace(/^(sk_|bip0?1[ _]?|def[-_]|b_)/i, "");
  const lower = n.toLowerCase();
  const cut = (re: RegExp) => lower.replace(re, "");
  if (/^left/.test(lower)) return ["L", cut(/^left[._ -]?/)];
  if (/^right/.test(lower)) return ["R", cut(/^right[._ -]?/)];
  if (/([._ -]l|left)$/.test(lower)) return ["L", cut(/([._ -]l|left)$/)];
  if (/([._ -]r|right)$/.test(lower)) return ["R", cut(/([._ -]r|right)$/)];
  if (/^l[._ -]/.test(lower)) return ["L", lower.slice(2)];
  if (/^r[._ -]/.test(lower)) return ["R", lower.slice(2)];
  const glued = lower.match(/^([lr])(.+)$/);
  if (glued && LIMB.test(glued[2]))
    return [glued[1].toUpperCase() as "L" | "R", glued[2]];
  return [null, lower];
}

const LIMBS: [string, RegExp[]][] = [
  ["thigh", [/^(thigh|upleg|upperleg|upper_leg|femur)\d*$/, /^hip\d*$/]],
  ["shin", [/^(knee|shin|calf|lowerleg|lower_leg|leg)\d*$/]],
  ["foot", [/^(ankle|foot)\d*$/]],
  ["upperArm", [/^(upperarm|upper_arm|uparm|arm)\d*$/, /^shoulder\d*$/]],
  ["foreArm", [/^(forearm|forarm|fore_arm|lowerarm|lower_arm|elbow)\d*$/]],
  ["hand", [/^(hand|wrist)\d*$/]],
];

const CENTER: [Slot, RegExp[]][] = [
  ["hips", [/^(hips?|pelvis)$/]],
  [
    "chest",
    [
      /^chest$/,
      /^upper_?chest$/,
      /^spine_?0?2$/,
      /^upper_?body$/,
      /^spine_?0?1$/,
    ],
  ],
  ["neck", [/^neck/]],
  ["head", [/^head$/, /^head/]],
];

export function autoMap(bones: Bone[]): Mapping {
  const parsed = bones.map((b) => (IGNORED.test(b.name) ? null : side(b.name)));
  const find = (wantSide: "L" | "R" | null, rules: RegExp[]) => {
    for (const re of rules) {
      const i = parsed.findIndex(
        (p) => p !== null && p[0] === wantSide && re.test(p[1]),
      );
      if (i >= 0) return i;
    }
    return undefined;
  };
  const map: Mapping = {};
  for (const [slot, rules] of CENTER) map[slot] = find(null, rules);
  for (const s of ["L", "R"] as const) {
    for (const [part, rules] of LIMBS) {
      map[`${part}${s}` as Slot] = find(s, rules);
    }
  }
  // the head of a rig without one is the end of its neck (Jak's neckA, neckB)
  if (map.head === undefined && map.neck !== undefined) {
    let end = map.neck;
    for (;;) {
      const next = bones.findIndex(
        (b, i) => b.parent === end && /^neck/i.test(bones[i].name),
      );
      if (next < 0) break;
      end = next;
    }
    if (end !== map.neck) map.head = end;
  }
  const parentOf = (i: number | undefined) =>
    i === undefined ? undefined : (bones[i].parent ?? undefined);
  map.hips ??= parentOf(map.thighL);
  map.chest ??= parentOf(map.neck);
  return map;
}
