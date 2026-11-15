<script lang="ts">
  import { openPath } from "@tauri-apps/plugin-opener";
  import { GAMES, gameNames, prefsFolder, type GameNames } from "$lib/rpc";
  import { app } from "$lib/settings.svelte";
  import { tick, untrack } from "svelte";
  import { t, type Key } from "$lib/i18n.svelte";
  import { help, THEMES, type Section } from "$lib/help.svelte";
  import { toast } from "$lib/toast.svelte";
  import list from "../../docs/screenshots/guide-list.png";
  import overview from "../../docs/screenshots/studio-overview.gif";
  import anims from "../../docs/screenshots/guide-animations.png";
  import appearance from "../../docs/screenshots/appearance.gif";
  import mocap from "../../docs/screenshots/new-animation-mocap.gif";
  import keys from "../../docs/screenshots/animation-editor.png";
  import exported from "../../docs/screenshots/export.png";
  import tried from "../../docs/screenshots/try-in-game.gif";
  import settingsShot from "../../docs/screenshots/settings.png";
  import mcp from "../../docs/screenshots/mcp-backflip-jak3.gif";

  let section = $state<"usage" | "insert">("usage");
  /** the insertion's lines made for the model opened rather than the Marauder */
  let useOpened = $state(false);

  /** the studio's features, each with a capture and its buttons: their labels are the
   * interface's own texts, what they do next to them */
  const k = (key: Key) => () => t(key);
  const both = (a: Key, b: Key) => () => `${t(a)} / ${t(b)}`;
  const usage: {
    id: Section;
    title: Key;
    text: Key;
    img?: string;
    controls: [() => string, Key][];
  }[] = [
    {
      id: "browse",
      title: "useBrowse",
      text: "useBrowseText",
      img: list,
      controls: [
        [() => Object.values(GAMES).join(" / "), "ctl_games"],
        [both("tabModels", "tabCreations"), "ctl_tabs"],
        [k("search"), "ctl_search"],
        [k("refreshList"), "ctl_refresh"],
        [k("import"), "ctl_import"],
        [() => t("variants", { n: "N" }), "ctl_variants"],
      ],
    },
    {
      id: "watch",
      title: "useWatch",
      text: "useWatchText",
      img: overview,
      controls: [
        [both("play", "pause"), "ctl_play"],
        [k("timeBar"), "ctl_timeBar"],
        [k("speed"), "ctl_speed"],
        [k("skeleton"), "ctl_skeleton"],
        [k("viewMouse"), "ctl_view"],
      ],
    },
    {
      id: "anims",
      title: "useAnims",
      text: "useAnimsText",
      img: anims,
      controls: [
        [both("original", "creation"), "ctl_badge"],
        [k("openFolder"), "ctl_folder"],
        [k("gameAnimation"), "ctl_gameTag"],
        [k("editKeys"), "ctl_editKeys"],
        [k("rename"), "ctl_rename"],
        [k("delete"), "ctl_delete"],
        [() => t("twinAnimations", { n: "N" }), "ctl_twins"],
        [k("newAnimation"), "ctl_new"],
      ],
    },
    {
      id: "looks",
      title: "useLooks",
      text: "useLooksText",
      img: appearance,
      controls: [
        [
          () => [t("lookJak"), t("lookDark"), t("lookLight")].join(" / "),
          "ctl_forms",
        ],
        [k("lightWings"), "ctl_wings"],
        [k("morphGun"), "ctl_gun"],
        [k("armor"), "ctl_armor"],
        [
          () => `${t("outfit", { n: "N" })} / ${t("outfitRandom")}`,
          "ctl_outfit",
        ],
        [
          () => `${t("segments", { n: "N" })} / ${t("parts", { n: "N" })}`,
          "ctl_segments",
        ],
        [() => t("shapes", { n: "N" }), "ctl_shapes"],
      ],
    },
    {
      id: "new",
      title: "useNew",
      text: "useNewText",
      img: mocap,
      controls: [
        [
          () => [t("fromVideo"), t("fromMocap"), t("fromManual")].join(" / "),
          "ctl_source",
        ],
        [k("fps"), "ctl_fps"],
        [k("extract"), "ctl_extract"],
        [k("smoothing"), "ctl_smoothing"],
        [k("faceForward"), "ctl_face"],
        [k("inPlace"), "ctl_inPlace"],
        [k("holdWeapon"), "ctl_hold"],
        [k("mapping"), "ctl_mapping"],
        [k("refine"), "ctl_refine"],
        [k("save"), "ctl_save"],
      ],
    },
    {
      id: "keys",
      title: "useKeys",
      text: "useKeysText",
      img: keys,
      controls: [
        [k("duration"), "ctl_duration"],
        [k("joint"), "ctl_joint"],
        [both("rotateTool", "moveTool"), "ctl_tools"],
        [both("prevKey", "nextKey"), "ctl_steps"],
        [k("addKey"), "ctl_addKey"],
        [k("deleteKey"), "ctl_deleteKey"],
        [k("restPose"), "ctl_rest"],
        [k("undo"), "ctl_undo"],
        [k("save"), "ctl_saveKeys"],
      ],
    },
    {
      id: "check",
      title: "useCheck",
      text: "useCheckText",
      img: exported,
      controls: [
        [k("gameCheck"), "ctl_check"],
        [() => t("exportGame", { game: "…" }), "ctl_export"],
        [k("openReadme"), "ctl_readme"],
      ],
    },
    {
      id: "try",
      title: "useTry",
      text: "useTryText",
      img: tried,
      controls: [
        [k("tryInGame"), "ctl_try"],
        [k("closeGame"), "ctl_close"],
      ],
    },
    {
      id: "settings",
      title: "useSettings",
      text: "useSettingsText",
      img: settingsShot,
      controls: [
        [k("modelsDirs"), "ctl_modelsDirs"],
        [k("creationsDir"), "ctl_creationsDir"],
        [() => t("gameDir", { game: "…" }), "ctl_gameDir"],
        [k("language"), "ctl_language"],
        [k("mcpTitle"), "ctl_mcp"],
      ],
    },
    { id: "mcp", title: "useMcp", text: "useMcpText", img: mcp, controls: [] },
    { id: "log", title: "useLog", text: "useLogText", controls: [] },
  ];
  const byId = Object.fromEntries(usage.map((u) => [u.id, u])) as Record<
    Section,
    (typeof usage)[number]
  >;

  /** the section scrolled to, lit a moment */
  let lit = $state<Section | null>(null);
  async function go(id: string, light?: Section) {
    await tick();
    // the captures above it take their height once loaded
    await Promise.all(
      [...document.querySelectorAll("main img")].map((i) =>
        (i as HTMLImageElement).decode().catch(() => {}),
      ),
    );
    document
      .getElementById(id)
      ?.scrollIntoView({ behavior: "smooth", block: "start" });
    if (light) {
      lit = light;
      setTimeout(() => lit === light && (lit = null), 2000);
    }
  }

  // a control's "See more": its section
  $effect(() => {
    const want = help.open;
    if (!want) return;
    untrack(() => {
      help.open = null;
      section = "usage";
      go(`use-${want}`, want);
    });
  });

  // the guide's example until a model is opened: Jak 3's Marauder with a wave made in the studio
  const marauder = [
    "idle0",
    "idle1",
    "walk0",
    "run0",
    "run1",
    "jump-start",
    "jump",
    "jump-land",
    "jump-out-car",
    "jump-out-car-land",
    "attack-run0-start",
    "attack-run0",
    "attack-jump0-start",
    "attack-jump0",
    "attack-jump0-land",
    "celebrate0",
    "ragdoll",
    "knocked-back0",
    "knocked-back0-land",
    "knocked-death",
    "knocked-death-land",
    "knocked-forward0",
    "knocked-forward0-land",
    "knocked-forward-death",
    "knocked-forward-death-land",
    "drown-lava",
    "run2",
    "attack-shoot0-start",
    "attack-shoot0-shoot",
    "attack-shoot0-end",
    "turn",
    "die",
    "wave",
  ];
  const EXAMPLE: GameNames = {
    actor: "marauder-male-lod0",
    art: marauder.map((a) => `${a}-ja`),
    slots: marauder.map((a, i) => ({
      slot: i + 2,
      animation: a === "wave" ? a : `marauder-male-${a}`,
      symbol: `marauder-male-lod0-${a}-ja`,
    })),
  };

  let opened = $state<GameNames | null>(null);
  let chosen = $state<string | null>(null);
  $effect(() => {
    const model = app.model;
    if (!model) return;
    gameNames(model)
      .then((n) => (opened = n.slots.length ? n : null))
      .catch(() => (opened = null));
  });

  const mine = $derived(useOpened ? opened : null);
  const names = $derived(mine ?? EXAMPLE);
  const game = $derived(mine ? (app.settings?.game ?? "jak1") : "jak3");
  const actor = $derived(names.actor);
  /** the entity's type: "my-" and the model's first word (my-marauder) */
  const type = $derived(`my-${actor.split("-")[0]}`);
  const idle = $derived(names.slots[0].symbol);
  const play = $derived(
    names.slots.find((s) => s.symbol === chosen) ?? names.slots.at(-1)!,
  );
  /** the event and the state that play it: the animation's short name */
  const event = $derived.by(() => {
    const short = play.symbol.slice(actor.length + 1, -"-ja".length);
    return short === "idle" ? "play" : short;
  });
  const code = $derived({
    model: `custom_assets/${game}/models/custom_levels/${actor}.glb`,
    build: `(build-actor "${actor}")`,
    models: `"custom_models": ["${actor}"],`,
    entity: `;;-*-Lisp-*-
(in-package goal)

;; the art group of the model, its animations in the file's order
(def-actor ${actor}
  :bounds (0 0 0 4)
  :art (${names.art.join(" ")}))

;; the entity: an animated model with two states
(deftype ${type} (process-drawable)
  ()
  (:state-methods
   idle
   ${event}))

;; its first animation again and again; the event '${event} plays the other one
(defstate idle (${type})
  :virtual #t
  :event
    (behavior ((proc process) (argc int) (message symbol) (block event-message-block))
      (case message
        (('${event}) (go-virtual ${event}))))
  :code
    (behavior ()
      (loop
        (ja-no-eval :group! ${idle} :num! (seek!) :frame-num 0.0)
        (until (ja-done? 0)
          (suspend)
          (ja :num! (seek!)))))
  :post ja-post)

;; the animation once, then idle again
(defstate ${event} (${type})
  :virtual #t
  :code
    (behavior ()
      (ja-no-eval :group! ${play.symbol} :num! (seek!) :frame-num 0.0)
      (until (ja-done? 0)
        (suspend)
        (ja :num! (seek!)))
      (go-virtual idle))
  :post ja-post)

;; the level makes one for each of its actors whose etype is ${type}
(defmethod init-from-entity! ((this ${type}) (e entity-actor))
  (set! (-> this root) (new 'process 'trsqv))
  (process-drawable-from-entity! this e)
  (initialize-skeleton this *${actor}-sg* '())
  (logclear! (-> this mask) (process-mask actor-pause))
  (go-virtual idle :proc this)
  (none))`,
    // the engine file the code needs, as for test-zone's own code
    compile: `(goal-src "levels/test-zone/${type}.gc" "${game === "jak1" ? "process-drawable" : "process-focusable"}")`,
    gd: `("TSZ.DGO"
 ("${type}.o"
  ...
  "${actor}-ag.go"
  "test-zone.go"
 ))`,
    actor: `{
  "trans": [0, 3.5, 4],
  "etype": "${type}",
  "game_task": 0,
  "quat": [0, 0, 0, 1],
  "bsphere": [0, 3.5, 4, 10],
  "lump": { "name": "${type}-1" }
}`,
    play: `(mi)
(lt)
(start 'play (get-continue-by-name *game-info* "test-zone-start"))
(send-event (process-by-ename "${type}-1") '${event})`,
    vis: "(set! (-> *pc-settings* ps2-actor-vis?) #f)",
  });

  const steps = $derived([
    { title: "guideModel", text: "guideModelText", code: code.model },
    { title: "guideBuild", text: "guideBuildText", code: code.build },
    { title: "guideMesh", text: "guideMeshText", code: code.models },
    {
      title: "guideCode",
      text: "guideCodeText",
      code: code.entity,
      pick: true,
    },
    { title: "guideCompile", text: "guideCompileText", code: code.compile },
    { title: "guideLoad", text: "guideLoadText", code: code.gd },
    { title: "guidePlace", text: "guidePlaceText", code: code.actor },
    { title: "guidePlay", text: "guidePlayText", code: code.play },
  ] as const);

  const args = $derived({
    game: GAMES[game],
    gameId: game,
    model: actor,
    type,
    anim: play.animation,
    event,
    level: `custom_assets/${game}/levels/test-zone`,
  });

  async function copy(text: string) {
    await navigator.clipboard.writeText(text);
    toast(t("copied"));
  }
</script>

{#snippet block(text: string)}
  <div class="relative">
    <pre
      class="overflow-x-auto rounded-md border border-white/10 bg-[#1b1b1b] px-4 py-3 font-mono text-xs leading-relaxed text-neutral-200">{text}</pre>
    <button
      class="absolute top-2 right-2 rounded-md bg-white/10 px-3 py-1 text-xs hover:bg-white/20"
      onclick={() => copy(text)}>{t("copy")}</button
    >
  </div>
{/snippet}

<div class="h-full overflow-y-auto p-8">
  <div class="mx-auto flex max-w-5xl flex-col gap-8">
    <div class="flex border-b border-white/10 text-sm">
      {#each ["usage", "insert"] as const as id (id)}
        <button
          class="px-4 py-3 {section === id
            ? 'border-b-2 border-orange-500 text-white'
            : 'text-neutral-400 hover:text-neutral-200'}"
          onclick={() => (section = id)}
          >{t(id === "usage" ? "guideUsage" : "guideTitle")}</button
        >
      {/each}
    </div>

    {#if section === "usage"}
      <div class="flex gap-10">
        <nav
          class="sticky top-0 hidden w-52 shrink-0 flex-col gap-4 self-start pt-1 text-sm lg:flex"
          aria-label={t("guideContents")}
        >
          <span
            class="text-xs font-semibold tracking-wide text-neutral-500 uppercase"
            >{t("guideContents")}</span
          >
          {#each THEMES as theme, ti (theme.id)}
            <div class="flex flex-col gap-1">
              <button
                class="text-left font-semibold text-neutral-200 hover:text-orange-400"
                onclick={() => go(`theme-${theme.id}`)}
                >{ti + 1}. {t(theme.title)}</button
              >
              <ul
                class="ml-1 flex flex-col gap-1 border-l border-white/10 pl-3"
              >
                {#each theme.sections as id (id)}
                  <li>
                    <button
                      class="text-left hover:text-neutral-100 {lit === id
                        ? 'text-orange-400'
                        : 'text-neutral-400'}"
                      onclick={() => go(`use-${id}`, id)}
                      >{t(byId[id].title)}</button
                    >
                  </li>
                {/each}
              </ul>
            </div>
          {/each}
        </nav>

        <div class="flex min-w-0 flex-1 flex-col gap-12">
          <p class="text-sm text-neutral-300">{t("guideUsageIntro")}</p>
          {#each THEMES as theme, ti (theme.id)}
            <div id="theme-{theme.id}" class="flex scroll-mt-2 flex-col gap-6">
              <h2
                class="font-title border-b border-white/10 pb-2 text-xl text-neutral-100"
              >
                {ti + 1}. {t(theme.title)}
              </h2>
              {#each theme.sections as id, si (id)}
                {@const item = byId[id]}
                <section
                  id="use-{id}"
                  class="flex scroll-mt-2 flex-col gap-3 rounded-lg border bg-white/[0.02] p-5 transition-colors duration-700 {lit ===
                  id
                    ? 'border-orange-500'
                    : 'border-white/10'}"
                >
                  <h3 class="text-base font-semibold text-neutral-100">
                    {ti + 1}.{si + 1}
                    {t(item.title)}
                  </h3>
                  <p class="text-sm text-neutral-400">{t(item.text)}</p>
                  {#if item.img}
                    <img
                      src={item.img}
                      alt={t(item.title)}
                      class="max-w-full self-start rounded-md border border-white/10"
                    />
                  {/if}
                  {#if item.controls.length}
                    <table class="text-sm">
                      <tbody>
                        {#each item.controls as [label, what] (what)}
                          <tr class="border-t border-white/5 align-top">
                            <td
                              class="w-56 py-2 pr-4 font-semibold text-neutral-200"
                              >{label()}</td
                            >
                            <td class="py-2 text-neutral-400">{t(what)}</td>
                          </tr>
                        {/each}
                      </tbody>
                    </table>
                  {/if}
                  {#if id === "log"}
                    <button
                      class="self-start text-sm text-orange-400 hover:text-orange-300"
                      onclick={async () => openPath(await prefsFolder())}
                      >{t("useLogOpen")}</button
                    >
                  {/if}
                </section>
              {/each}
            </div>
          {/each}
        </div>
      </div>
    {:else}
      <div class="mx-auto flex w-full max-w-3xl flex-col gap-8">
        <h1 class="font-title text-2xl">{t("guideTitle")}</h1>
        <p class="text-sm text-neutral-300">
          {t(mine ? "guideIntro" : "guideIntroExample", args)}
        </p>
        {#if opened}
          <label class="flex items-center gap-2 text-sm text-neutral-300">
            <input
              type="checkbox"
              class="rounded accent-orange-500"
              bind:checked={useOpened}
            />
            {t("guideUseOpened", { model: opened.actor })}
          </label>
        {/if}
        <p class="text-sm text-neutral-400">{t("guideNeed", args)}</p>

        {#each steps as step, i (step.title)}
          <section class="flex flex-col gap-2">
            <h2 class="text-sm font-semibold text-neutral-200">
              {i + 1}. {t(step.title)}
            </h2>
            <p class="text-sm text-neutral-400">{t(step.text, args)}</p>
            {#if "pick" in step}
              <label class="flex items-center gap-2 text-sm text-neutral-300">
                {t("guideEventAnim")}
                <select
                  class="rounded-md border border-white/10 bg-[#1b1b1b] px-3 py-1.5 text-sm"
                  value={play.symbol}
                  onchange={(e) => (chosen = e.currentTarget.value)}
                >
                  {#each names.slots.slice(1) as s (s.symbol)}
                    <option value={s.symbol}>{s.animation}</option>
                  {/each}
                </select>
              </label>
            {/if}
            {@render block(step.code)}
          </section>
        {/each}

        <section
          class="flex flex-col gap-2 rounded-lg border border-orange-500/40 bg-orange-500/10 p-4"
        >
          <h2 class="text-sm font-semibold text-neutral-200">
            {t("guideNotBorn")}
          </h2>
          <p class="text-sm text-neutral-300">{t("guideNotBornText")}</p>
          {@render block(code.vis)}
          {#if game === "jak1"}
            <p class="text-sm text-neutral-300">{t("guideJak1")}</p>
          {/if}
        </section>

        <section
          class="flex flex-col gap-2 rounded-lg border border-orange-500/40 bg-orange-500/10 p-4"
        >
          <h2 class="text-sm font-semibold text-neutral-200">
            {t("guideMemory")}
          </h2>
          <p class="text-sm text-neutral-300">{t("guideMemoryText", args)}</p>
        </section>
      </div>
    {/if}
  </div>
</div>
