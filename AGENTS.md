## Summary

- [1. Overview & Architecture](#1-overview--architecture)
- [2. Local Environment & Tooling](#2-local-environment--tooling)
- [3. AI Assistant Rules](#3-ai-assistant-rules)
- [4. The Game's Rules](#4-the-games-rules)
- [5. Documentation Standard](#5-documentation-standard)

---

## 1. Overview & Architecture

**Jak Animation Studio** plays, makes and checks the animations of Jak and Daxter entities
(skinned `.glb` models) for OpenGOAL's `build-actor`. It runs on its own, and the OpenGOAL Launcher
installs and starts it the way it does Jak Level Editor (`--lang`, `--default-library=`,
`--game-install=`, `--tooling-version=`: started so, it tries models in the Launcher's games only,
with its active tooling).

### Technology Stack

- **Desktop runtime**: Tauri v2 (Rust), in `src-tauri/`.
- **Interface**: Svelte 5 (runes), Tailwind CSS v4, Flowbite Svelte, Vite, TypeScript, three.js.
- **Workspace** (`Cargo.toml`):
  - `crates/core/`: everything about models, shared by the program and the MCP server.
    - `glb.rs`: the `.glb` container, read and written in place (what is not touched is kept as is; a file left as it was is written back byte for byte), compaction of unused data.
    - `model.rs`: the skeleton (the joints of the single skin, in `build-actor`'s order) and the animations.
    - `rules.rs`: what `build-actor` needs, checked (the size of the animations once built: `varies` replays its sampling; the names the animations get in the game, `game_name`; the weights of the points a model draws only); the issues the interface and the MCP server show, as `Text` (a code, its arguments, the English message).
    - `looks.rs`: a model's parts (runs of primitives drawing the same vertices: the extraction does not keep which merc effect a primitive was) and blend shapes; a look applied (parts removed, shapes baked).
    - `names.rs`, `names.json`: the official English and French names of the entities, by game and art group, each with its source.
    - `living.json`: per game, the models that are living entities and those that are objects (every skeleton of `goal_src/<game>/engine/data/art-elts.gc`). Made from the code (the model's type derives from `nav-enemy`, `enemy` or `process-taskable`), then checked by hand; the Models list hides the objects.
    - `edit.rs`: animations written as `build-actor` reads them; `normalize` for the export (only the animations that need it, names kept); `for_build_actor`, the skin set so that any `build-actor` builds the model right, `trim_points`, the points no triangle draws dropped, and `hold_still_keys`, keys all at 0 s given a second one (all three in the export and the try); `copy_animations` from a twin.
    - `library.rs`: settings (a models folder per game), the models listed (one copy of each model the extraction wrote out of several levels), the creations folder (by game; `edit_creation` refuses a change that gives two animations one name in the game), import, the export, the twins (other files of the same entity and skeleton, where the extraction put its animations), and the MCP server's requests to play in the window (`play.json`).
    - `game_test.rs`: the try in the game (Jak Level Editor's way): the installation found, goalc's project in `<prefs>/game-test/<game>/`, test-zone rebuilt with the model as `ogas-model` and the viewer `ogas-play`, gk started, the game's files kept and put back.
  - `crates/mcp/`: the MCP server (JSON-RPC over stdio), on the files and the settings.
  - `src-tauri/`: the program's command line, its Tauri commands, and the watch of the play requests (it brings the window up).
- **Interface (`src/`)**:
  - `lib/viewer.ts`: the 3D view (three.js).
  - `lib/frames.ts`: body points over time, the common form of every capture.
  - `lib/pose.ts`: captures from a video (MediaPipe Pose) and from mocap files (BVH, FBX, glTF).
  - `lib/mapping.ts`: which joint plays which body part, guessed from names.
  - `lib/retarget.ts`: captured points to joint rotations, by bone directions.
  - `lib/looks.ts`: what the games' code does to Jak's looks (Dark Jak's blend targets, Light Jak's wings), with its sources.
  - `lib/textures.ts`: the game's textures (`decompiler_out/<game>/textures`): the eyes drawn as the game's eye renderer does, Dark Jak's skin. `lib/viewer.ts` draws the materials as the game's merc and emerc shaders do, and Light Jak's rim lights.
  - `lib/keys.ts`: the animation editor's keys (set, delete, saved on the short way); `lib/variants.ts`: the tags of an entity's variants. Each `lib/*.check.ts` is their runnable check.
  - `components/`: the screens.

---

## 2. Local Environment & Tooling

| Tool              | Command                                                                           | Notes                                                                                                                                                                    |
| :---------------- | :-------------------------------------------------------------------------------- | :----------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Rust              | `cargo`                                                                           | `cargo test -p animation-studio-core` runs the core's tests; `-- --ignored` runs those on the games' models and the try in a game (docs/technical.md, Build the studio). |
| Node packages     | `npx yarn install`                                                                | Yarn 1, run through npx.                                                                                                                                                 |
| Desktop dev       | `npx yarn tauri dev`                                                              | Vite on port 3002 (the launcher takes 3000, Jak Level Editor 3001).                                                                                                      |
| Typecheck         | `npx yarn typecheck`                                                              | `svelte-check`.                                                                                                                                                          |
| Retargeting check | `npx yarn check:retarget`, `npx yarn check:variants`                              | Node runs `src/lib/*.check.ts` directly.                                                                                                                                 |
| Formatting        | `npx yarn format`                                                                 | Prettier with the Svelte plugin, then rustfmt (2-space tabs).                                                                                                            |
| Autoformat        | `.githooks/pre-commit`                                                            | Formats each commit's files; `npx yarn install` turns it on.                                                                                                             |
| Rust linter       | `cargo clippy --workspace --all-targets -- -D warnings`                           | Needs `dist/` (`npx yarn build`): the program embeds it.                                                                                                                 |
| CI                | `.github/workflows/`                                                              | Lint, Build, Release (by hand: patch, minor, major), Format.                                                                                                             |
| Agent skills      | `git submodule update --init --remote .agents/skills` then `npx yarn skills:link` | The `opengoal-modding-kb` submodule, linked into `.claude/skills/`.                                                                                                      |

---

## 3. AI Assistant Rules

1. **Say what changes**: never create, change or delete a file without saying so in the answer.
2. **Keep the project's way**: respect the architecture above and the code's style; no change to unrelated files.
3. **No filler comments**: comments only for what the code cannot say (the game's quirks, `build-actor`'s behaviour, workarounds).
4. **Never edit generated files** (`src-tauri/gen/`, `dist/`, `src-tauri/icons/` made by `tauri icon` from `src-tauri/app-icon.svg`).
5. **Commits by batch**: one commit per coherent change (a feature, a fix, the docs), in English:
   - the title: one short line saying what the commit brings, ending with `(AI-assisted)` for AI-assisted work;
   - the body: a few short lines with the essentials (what the user gets, what behaves differently).

   ```text
   Complete Jak 3's animations from the twin file (AI-assisted)

   The extraction puts Jak 3's mesh in jakb-c-lod0.glb and its 289 animations in jakb-lod0.glb.
   The studio finds the files of the same entity and skeleton, and a button copies their
   animations into the creation.
   ```

6. **No commit and no push** unless the user asks for it.
7. **No game data**: no model, texture, sound or extracted file of the games in the repository. Tests make their own models.
8. **Interface texts**: every text goes through `src/lib/i18n.svelte.ts`, in English and French.
   A text of the core the interface shows (issue, export note, error) is a `rules::Text`; its code
   is the key `core_<code>` there. Its English message is what the MCP server says.
9. **Official names**: an entity's name in `names.json` has a source (the game's text or a wiki
   page). No name is translated or guessed.

---

## 4. The Game's Rules

The source of truth is jak-project's `goalc/build_actor/` (`common/animation_processing.cpp`,
`common/build_actor.cpp`, `jak1|jak2|jak3/build_actor.cpp`). Before changing `rules.rs` or
`edit.rs`, read the code there. The facts the studio relies on:

- one skin per model; joints topologically sorted; joint 0 `align` (else one is added in front);
- `build-actor` of OpenGOAL v0.3.9 adds its own `align` in front even when the skin has one
  (whozghiar's tooling only when it has none), while the mesh gives each point the matrix of its
  joint's number + 1, which the game moves with joint `k - 1` (`common/util/gltf_util.cpp`,
  `bones.gc`): points follow the joint before theirs. `edit::for_build_actor` takes `align` out of
  the skin (its pose to its children) so that every `build-actor` builds the model right;
- a joint without keys in an animation gets identity (origin) values, not its rest pose;
- channels other than translation, rotation and scale stop the build; keys are read as linear floats;
- at most 2 + 14 × 8 joints; at most 133 quadwords per frame and for the constant part;
- the art group's elements are the joint geometry, the mesh, then the animations in the file's order:
  animation `i` is slot `i + 2`, which `def-actor :art` names `<actor>-<anim>-ja`; the names inside
  the file matter to name lookups only (`lookup-art`);
- keys are sampled at 60 frames a second, blending neighbouring keys in 32-bit floats: equal keys can
  come out a rounding apart, and the channel then takes room in every frame;
- no limit on the number of animations; a level loads each file of its DGO whole into one buffer
  (2 MB in Jak 1, `#x1b5800` bytes in Jak 2 and 3, less in Jak 3's smallest levels) and checks nothing.

The try in the game (`game_test.rs`) relies on these, each seen in the games:

- gk listens on 8112, 8113 or 8114 (`DECI2_PORT - 1 +` the game) whatever its `--port`, and reads
  every file of `out/<game>/iso/` when it starts (names of 15 letters at most: nothing else there);
- a custom actor's mesh is drawn from the level's `.fr3`: the model is in the level's `custom_models`;
- Jak 2 and 3 look for a process's art groups in its level only: the viewer takes test-zone's;
- goalc compiles the code of `GAME.CGO` in `game.gd`'s order (the viewer's dependencies): another
  order fails (`*pc-settings*` unknown in `pad.gc`, Jak 1);
- `build-actor` of OpenGOAL v0.3.9 makes no frame of a joint whose keys are all at 0 s (a single
  key): the joint goes to the origin (a wave folds the Marauder into a ball), and an animation with
  no frame at all (a still pose) makes it die;
- Jak 1 closes when a level loads a custom actor with a skeleton (jak-project of October 2026);
- a level made by hand has no visibility data: with `ps2-actor-vis?` on (PS2 actor vis in the debug
  menu), its actors are never born, the game's own `test-actor` included (the in-game guide says so).

The `custom-actors-levels` and `goal-lisp` skills in `.agents/skills/` explain the whole pipeline.

---

## 5. Documentation Standard

Follow the `documentalist` skill (`.agents/skills/documentalist/`): English only, written for
beginners, short and direct, tables for mappings, numbered lists for steps, relative links, every
fact checked against the repository.

Teach with a concrete example: each feature, setting or rule shows a real case (an entity, a file,
a command, a value) next to the general description. The export of Babak with a `wave` animation
in `docs/technical.md` is the model: the files written, the lines to add, the slot the animation
gets. Run the example before writing it down.

The README presents the studio briefly, without technical detail; `docs/tutorial.md` teaches it
step by step with screenshots (`docs/screenshots/`); `docs/in-game-importation.md` takes an exported
model into a mod (the **In-game guide** page, `components/Guide.svelte`, shows the same steps with
the lines of `library::game_names` for the model opened); `docs/technical.md` explains how it works, one
concept at a time with an example each, without going into the code.
