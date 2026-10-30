// The variants of one entity: the game cuts a character into several models, each with only the
// animations one place or use needs (a level, the cutscenes, a vehicle, a minigame). They share its
// official name; their tags tell them apart.

export type Tag =
  | {
      kind:
        | "cutscene"
        | "lowres"
        | "simple"
        | "rider"
        | "escort"
        | "minigame"
        | "main";
    }
  | { text: string };

/** words of the art groups' names that say what a variant is for */
const KINDS: Record<string, Exclude<Tag, { text: string }>["kind"]> = {
  highres: "cutscene",
  fma: "cutscene",
  lowres: "lowres",
  simple: "simple",
  rider: "rider",
  escort: "escort",
  bt: "minigame",
};

/** The tags of each variant (file names without .glb), in their order: what its name says of its
 * use, and the part of its name the others do not share ("beach" for bird-lady-beach beside
 * bird-lady, which is the main one). Copies of one name, each with the animations of the level it
 * comes from, also show that level (`folders`, the folder of each). */
export function variantTags(names: string[], folders: string[] = []): Tag[][] {
  const words = names.map((n) => n.replace(/-lod\d+$/, "").split("-"));
  let common = 0;
  while (
    words.every((w) => w.length > common && w[common] === words[0][common])
  )
    common++;
  return names.map((name, i) => {
    const tags: Tag[] = [...new Set(words[i].map((w) => KINDS[w]))]
      .filter((kind) => kind)
      .map((kind) => ({ kind }));
    const own = words[i]
      .slice(common)
      .filter((w) => !(w in KINDS))
      .join("-");
    if (own) tags.push({ text: own });
    const lod = /-lod([1-9]\d*)$/.exec(name);
    if (lod) tags.push({ text: `LOD ${lod[1]}` });
    if (!tags.length) tags.push({ kind: "main" });
    if (folders[i] && names.indexOf(name) !== names.lastIndexOf(name))
      tags.push({ text: folders[i] });
    return tags;
  });
}
