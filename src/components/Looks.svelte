<script lang="ts">
  import { ask } from "@tauri-apps/plugin-dialog";
  import IconChevron from "~icons/mdi/chevron-down";
  import {
    applyLooks,
    findModel,
    gameTextures,
    modelLooks,
    readModel,
    type Game,
    type Looks,
    type ModelInfo,
    type Part,
  } from "$lib/rpc";
  import { isArmor, known, pieceName } from "$lib/looks";
  import { blend, eye, pixels } from "$lib/textures";
  import type { Viewer } from "$lib/viewer";
  import { errorText, t } from "$lib/i18n.svelte";
  import { toast } from "$lib/toast.svelte";

  let {
    viewer,
    model,
    game,
    onapplied,
  }: {
    viewer: Viewer;
    model: ModelInfo;
    game: Game;
    onapplied: (path: string) => void;
  } = $props();

  let looks = $state.raw<Looks | null>(null);
  let hidden = $state.raw(new Set<string>());
  let weights = $state.raw<number[]>([]);
  let light = $state(false);
  let noTextures = $state(false);
  /** the game's textures of the eyes, and of Dark Jak's skin (its normal and dark ones) */
  let eyes: { iris: ImageData; pupil: ImageData } | null = null;
  let skins: { name: string; norm: ImageData; dark: ImageData }[] = [];
  let showParts = $state(false);
  let showShapes = $state(false);
  let applying = $state(false);

  const jak = $derived(known(game, model.name));
  const armor = $derived(
    looks?.parts.filter((p) => isArmor(p.materials)) ?? [],
  );
  const isDark = $derived(
    !!jak && jak.dark.every((i) => (weights[i] ?? 0) >= 1),
  );
  const keys = (p: Part) => p.primitives.map((i) => `${p.mesh}:${i}`);
  const shown = (p: Part) => keys(p).every((k) => !hidden.has(k));

  $effect(() => {
    const path = model.path;
    modelLooks(path)
      .then((l) => {
        looks = l;
        weights = Array(l.targets).fill(0);
        return loadTextures(l);
      })
      .catch((e) => toast(errorText(e), "error"));
  });

  /** The model's eyes are drawn by the game (its materials are only "programmer_eye" stand-ins),
   * from textures in the page of its level: `<prefix>-eye` or `<prefix>-iris...`, else common's
   * autoeye ones, which Jak's are. */
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
    if (hasEyes && files.has(iris) && files.has(pupil))
      eyes = {
        iris: await pixels(files.get(iris)!),
        pupil: await pixels(files.get(pupil)!),
      };
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
    // Dark Jak's eyes are black: the game sets their pupil 2 eyes wide (target-darkjak.gc)
    const darkness = jak
      ? jak.dark.reduce((sum, i) => sum + (weights[i] ?? 0), 0) /
        jak.dark.length
      : 0;
    const textures = new Map<string, ImageData>();
    if (eyes) {
      const e = eye(eyes.iris, eyes.pupil, 0.55, 0.45 + 1.55 * darkness);
      textures.set("programmer_eye_left", e).set("programmer_eye_right", e);
    }
    if (darkness > 0)
      for (const s of skins)
        if (s.norm.width === s.dark.width && s.norm.height === s.dark.height)
          textures.set(s.name, blend(s.norm, s.dark, darkness));
    viewer.setTextures(textures);
    viewer.setLit(light);
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

  function setWeight(target: number, w: number) {
    weights = weights.map((x, i) => (i === target ? w : x));
    show();
  }

  function setDark(on: boolean) {
    weights = weights.map((x, i) => (jak?.dark.includes(i) ? +on : x));
    show();
  }

  async function setWings(on: boolean) {
    if (!on || !jak?.wings) return viewer.setWings(null);
    try {
      const path = await findModel(model.path, game, jak.wings.model);
      if (!path) throw new Error(t("noWings", { name: jak.wings.model }));
      await viewer.setWings(
        await readModel(path),
        jak.wings.joint,
        jak.wings.anchor,
      );
    } catch (e) {
      toast(errorText(e), "error");
    }
  }

  /** Light Jak: his wings, and no textures under the rim lights (powerups.gc) */
  function preset(look: "jak" | "dark" | "light") {
    light = look === "light";
    setDark(look === "dark");
    setWings(light);
  }

  function reset() {
    hidden = new Set();
    weights = weights.map(() => 0);
    light = false;
    setWings(false);
    show();
  }

  const shapeName = (i: number) =>
    jak?.dark.includes(i)
      ? t("shapeDark", { n: jak.dark.indexOf(i) + 1 })
      : jak?.shapes?.[i]
        ? t(`shape_${jak.shapes[i]}`)
        : t("shape", { n: i });

  async function apply() {
    if (!(await ask(t("applyLookConfirm"), { kind: "warning" }))) return;
    applying = true;
    try {
      const pairs = [...hidden].map(
        (k) => k.split(":").map(Number) as [number, number],
      );
      const path = await applyLooks(model.path, pairs, weights);
      toast(t("savedToCreations"));
      onapplied(path);
    } catch (e) {
      toast(errorText(e), "error");
    } finally {
      applying = false;
    }
  }
</script>

{#if looks && (looks.parts.length > 1 || looks.targets > 0)}
  <section class="flex flex-col gap-2">
    <h3 class="text-sm font-semibold text-neutral-200">{t("looks")}</h3>
    <p class="text-xs text-neutral-400">{t("looksHint")}</p>
    {#if noTextures}
      <p class="text-xs text-amber-300">{t("noGameTextures")}</p>
    {/if}

    {#if jak}
      <div class="grid grid-cols-3 gap-2">
        {#each [["jak", "lookJak", !isDark && !light], ["dark", "lookDark", isDark], ["light", "lookLight", light]] as const as [id, label, active] (id)}
          {#if id === "jak" || (id === "dark" ? looks.targets > 0 : jak.wings)}
            <button
              class="rounded-md border px-2 py-1.5 text-sm {active
                ? 'border-orange-500 bg-orange-500/10'
                : 'border-white/10 hover:bg-white/5'}"
              onclick={() => preset(id)}>{t(label)}</button
            >
          {/if}
        {/each}
      </div>
    {/if}
    {#if armor.length}
      <label class="flex items-center gap-2 text-sm">
        <input
          type="checkbox"
          class="rounded border-white/20 bg-[#1b1b1b] text-orange-500 focus:ring-0"
          checked={armor.every(shown)}
          onchange={(e) => setParts(armor, e.currentTarget.checked)}
        />
        <span class="text-neutral-300">{t("armor")}</span>
      </label>
    {/if}

    {#if looks.parts.length > 1}
      <button
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

    <div class="flex gap-2">
      <button
        class="flex-1 rounded-md bg-white/10 px-3 py-1.5 text-sm hover:bg-white/20"
        onclick={reset}>{t("resetLook")}</button
      >
      <button
        class="flex-1 rounded-md bg-white/10 px-3 py-1.5 text-sm hover:bg-white/20 disabled:opacity-40"
        disabled={applying}
        onclick={apply}>{t("applyLook")}</button
      >
    </div>
  </section>
{/if}
