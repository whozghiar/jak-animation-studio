<script lang="ts">
  import IconChevron from "~icons/mdi/chevron-down";
  import {
    findModel,
    gameTextures,
    modelEyeFrames,
    modelEyes,
    modelLooks,
    readModel,
    type EyeFrame,
    type Game,
    type Looks,
    type ModelInfo,
    type Part,
  } from "$lib/rpc";
  import { GUN_MODS, isArmor, jakScales, known, pieceName } from "$lib/looks";
  import { masksOf, outfits } from "$lib/masks";
  import { blend, eye, pixels } from "$lib/textures";
  import type { Viewer } from "$lib/viewer";
  import { errorText, t } from "$lib/i18n.svelte";
  import { toast } from "$lib/toast.svelte";
  import { helpOn } from "$lib/help.svelte";

  let {
    viewer,
    model,
    game,
  }: {
    viewer: Viewer;
    model: ModelInfo;
    game: Game;
  } = $props();

  let looks = $state.raw<Looks | null>(null);
  let hidden = $state.raw(new Set<string>());
  let weights = $state.raw<number[]>([]);
  let light = $state(false);
  /** Light Jak's wings: out only when he glides (lightjak-stage swoop, target-lightjak.gc) */
  let wings = $state(false);
  let gun = $state(false);
  let mod = $state<(typeof GUN_MODS)[number]>("red");
  let noTextures = $state(false);
  /** the game's textures of the eyes, and of Dark Jak's skin (its normal and dark ones) */
  /** each eye's iris, pupil and lid (Krew's differ) */
  let eyes:
    { iris: ImageData; pupil: ImageData; lid: ImageData | null }[] | null =
    null;
  /** each animation's eye frames, by name (looks.rs eye_frames) */
  let eyeFrames: Record<string, [EyeFrame, EyeFrame][]> = {};
  let skins: { name: string; norm: ImageData; dark: ImageData }[] = [];
  /** Dark Jak's skin at the darkness shown, and the eye frame drawn */
  let skinTextures = new Map<string, ImageData>();
  let painted = "";
  let showParts = $state(false);
  /** the game's hidden segments, bit by bit */
  let segs = $state(0);
  /** the look of the game's shown: its number in `list` */
  let outfit = $state(0);
  let showShapes = $state(false);

  const jak = $derived(known(game, model.name));
  const armor = $derived(
    looks?.parts.filter((p) => isArmor(p.materials)) ?? [],
  );
  const isDark = $derived(
    !!jak && jak.dark.every((i) => (weights[i] ?? 0) >= 1),
  );
  const keys = (p: Part) => p.primitives.map((i) => `${p.mesh}:${i}`);
  const shown = (p: Part) => keys(p).every((k) => !hidden.has(k));
  const segMaterials = (l: Looks) =>
    l.segments.map((parts) => parts.flatMap((p) => l.parts[p].materials));
  const masks = $derived(
    looks?.segments.length ? masksOf(game, model.name) : undefined,
  );
  /** every look the game's code gives the model, as hidden segments */
  const list = $derived(masks ? outfits(masks) : []);
  const darkness = $derived(
    jak
      ? jak.dark.reduce((sum, i) => sum + (weights[i] ?? 0), 0) /
          jak.dark.length
      : 0,
  );

  $effect(() => {
    const path = model.path;
    // the viewer drops what the model before carried
    gun = false;
    eyes = null;
    skins = [];
    eyeFrames = {};
    viewer.onFrame = paintEyes;
    modelLooks(path, game)
      .then(async (l) => {
        looks = l;
        weights = Array(l.targets).fill(0);
        outfit = 0;
        hideSegments(list[0] ?? 0);
        eyeFrames = await modelEyeFrames(path, game);
        return loadTextures(l);
      })
      .catch((e) => toast(errorText(e), "error"));
    return () => (viewer.onFrame = undefined);
  });

  /** The model's eyes are drawn by the game (its materials are only "programmer_eye" stand-ins):
   * with the textures its art group's eye control names (Jak's bam-iris-16x16, Kleever's
   * klever-eye), else, when it cannot be read, those of the page of its level: `<prefix>-eye` or
   * `<prefix>-iris...`, else common's autoeye ones. */
  async function loadTextures(l: Looks) {
    const own = [
      ...new Set(
        l.parts
          .flatMap((p) => p.materials)
          .map((m) => m.replace(/-envmap$/, "")),
      ),
    ];
    const beside = own.find((m) => !/^(programmer_eye|common-)/.test(m));
    if (!beside) return;
    const files = new Map<string, string>();
    for (const f of await gameTextures(game, model.path, beside)) {
      const name = f
        .split(/[\\/]/)
        .pop()!
        .replace(/\.png$/, "");
      if (!files.has(name)) files.set(name, f);
    }
    const prefix = beside.split("-")[0];
    const names = [...files.keys()];
    const iris =
      names.find(
        (n) =>
          n.startsWith(`${prefix}-`) &&
          /-(eye|iris)(-|$)/.test(n.slice(prefix.length)),
      ) ?? "autoeye-iris";
    const pupil = files.has(`${prefix}-pupil`)
      ? `${prefix}-pupil`
      : "autoeye-pupil";
    const hasEyes = own.some((m) => m.startsWith("programmer_eye"));
    noTextures = (hasEyes || !!jak) && !files.size;
    const told = hasEyes ? await modelEyes(model.path, game) : null;
    if (told)
      eyes = await Promise.all(
        told.map(async ([i, p, l]) => ({
          iris: await pixels(i),
          pupil: await pixels(p),
          lid: await pixels(l).catch(() => null),
        })),
      );
    else if (hasEyes && files.has(iris) && files.has(pupil)) {
      const e = {
        iris: await pixels(files.get(iris)!),
        pupil: await pixels(files.get(pupil)!),
        lid: null,
      };
      eyes = [e, e];
    }
    if (jak)
      for (const name of own) {
        const [norm, dark] = [`${name}-norm`, `${name}-dark`].map((n) =>
          files.get(n),
        );
        if (norm && dark)
          skins.push({
            name,
            norm: await pixels(norm),
            dark: await pixels(dark),
          });
      }
    show();
  }

  function show() {
    viewer.setLook(hidden, weights);
    skinTextures = new Map();
    if (darkness > 0)
      for (const s of skins)
        if (s.norm.width === s.dark.width && s.norm.height === s.dark.height)
          skinTextures.set(s.name, blend(s.norm, s.dark, darkness));
    viewer.setJointScales(jak ? jakScales(jak, darkness) : []);
    painted = "";
    paintEyes(viewer.clipName, viewer.time, viewer.duration);
    viewer.setLit(light);
  }

  /** eye.gc's eyes without eye data: ahead, the lid open */
  const AHEAD: EyeFrame = [0, 0, -1, 0.55, 0.45, 1];
  /** Dark Jak's: skel override 41 on, 46 and 52 (the pupils) 2, the rest 0 (target-darkjak.gc,
   * powerups.gc target-draw-process): all black */
  const DARK: EyeFrame = [0, 0, 0, 0, 2, 0];

  /** The eyes at the animation's frame, as the game draws them, with Dark Jak's skin. */
  function paintEyes(clip: string | undefined, time: number, duration: number) {
    let frame: [EyeFrame, EyeFrame] = [AHEAD, AHEAD];
    const frames = clip ? eyeFrames[clip] : undefined;
    if (darkness > 0) frame = [DARK, DARK];
    else if (frames?.length)
      frame =
        frames[
          duration > 0
            ? Math.min(
                frames.length - 1,
                Math.round((time / duration) * (frames.length - 1)),
              )
            : 0
        ];
    const key = JSON.stringify(frame);
    if (key === painted) return;
    painted = key;
    const textures = new Map(skinTextures);
    if (eyes) {
      const [left, right] = eyes.map((e, k) =>
        eye(e.iris, e.pupil, e.lid, frame[k], frame[0][5]),
      );
      textures
        .set("programmer_eye_left", left)
        .set("programmer_eye_right", right);
    }
    viewer.setTextures(textures);
  }

  function setParts(parts: Part[], on: boolean) {
    const next = new Set(hidden);
    for (const k of parts.flatMap(keys)) {
      if (on) next.delete(k);
      else next.add(k);
    }
    hidden = next;
    show();
  }

  function setOutfit(n: number) {
    outfit = n;
    setSegments(list[n] ?? 0);
  }

  function hideSegments(m: number) {
    segs = m;
    const parts = looks?.parts ?? [];
    hidden = new Set(
      (looks?.segments ?? []).flatMap((ps, s) =>
        (m >> s) & 1 ? ps.flatMap((p) => keys(parts[p])) : [],
      ),
    );
  }

  function setSegments(m: number) {
    hideSegments(m);
    show();
  }

  function setWeight(target: number, w: number) {
    weights = weights.map((x, i) => (i === target ? w : x));
    // the game swaps the scarves once the scarf is up (powerups.gc)
    if (masks?.scarf && jak?.shapes?.[target] === "scarf") {
      const { up, down } = masks.scarf;
      hideSegments(w >= 1 ? (segs & ~up) | down : (segs & ~down) | up);
    }
    show();
  }

  function setDark(on: boolean) {
    weights = weights.map((x, i) => (jak?.dark.includes(i) ? +on : x));
    show();
  }

  async function setWings(on: boolean) {
    if (!on || !jak?.wings) return viewer.setCarried("wings", null);
    try {
      const path = await findModel(model.path, game, jak.wings.model);
      if (!path) throw new Error(t("noWings", { name: jak.wings.model }));
      await viewer.setCarried(
        "wings",
        await readModel(path),
        jak.wings.joint,
        jak.wings.anchor,
        true,
      );
    } catch (e) {
      toast(errorText(e), "error");
    }
  }

  /** the Morph Gun in Jak's hand */
  async function setGun(on: boolean) {
    gun = on;
    if (!on || !jak?.gun) return viewer.setCarried("gun", null);
    try {
      const path = await findModel(model.path, game, jak.gun.model);
      if (!path) throw new Error(t("noWings", { name: jak.gun.model }));
      await viewer.setCarried(
        "gun",
        await readModel(path),
        jak.gun.joint,
        "",
        false,
        `gun-idle-${mod}`,
      );
    } catch (e) {
      gun = false;
      toast(errorText(e), "error");
    }
  }

  /** Light Jak: no textures under the rim lights (powerups.gc); his wings only when he glides */
  function preset(look: "jak" | "dark" | "light") {
    light = look === "light";
    setDark(look === "dark");
    if (!light) {
      wings = false;
      setWings(false);
    }
  }

  const shapeName = (i: number) =>
    jak?.dark.includes(i)
      ? t("shapeDark", { n: jak.dark.indexOf(i) + 1 })
      : jak?.shapes?.[i]
        ? t(`shape_${jak.shapes[i]}`)
        : t("shape", { n: i });
</script>

{#if looks && (looks.parts.length > 1 || looks.targets > 0)}
  <section class="flex flex-col gap-2">
    <h3 class="text-sm font-semibold text-neutral-200">{t("looks")}</h3>
    {#if list.length > 1}
      <p class="text-xs text-neutral-400">{t("looksHint")}</p>
    {/if}
    {#if noTextures}
      <p class="text-xs text-amber-300">{t("noGameTextures")}</p>
    {/if}

    {#if jak}
      <div class="grid grid-cols-3 gap-2">
        {#each [["jak", "lookJak", !isDark && !light], ["dark", "lookDark", isDark], ["light", "lookLight", light]] as const as [id, label, active] (id)}
          {#if id === "jak" || (id === "dark" ? looks.targets > 0 : jak.wings)}
            <button
              use:helpOn={{ note: "ctl_forms", section: "looks" }}
              class="rounded-md border px-2 py-1.5 text-sm {active
                ? 'border-orange-500 bg-orange-500/10'
                : 'border-white/10 hover:bg-white/5'}"
              onclick={() => preset(id)}>{t(label)}</button
            >
          {/if}
        {/each}
      </div>
    {/if}
    {#if light && jak?.wings}
      <label class="flex items-center gap-2 text-sm">
        <input
          use:helpOn={{
            note: "ctl_wings",
            section: "looks",
            label: t("lightWings"),
          }}
          type="checkbox"
          class="rounded border-white/20 bg-[#1b1b1b] text-orange-500 focus:ring-0"
          bind:checked={wings}
          onchange={() => setWings(wings)}
        />
        <span class="text-neutral-300">{t("lightWings")}</span>
      </label>
    {/if}
    {#if jak?.gun}
      <label class="flex items-center gap-2 text-sm">
        <input
          use:helpOn={{
            note: "ctl_gun",
            section: "looks",
            label: t("morphGun"),
          }}
          type="checkbox"
          class="rounded border-white/20 bg-[#1b1b1b] text-orange-500 focus:ring-0"
          checked={gun}
          onchange={(e) => setGun(e.currentTarget.checked)}
        />
        <span class="flex-1 text-neutral-300">{t("morphGun")}</span>
        {#if gun}
          <select
            class="rounded border border-white/10 bg-[#1b1b1b] px-2 py-0.5 text-xs"
            bind:value={mod}
            onchange={() => setGun(true)}
          >
            {#each GUN_MODS as m (m)}
              <option value={m}>{t(`gunMod_${m}`)}</option>
            {/each}
          </select>
        {/if}
      </label>
    {/if}
    {#if armor.length && !looks.segments.length}
      <label class="flex items-center gap-2 text-sm">
        <input
          use:helpOn={{
            note: "ctl_armor",
            section: "looks",
            label: t("armor"),
          }}
          type="checkbox"
          class="rounded border-white/20 bg-[#1b1b1b] text-orange-500 focus:ring-0"
          checked={armor.every(shown)}
          onchange={(e) => setParts(armor, e.currentTarget.checked)}
        />
        <span class="text-neutral-300">{t("armor")}</span>
      </label>
    {/if}
    {#if list.length > 1}
      <div class="flex items-center gap-2 text-sm">
        <select
          use:helpOn={{
            note: "ctl_outfit",
            section: "looks",
            label: t("outfit", { n: outfit + 1 }),
          }}
          class="min-w-0 flex-1 rounded border border-white/10 bg-[#1b1b1b] px-2 py-1 text-sm"
          value={outfit}
          onchange={(e) => setOutfit(+e.currentTarget.value)}
        >
          {#each list as _, i (i)}
            <option value={i}>{t("outfit", { n: i + 1 })}</option>
          {/each}
        </select>
        <span class="shrink-0 text-xs text-neutral-400"
          >{t("outfits", { n: list.length })}</span
        >
        <button
          use:helpOn={{ note: "ctl_outfit", section: "looks" }}
          class="shrink-0 rounded-md border border-white/10 px-2 py-1 text-xs hover:bg-white/5"
          onclick={() => setOutfit(Math.floor(Math.random() * list.length))}
          >{t("outfitRandom")}</button
        >
      </div>
    {/if}

    {#if looks.segments.length}
      <button
        use:helpOn={{ note: "ctl_segments", section: "looks" }}
        class="flex items-center justify-between text-xs text-neutral-300 hover:text-neutral-100"
        onclick={() => (showParts = !showParts)}
      >
        {t("segments", { n: looks.segments.length })}
        <IconChevron class={showParts ? "rotate-180" : ""} />
      </button>
      {#if showParts}
        <ul class="flex flex-col gap-1">
          {#each segMaterials(looks) as materials, s (s)}
            {#if materials.length}
              <li>
                <label class="flex items-center gap-2 text-xs">
                  <input
                    type="checkbox"
                    class="rounded border-white/20 bg-[#1b1b1b] text-orange-500 focus:ring-0"
                    checked={!((segs >> s) & 1)}
                    onchange={(e) =>
                      setSegments(
                        e.currentTarget.checked
                          ? segs & ~(1 << s)
                          : segs | (1 << s),
                      )}
                  />
                  <span class="truncate font-mono text-neutral-300"
                    >{masks?.names?.[s]
                      ? t(`seg_${masks.names[s]}`)
                      : [...new Set(materials.map(pieceName))].join(", ")}</span
                  >
                </label>
              </li>
            {/if}
          {/each}
        </ul>
      {/if}
    {:else if looks.parts.length > 1}
      <button
        use:helpOn={{ note: "ctl_segments", section: "looks" }}
        class="flex items-center justify-between text-xs text-neutral-300 hover:text-neutral-100"
        onclick={() => (showParts = !showParts)}
      >
        {t("parts", { n: looks.parts.length })}
        <IconChevron class={showParts ? "rotate-180" : ""} />
      </button>
      {#if showParts}
        <ul class="flex flex-col gap-1">
          {#each looks.parts as part, i (i)}
            <li>
              <label class="flex items-center gap-2 text-xs">
                <input
                  type="checkbox"
                  class="rounded border-white/20 bg-[#1b1b1b] text-orange-500 focus:ring-0"
                  checked={shown(part)}
                  onchange={(e) => setParts([part], e.currentTarget.checked)}
                />
                <span class="truncate font-mono text-neutral-300"
                  >{part.materials.map(pieceName).join(", ")}</span
                >
                {#if isArmor(part.materials)}
                  <span
                    class="shrink-0 rounded bg-orange-500/20 px-1.5 text-[11px] text-orange-200"
                    >{t("armorTag")}</span
                  >
                {/if}
              </label>
            </li>
          {/each}
        </ul>
      {/if}
    {/if}

    {#if looks.targets > 0}
      <button
        use:helpOn={{ note: "ctl_shapes", section: "looks" }}
        class="flex items-center justify-between text-xs text-neutral-300 hover:text-neutral-100"
        onclick={() => (showShapes = !showShapes)}
      >
        {t("shapes", { n: looks.targets })}
        <IconChevron class={showShapes ? "rotate-180" : ""} />
      </button>
      {#if showShapes}
        <div class="flex flex-col gap-1">
          {#each weights as w, i (i)}
            <label class="flex items-center justify-between gap-2 text-xs">
              <span class="text-neutral-400">{shapeName(i)}</span>
              <input
                type="range"
                class="w-40 accent-orange-500"
                min="0"
                max="1"
                step="0.05"
                value={w}
                oninput={(e) => setWeight(i, +e.currentTarget.value)}
              />
            </label>
          {/each}
        </div>
      {/if}
    {/if}
  </section>
{/if}
