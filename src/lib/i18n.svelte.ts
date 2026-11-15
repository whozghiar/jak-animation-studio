const en = {
  navStudio: "Studio",
  navSettings: "Settings",
  minimize: "Minimize",
  maximize: "Maximize",
  close: "Close",
  tabModels: "Models",
  tabCreations: "Creations",
  search: "Search",
  variants: "{n} variants",
  tag_highres: "High detail",
  tag_lowres: "Low detail",
  tag_simple: "Simple",
  tag_rider: "In a vehicle",
  tag_escort: "Escort",
  tag_main: "Main",
  import: "Import models (.glb) into the creations",
  refreshList: "Read the list again",
  importReplace:
    "A creation is already named {name}. Replace it? Its animations will be lost.",
  noModelsDir: "Choose the models folder of {game} in the settings.",
  listingModels: "Reading the game's models… {p} %",
  noModels:
    "No entity here: only the living entities of the game (characters, creatures, enemies) are listed.",
  noCreations:
    "Nothing yet: what you change in a model is saved here, the original is never touched.",
  pickModel: "Pick an entity on the left to see it and play its animations.",
  joints: "{n} joints",
  anims: "{n} animations",
  creation: "Creation",
  original: "Original",
  openFolder: "Show in folder",
  animations: "Animations",
  noAnimation: "No animation yet.",
  newAnimation: "New animation",
  twinAnimations: "Add {n} animations from models with the same skeleton",
  rename: "Rename",
  delete: "Delete",
  deleteConfirm: "Delete the animation {name}?",
  gameCheck: "Game check",
  allGood: "Ready for build-actor.",
  gameAnimation: "Game",
  gameAnimations:
    "These {n} animations are the game's (Jak and Daxter are in every level): only those you add are checked and exported.",
  exportGame: "Export for {game}",
  tryInGame: "Try in the game",
  tryStep_export: "Writing the model",
  tryStep_compiler:
    "Starting the game's compiler (a minute or two the first time)",
  tryStep_build: "Building the test level",
  tryStep_start: "Starting the game",
  tryStep_load: "Taking Jak to the test level",
  tryStep_play: "Showing the model",
  inGame:
    "In the game: the animation picked here plays there too. One the game does not hold yet, or one you edit, is built again (the game restarts).",
  closeGame: "Close the game",
  navGuide: "Guide",
  guideUsage: "Using the studio",
  guideUsageIntro:
    "What the studio does, from finding a model to seeing it in the game. The second tab puts a model in a mod of yours, step by step.",
  useBrowse: "The list of models",
  useBrowseText:
    "The list on the left holds the game's living entities (characters, creatures, enemies) and your creations, the copies the studio keeps of what you change. Pick one to open it.",
  useLooks: "Appearance",
  useLooksText:
    "The model as the game's code draws it. Nothing here changes the model or its textures: the studio only edits animations.",
  useNew: "Make an animation",
  useNewText:
    "New animation puts a movement on the model: from a video of one person, a motion capture file (BVH, FBX, glTF), or by hand. It is saved in the model's creation: the original model never changes.",
  useKeys: "Edit the keys",
  useKeysText:
    "The clapperboard next to an animation opens its keys: pick a joint, turn or move it, and a key is set at the moment shown. K keys a joint as it is, Del deletes its key, Ctrl+Z undoes.",
  useCheck: "Check and export",
  useCheckText:
    "Game check lists what build-actor would stop on or build wrong. Export writes the creation as build-actor needs it, with a README of the lines to add to the game. Jak's and Daxter's own animations are the game's: only those you add are checked and exported.",
  useTry: "Try it in the game",
  useTryText:
    "Try in the game builds the model into the game's test level and starts the game: the model plays the animation picked, and the next one you pick. The game's files are put back when it closes.",
  useSettings: "Settings",
  useSettingsText:
    "The folders the studio works with, the language, and the AI assistant.",
  useMcp: "With an AI assistant",
  useMcpText:
    "Settings give the command that adds the studio's MCP server to Claude Code. The assistant then lists the models, makes animations from your words or from a motion capture, and checks them in the game; the studio shows what it does.",
  useLog: "When something goes wrong",
  useLogText:
    "Every error is written to errors.log in the studio's folder; the try in the game writes what goalc and the game said to game-test/<game>/goalc.log and gk.log there.",
  useLogOpen: "Open the studio's folder",
  guideContents: "Contents",
  seeMore: "See more",
  themeFind: "Find a model",
  themeAnimate: "Animate",
  themeLooks: "The model's looks",
  themeGame: "To the game",
  themeSetup: "Settings and help",
  useWatch: "Watch a model",
  useWatchText:
    "The model plays its first animation in the view; the bar under the view drives it.",
  useAnims: "The animations",
  useAnimsText:
    "The right panel lists the model's animations: click one to play it. The buttons of a line show when the mouse is over it.",
  timeBar: "Time bar",
  viewMouse: "The view",
  ctl_games: "The game whose models are listed.",
  ctl_tabs:
    "The game's entities, or your creations: the copies the studio keeps of the models you changed (the original never changes).",
  ctl_search: "Finds an entity by its official name or its file's.",
  ctl_refresh:
    "Reads the list again: a creation the AI assistant made, or a new extraction of the game.",
  ctl_import: "Copies .glb files of your own into the creations.",
  ctl_variants:
    "An entity the game splits into several models (high detail, in a vehicle...): the arrow opens them.",
  ctl_play: "Plays or pauses the animation.",
  ctl_timeBar:
    "Drags the animation to a moment; each mark above it is a key, the orange ones the picked joint's.",
  ctl_speed: "Plays slower (0.25x, 0.5x) or faster (2x).",
  ctl_skeleton: "Shows the joints and bones over the model.",
  ctl_view: "Drag to turn around the model, right-drag to move, wheel to zoom.",
  ctl_badge:
    "Original: the game's model, read only. Creation: your copy, the one exported.",
  ctl_folder: "Opens the folder of the model's file.",
  ctl_gameTag:
    "Jak's or Daxter's own animation: the game has it, so it is neither checked nor exported; it can be tried in the game, and edited into a version of your own.",
  ctl_editKeys: "Opens the animation in the key editor.",
  ctl_rename: "Renames the animation (in the creation).",
  ctl_delete: "Deletes the animation (from the creation).",
  ctl_twins:
    "Adds the animations other files of the same entity and skeleton hold, where the extraction put them.",
  ctl_new:
    "Makes a new animation from a video, a motion capture file or by hand.",
  ctl_forms:
    "Jak as himself, as Dark Jak (shapes, skin, black eyes) or as Light Jak (the game's lights).",
  ctl_wings: "Light Jak's wings, out only when he glides.",
  ctl_gun: "Puts the Morph Gun in Jak's hand, with the mod of your choice.",
  ctl_armor:
    "Shows or hides the armor pieces of a model without the game's segments.",
  ctl_outfit:
    "Every look the game's code gives the model (colours, hats, weapons...); Random picks one.",
  ctl_segments: "The model's pieces one by one, to show or hide.",
  ctl_shapes:
    "The blend shapes (Dark Jak's, Jak's goggles and scarf), from 0 to 1.",
  ctl_source:
    "Where the movement comes from: a video of one person, a BVH, FBX or glTF file, or nothing (the rest pose, to key by hand).",
  ctl_fps:
    "How many keys the animation gets a second: 15 by default, as the game's own animations; more is finer, and heavier.",
  ctl_extract:
    "Reads the movement and puts it on the model, next to the orange figure of the capture.",
  ctl_smoothing: "Calms a shaky capture.",
  ctl_face: "Turns the movement to face the model's front.",
  ctl_inPlace:
    "Keeps the model on the spot, as the game wants: its code moves it through the level.",
  ctl_hold:
    "Keeps a weapon in both hands, with the arms of the model's first animation.",
  ctl_mapping: "Which joint of the model plays which body part.",
  ctl_refine: "Goes on in the key editor.",
  ctl_save: "Saves the animation in the model's creation.",
  ctl_duration: "How long the animation lasts, in seconds.",
  ctl_joint: "The joint to move; clicking its dot in the view picks it too.",
  ctl_tools:
    "Turns the joint with its rings, or moves it with its arrows: a key is set at the moment shown.",
  ctl_steps: "Goes to the previous or next key.",
  ctl_addKey: "Keys the joint as it is, at the moment shown.",
  ctl_deleteKey: "Deletes the joint's key at the moment shown.",
  ctl_rest: "Puts the joint back to its rest pose at the moment shown.",
  ctl_undo: "Undoes the last change.",
  ctl_saveKeys:
    "Saves the animation; with the game of a try open, the studio builds it again with your version.",
  ctl_check:
    "What build-actor would stop on (red) or what is worth knowing (yellow), animation by animation.",
  ctl_export:
    "Writes the creation as build-actor needs it, with a README of the lines to add to the game.",
  ctl_readme: "Opens that README.",
  ctl_try:
    "Shows the model in the game's test level, playing the animation picked. Greyed out, with the reason under it, when the game cannot be started: no installation, another game open, Jak and Daxter.",
  ctl_close: "Closes the game; its own files are put back.",
  ctl_modelsDirs:
    "Each game's models, the decompiler's decompiler_out/<game>/levels folder. Read only.",
  ctl_creationsDir: "Where the studio keeps your creations.",
  ctl_gameDir:
    "Each game's installation: the OpenGOAL Launcher's folder, an OpenGOAL release or a jak-project. The game the studio tries models in, and where exports go.",
  ctl_language: "English or French.",
  ctl_mcp: "The command that adds the studio's MCP server to Claude Code.",
  guideUseOpened: "Make the lines for the model opened: {model}",
  guideTitle: "Put a model in the game",
  guideIntro:
    "These steps put {model} in {game}'s test level as an entity of its own, which plays its animations. Every line is made for this model, the one last opened in the studio.",
  guideIntroExample:
    "These steps put a model in the game's test level as an entity of its own, which plays its animations. The example is Jak 3's Marauder, with a wave made in the studio.",
  guideNeed:
    "You need a jak-project where {game} builds, and the model exported with Export for {game}. The studio's Try in the game does all of this in a copy of the test level: these steps are for your own mod.",
  guideModel: "Put the model in place",
  guideModelText:
    "Copy the export's custom_assets folder into your jak-project: the model lands here. With the game's installation folder set in the settings, the export already put it there.",
  guideBuild: "Build its art group",
  guideBuildText:
    "In goal_src/{gameId}/game.gp, next to the other build-actor lines: the art group holds the model's skeleton and animations.",
  guideMesh: "Put its mesh in the level",
  guideMeshText:
    "In {level}/test-zone.jsonc, add the model to the list custom_models: the level's file holds its mesh.",
  guideCode: "Write the entity's code",
  guideCodeText:
    "A new file, goal_src/{gameId}/levels/test-zone/{type}.gc. def-actor names the animations in the file's order: each symbol is a slot of the art group. The entity loops on its first animation, and plays {anim} once when it gets the event '{event}.",
  guideEventAnim: "Animation the event plays",
  guideCompile: "Compile it with the game",
  guideCompileText:
    "In goal_src/{gameId}/game.gp: the code becomes an object file, {type}.o.",
  guideLoad: "Load both with the level",
  guideLoadText:
    "In {level}/testzone.gd, the level's list of files: the entity's code first, the art group before the level's own file (the lines already there stay in place of ...).",
  guidePlace: "Place the entity in the level",
  guidePlaceText:
    "In {level}/test-zone.jsonc, add it to the list actors: its etype is the entity's type, its name the one the code finds it by, trans its place in metres.",
  guidePlay: "Build and play",
  guidePlayText:
    "In goalc: build the game with (mi), start it, connect with (lt), go to the test level, then send the event: the entity plays {anim}.",
  guideNotBorn: "The entity does not appear?",
  guideNotBornText:
    "A level made by hand has no visibility data: with the game's PS2 actor visibility on (PS2 actor vis in the debug menu), its entities are never born. Turn it off, or in goalc:",
  guideMemory: "Every animation takes the game's memory",
  guideMemoryText:
    "A level loads each of its files, the art group whole, into a buffer of 2 MB in Jak and Daxter, 1 750 KB in Jak II and Jak 3, and checks nothing: a bigger art group writes over the memory after it, and the game crashes. Game check gives its size once built: keep the animations your entity plays. Past that, the game itself has to change, probably a mod of yours: the dgo-level-buf-2 buffers of goal_src/{gameId}/engine/level/level.gc, and the level's heap they come from (LEVEL_HEAP_SIZE).",
  guideJak1:
    "Jak and Daxter closes when a level loads an entity with a skeleton added this way (jak-project of October 2026): try your models in Jak II or Jak 3.",
  exportedTo: "Written to {path}",
  openReadme: "Open the README",
  savedToCreations: "Saved in the creations.",
  play: "Play",
  pause: "Pause",
  skeleton: "Skeleton",
  speed: "Speed",
  fromVideo: "Video",
  fromMocap: "Mocap file",
  looks: "Appearance",
  looksHint:
    "The model's variants, as the game's code draws them: Outfit picks one, the lists show or hide its parts. What is shown here never changes the model: the studio only edits animations.",
  lookJak: "Jak",
  lookDark: "Dark Jak",
  lookLight: "Light Jak",
  morphGun: "Morph Gun",
  gunMod_red: "Red",
  gunMod_yellow: "Yellow",
  gunMod_blue: "Blue",
  gunMod_dark: "Dark",
  armor: "Armor",
  armorTag: "armor",
  parts: "{n} parts",
  segments: "{n} segments (as the game turns them on)",
  seg_goatee: "Goatee",
  lightWings: "Wings (gliding)",
  seg_scarfUp: "Scarf over the mouth",
  outfit: "Outfit {n}",
  outfits: "of {n}",
  outfitRandom: "Random",
  shapes: "{n} blend shapes",
  shape: "Shape {n}",
  shapeDark: "Dark Jak {n}",
  shape_goggles: "Goggles",
  shape_scarf: "Scarf",
  noWings:
    "{name}.glb is not in the model's folder nor in the game's models folder.",
  noGameTextures:
    "The eyes and Dark Jak's skin are drawn from the game's textures (decompiler_out/<game>/textures), next to the models folder or in the game's installation folder of the settings.",
  fromManual: "Manual",
  manualHint:
    "Pose the joints yourself with the mouse, key by key, from the rest pose.",
  openEditor: "Open the animation editor",
  refine: "Refine by hand",
  editor: "Animation editor",
  editorHint:
    "Click a joint (a dot in the view) and turn it with its rings: a key is set at the time shown. Move along the time bar and pose it again: the game blends from key to key.",
  duration: "Length (s)",
  joint: "Joint",
  noJoint: "Pick a joint",
  rotateTool: "Turn (R)",
  moveTool: "Move (T)",
  addKey: "Key the joint as it is (K)",
  deleteKey: "Delete the joint's key here (Del)",
  restPose: "Back to the rest pose here",
  prevKey: "Previous key",
  nextKey: "Next key",
  undo: "Undo (Ctrl+Z)",
  jointKeys: "{n} keys on this joint",
  keyedJoints: "{n} joints with keys",
  editKeys: "Edit the keys",
  keyframes: "{n} key times",
  videoHint:
    "A video of one person, whole body in view. The first time, the pose model (about 9 MB) is downloaded.",
  mocapHint: "BVH, FBX or glTF with an animated skeleton (Mixamo, CMU...).",
  chooseFile: "Choose a file",
  clip: "Clip",
  name: "Name",
  fps: "Keys per second",
  fpsHint:
    "The game's own animations have 15 keys a second. More keeps finer detail of the movement in a heavier file; in the game, an animation's size depends on its length and on the joints that move (build-actor makes 60 frames a second whatever the keys).",
  smoothing: "Smoothing",
  faceForward: "Face forward",
  inPlace: "In place",
  inPlaceHint:
    "Unticked: the model travels during the animation, then goes back to its start at each loop.",
  mapping: "Bone mapping",
  extract: "Extract the movement",
  extracting: "Reading the video... {p}%",
  stop: "Stop",
  frames: "{n} frames, {s} s",
  save: "Save the animation",
  cancel: "Close",
  replaceConfirm: "Replace the animation {name}?",
  none: "None",
  slot_hips: "Hips",
  slot_chest: "Chest",
  slot_neck: "Neck",
  slot_head: "Head",
  slot_upperArmL: "Left upper arm",
  slot_foreArmL: "Left forearm",
  slot_handL: "Left hand",
  slot_upperArmR: "Right upper arm",
  slot_foreArmR: "Right forearm",
  slot_handR: "Right hand",
  slot_thighL: "Left thigh",
  slot_shinL: "Left shin",
  slot_footL: "Left foot",
  slot_thighR: "Right thigh",
  slot_shinR: "Right shin",
  slot_footR: "Right foot",
  modelsDirs: "Models folders",
  modelsDirHint:
    "For each game, the .glb files of its entities (models with an armature). They are only read.",
  creationsDir: "Creations folder",
  creationsDirHint:
    "Each model you change is copied here with its animations, ready to export.",
  game: "Game",
  gameDir: "Game installation folder: {game} (optional)",
  gameDirHint:
    "Where the game is installed: the studio tries models in it when started on its own (from the OpenGOAL Launcher, it uses the Launcher's games), and exports go into its custom_assets folder. Expected folders, for example:",
  gameDirLauncher:
    "the OpenGOAL Launcher's installation folder, the one that holds active: C:\\Games\\OpenGOAL",
  gameDirRelease:
    "an OpenGOAL release unzipped, the folder that holds gk.exe: C:\\Games\\opengoal-v0.3.9",
  gameDirProject:
    "a jak-project built with its game, your mod's: C:\\dev\\jak-project",
  language: "Language",
  choose: "Choose",
  clear: "Clear",
  notSet: "Not set",
  mcpTitle: "AI assistant (MCP)",
  mcpHint:
    "Lets Claude Code, or any MCP client, define game-ready animations. Register it once:",
  copy: "Copy",
  copied: "Copied",
  welcome: "Choose the models folder of a game to start.",
  videoUnreadable: "{file}: the video cannot be read.",
  noBody: "{file}: no body was found in the video.",
  mocapNoModel: "{model} could not be opened.",
  tryNoAnimation: "Pick the animation to try in the game.",
  holdWeapon: "Hands on the weapon",
  holdWeaponHint:
    "The arms keep the pose of the model's first animation, the weapon in both hands; the rest of the body follows the capture. Unticked: the arms follow the capture, the weapon the hand holding it.",
  noBodyMocap:
    "{file}: none of its bones was recognised as hips and legs (by their names).",
  notMocap: "{file}: a motion capture file is a .bvh, .fbx, .glb or .gltf.",
  "core_no-skin": "The model has no armature (skin): it is not an entity.",
  "core_several-skins":
    "The model has {count} armatures (skins), build-actor takes one.",
  core_skeleton: "{error}: it is not an entity.",
  "core_joint-order":
    "Joint '{joint}' comes before its parent in the armature: build-actor needs every parent first.",
  "core_joint-child":
    "Joint '{joint}' has a child that is not a joint ('{child}'): build-actor stops on it. Parent that object to the armature instead.",
  "core_no-align":
    "The first joint is not 'align': build-actor adds one, so the game's joint numbers are one more than the file's, and 'align' (which moves the entity in game) cannot be animated.",
  "core_align-origin": "Joint 'align' is not at the origin.",
  "core_too-many-joints":
    "The armature has {count} joints, build-actor takes {max} at most.",
  "core_too-few-joints": "build-actor needs at least two joints.",
  "core_skin-joints":
    "{outside} vertex weights go to joints the armature does not have (up to joint {highest}, it has {joints}): build-actor passes them on, and the game moves them with other joints.",
  "core_model-name":
    "The game names the art group after the file: '{suggested}' would be safer than '{name}' (lowercase, digits and dashes).",
  "core_no-animation":
    "The model has no animation: build-actor makes a still pose of it.",
  "core_duplicate-name": "Two animations are named '{name}'.",
  core_empty: "The animation has no keys.",
  core_weights:
    "It has morph target (weights) keys on {target}: build-actor stops on them. Export removes them.",
  "core_not-a-joint":
    "It moves node {target}, which is not a joint of the armature: build-actor stops on it. Export drops these keys.",
  "core_cubic-spline":
    "Its keys on {target} are cubic splines: build-actor reads linear keys only. Export makes them linear.",
  core_step: "Its keys on {target} are steps: the game blends between them.",
  core_keys: "Its keys on {target} cannot be read: {error}.",
  "core_starts-late":
    "Its keys on {target} start at {time} s: the game starts at 0.",
  "core_missing-keys":
    "{count} joint channels have no keys ({list}): the game puts them at the origin instead of their rest pose. Export gives them their rest pose.",
  "core_frame-too-big":
    "One frame takes {bytes} bytes, build-actor allows {max}: move fewer joints (a channel whose keys never change costs nothing per frame).",
  "core_fixed-too-big":
    "Its constant part takes {bytes} bytes, build-actor allows {max}.",
  "core_weights-removed": "{anim}: morph target (weights) keys removed",
  "core_node-dropped": "{anim}: keys of node {node} (not a joint) dropped",
  "core_made-linear": "{anim}: {kind} keys made linear",
  "core_rest-keyed": "{anim}: joints without keys keyed at their rest pose",
  "core_still-pose-held":
    "{anim}: a still pose, given a second key 1/60 s later (build-actor stops on it otherwise)",
  "core_no-game-folder":
    "Choose the game's installation folder in the settings, or start the studio from the OpenGOAL Launcher.",
  "core_game-not-in-launcher":
    "This game is not installed with the OpenGOAL Launcher's active tooling ({version}): install or update it in the Launcher.",
  "core_game-not-found":
    "{folder} holds no game to try in: choose an OpenGOAL Launcher installation, an OpenGOAL release or a jak-project with the game built.",
  "core_game-incomplete":
    "The game in {folder} lacks {file}: install or build it again.",
  "core_game-port-taken":
    "Another game listens on port {port}: close it first.",
  "core_game-step-failed": "{step} failed: {output}",
  "core_game-closed": "The game closed during the try: {output}",
  "core_game-not-playing":
    "The game does not play {anim}: the art group's element {slot} is another animation.",
  "core_try-animation-missing": "{model} has no animation {anim} to try.",
  "core_try-jak1":
    "Jak and Daxter closes when a level loads a character added this way: try models in Jak II or Jak 3.",
  "core_art-group-too-big":
    "The model takes {size} KB once built, more than the {max} KB a level of the game loads it into: remove or shorten animations.",
  "core_animations-too-big":
    "The animations take {size} KB once built, the mesh not counted: more than a level of any game loads an art group into (2048 KB in Jak and Daxter, 1750 KB in Jak II and Jak 3). Remove or shorten animations.",
  "core_animations-too-big-23":
    "The animations take {size} KB once built, the mesh not counted: more than a level of Jak II or Jak 3 loads an art group into (1750 KB; 2048 KB in Jak and Daxter). Remove or shorten animations for those games.",
  core_unreadable: "{file} is not a usable .glb",
  "core_not-an-entity":
    "{file} is not an entity: an entity has one armature (skin).",
  "core_creation-exists": "A creation is already named '{name}'.",
  "core_animation-unnamed": "An animation needs a name.",
  "core_animation-exists": "An animation is already named '{name}'.",
  "core_no-such-animation": "There is no animation '{name}'.",
  "core_game-name-shared":
    "In the game, '{anim}' would be named {name}, as '{other}' is: give it another name.",
};

export type Key = keyof typeof en;

const fr: Record<Key, string> = {
  navStudio: "Studio",
  navSettings: "Paramètres",
  minimize: "Réduire",
  maximize: "Agrandir",
  close: "Fermer",
  tabModels: "Modèles",
  tabCreations: "Créations",
  search: "Rechercher",
  variants: "{n} variantes",
  tag_highres: "Haute définition",
  tag_lowres: "Basse résolution",
  tag_simple: "Simplifié",
  tag_rider: "En véhicule",
  tag_escort: "Escorte",
  tag_main: "Principal",
  import: "Importer des modèles (.glb) dans les créations",
  refreshList: "Relire la liste",
  importReplace:
    "Une création s'appelle déjà {name}. La remplacer ? Ses animations seront perdues.",
  noModelsDir:
    "Choisissez le dossier des modèles de {game} dans les paramètres.",
  listingModels: "Lecture des modèles du jeu… {p} %",
  noModels:
    "Aucune entité ici : seules les entités vivantes du jeu (personnages, créatures, ennemis) sont listées.",
  noCreations:
    "Rien pour l'instant : ce que vous changez dans un modèle est enregistré ici, l'original n'est jamais modifié.",
  pickModel:
    "Choisissez une entité à gauche pour la voir et jouer ses animations.",
  joints: "{n} joints",
  anims: "{n} animations",
  creation: "Création",
  original: "Original",
  openFolder: "Afficher dans le dossier",
  animations: "Animations",
  noAnimation: "Aucune animation pour l'instant.",
  newAnimation: "Nouvelle animation",
  twinAnimations: "Ajouter {n} animations des modèles au même squelette",
  rename: "Renommer",
  delete: "Supprimer",
  deleteConfirm: "Supprimer l'animation {name} ?",
  gameCheck: "Vérification pour le jeu",
  allGood: "Prêt pour build-actor.",
  gameAnimation: "Jeu",
  gameAnimations:
    "Ces {n} animations sont celles du jeu (Jak et Daxter sont dans tous les niveaux) : seules celles que vous ajoutez sont vérifiées et exportées.",
  exportGame: "Exporter pour {game}",
  tryInGame: "Essayer dans le jeu",
  tryStep_export: "Écriture du modèle",
  tryStep_compiler:
    "Démarrage du compilateur du jeu (une à deux minutes la première fois)",
  tryStep_build: "Construction du niveau de test",
  tryStep_start: "Démarrage du jeu",
  tryStep_load: "Jak va au niveau de test",
  tryStep_play: "Affichage du modèle",
  inGame:
    "Dans le jeu : l'animation choisie ici y joue aussi. Celle que le jeu n'a pas encore, ou celle que vous modifiez, est reconstruite (le jeu redémarre).",
  closeGame: "Fermer le jeu",
  navGuide: "Guide",
  guideUsage: "Utilisation de l'outil",
  guideUsageIntro:
    "Ce que fait le studio, de la recherche d'un modèle jusqu'à le voir dans le jeu. Le second onglet met un modèle dans votre mod, étape par étape.",
  useBrowse: "La liste des modèles",
  useBrowseText:
    "La liste de gauche contient les entités vivantes du jeu (personnages, créatures, ennemis) et vos créations, les copies que le studio garde de ce que vous modifiez. Choisissez-en une pour l'ouvrir.",
  useLooks: "Apparence",
  useLooksText:
    "Le modèle tel que le code du jeu le dessine. Rien ici ne modifie le modèle ni ses textures : le studio ne modifie que les animations.",
  useNew: "Créer une animation",
  useNewText:
    "Nouvelle animation met un mouvement sur le modèle : depuis une vidéo d'une personne, un fichier de capture de mouvement (BVH, FBX, glTF), ou à la main. Elle est enregistrée dans la création du modèle : le modèle d'origine ne change jamais.",
  useKeys: "Modifier les clés",
  useKeysText:
    "Le clap à côté d'une animation ouvre ses clés : choisissez un joint, tournez-le ou déplacez-le, et une clé est posée au moment affiché. K pose une clé sur le joint tel quel, Suppr efface sa clé, Ctrl+Z annule.",
  useCheck: "Vérifier et exporter",
  useCheckText:
    "Vérification pour le jeu liste ce sur quoi build-actor s'arrêterait ou qu'il construirait mal. Exporter écrit la création comme build-actor la veut, avec un README des lignes à ajouter au jeu. Les animations de Jak et de Daxter sont celles du jeu : seules celles que vous ajoutez sont vérifiées et exportées.",
  useTry: "L'essayer dans le jeu",
  useTryText:
    "Essayer dans le jeu construit le modèle dans le niveau de test du jeu et lance le jeu : le modèle joue l'animation choisie, puis celle que vous choisissez ensuite. Les fichiers du jeu sont remis en place à sa fermeture.",
  useSettings: "Réglages",
  useSettingsText:
    "Les dossiers avec lesquels le studio travaille, la langue et l'assistant IA.",
  useMcp: "Avec un assistant IA",
  useMcpText:
    "Les réglages donnent la commande qui ajoute le serveur MCP du studio à Claude Code. L'assistant liste alors les modèles, crée des animations depuis vos mots ou une capture de mouvement, et les vérifie dans le jeu ; le studio montre ce qu'il fait.",
  useLog: "Quand quelque chose ne va pas",
  useLogText:
    "Chaque erreur est écrite dans errors.log, dans le dossier du studio ; l'essai dans le jeu y écrit ce qu'ont dit goalc et le jeu dans game-test/<jeu>/goalc.log et gk.log.",
  useLogOpen: "Ouvrir le dossier du studio",
  guideContents: "Sommaire",
  seeMore: "Voir plus",
  themeFind: "Trouver un modèle",
  themeAnimate: "Animer",
  themeLooks: "L'apparence du modèle",
  themeGame: "Vers le jeu",
  themeSetup: "Réglages et aide",
  useWatch: "Regarder un modèle",
  useWatchText:
    "Le modèle joue sa première animation dans la vue ; la barre sous la vue la pilote.",
  useAnims: "Les animations",
  useAnimsText:
    "Le panneau de droite liste les animations du modèle : cliquez sur l'une d'elles pour la jouer. Les boutons d'une ligne apparaissent au survol de la souris.",
  timeBar: "Barre de temps",
  viewMouse: "La vue",
  ctl_games: "Le jeu dont les modèles sont listés.",
  ctl_tabs:
    "Les entités du jeu, ou vos créations : les copies que le studio garde des modèles que vous avez modifiés (l'original ne change jamais).",
  ctl_search: "Trouve une entité par son nom officiel ou celui de son fichier.",
  ctl_refresh:
    "Relit la liste : une création faite par l'assistant IA, ou une nouvelle extraction du jeu.",
  ctl_import: "Copie vos propres fichiers .glb dans les créations.",
  ctl_variants:
    "Une entité que le jeu sépare en plusieurs modèles (haute définition, dans un véhicule...) : la flèche les ouvre.",
  ctl_play: "Joue ou met en pause l'animation.",
  ctl_timeBar:
    "Amène l'animation à un moment ; chaque trait au-dessus est une clé, les orange celles du joint choisi.",
  ctl_speed: "Joue plus lentement (0,25x, 0,5x) ou plus vite (2x).",
  ctl_skeleton: "Montre les joints et les os par-dessus le modèle.",
  ctl_view:
    "Glissez pour tourner autour du modèle, clic droit pour le déplacer, la molette pour zoomer.",
  ctl_badge:
    "Original : le modèle du jeu, en lecture seule. Création : votre copie, celle qui est exportée.",
  ctl_folder: "Ouvre le dossier du fichier du modèle.",
  ctl_gameTag:
    "Une animation de Jak ou de Daxter : le jeu l'a déjà, elle n'est donc ni vérifiée ni exportée ; elle peut être essayée dans le jeu, et modifiée pour en faire votre version.",
  ctl_editKeys: "Ouvre l'animation dans l'éditeur de clés.",
  ctl_rename: "Renomme l'animation (dans la création).",
  ctl_delete: "Supprime l'animation (de la création).",
  ctl_twins:
    "Ajoute les animations que d'autres fichiers de la même entité et du même squelette contiennent, là où l'extraction les a mises.",
  ctl_new:
    "Crée une animation depuis une vidéo, un fichier de capture de mouvement ou à la main.",
  ctl_forms:
    "Jak lui-même, en Dark Jak (formes, peau, yeux noirs) ou en Light Jak (les lumières du jeu).",
  ctl_wings: "Les ailes de Light Jak, sorties seulement quand il plane.",
  ctl_gun: "Met le Morph Gun dans la main de Jak, avec le mod de votre choix.",
  ctl_armor:
    "Montre ou cache les pièces d'armure d'un modèle sans les segments du jeu.",
  ctl_outfit:
    "Chaque apparence que le code du jeu donne au modèle (couleurs, chapeaux, armes...) ; Au hasard en tire une.",
  ctl_segments: "Les pièces du modèle une par une, à montrer ou cacher.",
  ctl_shapes:
    "Les formes de mélange (celles de Dark Jak, les lunettes et le foulard de Jak), de 0 à 1.",
  ctl_source:
    "D'où vient le mouvement : une vidéo d'une personne, un fichier BVH, FBX ou glTF, ou rien (la pose de repos, à animer à la main).",
  ctl_fps:
    "Le nombre de clés par seconde de l'animation : 15 par défaut, comme les animations du jeu ; plus est plus fin, et plus lourd.",
  ctl_extract:
    "Lit le mouvement et le met sur le modèle, à côté de la silhouette orange de la capture.",
  ctl_smoothing: "Calme une capture qui tremble.",
  ctl_face: "Tourne le mouvement face à l'avant du modèle.",
  ctl_inPlace:
    "Garde le modèle sur place, comme le veut le jeu : son code le déplace dans le niveau.",
  ctl_hold:
    "Garde une arme dans les deux mains, avec les bras de la première animation du modèle.",
  ctl_mapping: "Quel joint du modèle joue quelle partie du corps.",
  ctl_refine: "Continue dans l'éditeur de clés.",
  ctl_save: "Enregistre l'animation dans la création du modèle.",
  ctl_duration: "La durée de l'animation, en secondes.",
  ctl_joint:
    "Le joint à bouger ; cliquer sur son point dans la vue le choisit aussi.",
  ctl_tools:
    "Tourne le joint avec ses anneaux, ou le déplace avec ses flèches : une clé est posée au moment affiché.",
  ctl_steps: "Va à la clé précédente ou suivante.",
  ctl_addKey: "Pose une clé sur le joint tel qu'il est, au moment affiché.",
  ctl_deleteKey: "Efface la clé du joint au moment affiché.",
  ctl_rest: "Remet le joint dans sa pose de repos au moment affiché.",
  ctl_undo: "Annule la dernière modification.",
  ctl_saveKeys:
    "Enregistre l'animation ; si le jeu d'un essai est ouvert, le studio le reconstruit avec votre version.",
  ctl_check:
    "Ce sur quoi build-actor s'arrêterait (rouge) ou ce qui est bon à savoir (jaune), animation par animation.",
  ctl_export:
    "Écrit la création comme build-actor la veut, avec un README des lignes à ajouter au jeu.",
  ctl_readme: "Ouvre ce README.",
  ctl_try:
    "Montre le modèle dans le niveau de test du jeu, en train de jouer l'animation choisie. Grisé, avec la raison en dessous, quand le jeu ne peut pas être lancé : pas d'installation, un autre jeu ouvert, Jak and Daxter.",
  ctl_close: "Ferme le jeu ; ses propres fichiers sont remis en place.",
  ctl_modelsDirs:
    "Les modèles de chaque jeu, le dossier decompiler_out/<jeu>/levels du décompilateur. En lecture seule.",
  ctl_creationsDir: "Là où le studio garde vos créations.",
  ctl_gameDir:
    "L'installation de chaque jeu : le dossier de l'OpenGOAL Launcher, une version d'OpenGOAL ou un jak-project. Le jeu où le studio essaie les modèles, et là où vont les exports.",
  ctl_language: "Anglais ou français.",
  ctl_mcp: "La commande qui ajoute le serveur MCP du studio à Claude Code.",
  guideUseOpened: "Faire les lignes pour le modèle ouvert : {model}",
  guideTitle: "Mettre un modèle dans le jeu",
  guideIntro:
    "Ces étapes mettent {model} dans le niveau de test de {game}, comme une entité à part entière qui joue ses animations. Chaque ligne est faite pour ce modèle, le dernier ouvert dans le studio.",
  guideIntroExample:
    "Ces étapes mettent un modèle dans le niveau de test du jeu, comme une entité à part entière qui joue ses animations. L'exemple est le Marauder de Jak 3, avec un salut (wave) fait dans le studio.",
  guideNeed:
    "Il vous faut un jak-project où {game} se compile, et le modèle exporté avec Exporter pour {game}. L'essai dans le jeu du studio fait tout cela dans une copie du niveau de test : ces étapes sont pour votre propre mod.",
  guideModel: "Mettre le modèle en place",
  guideModelText:
    "Copiez le dossier custom_assets de l'export dans votre jak-project : le modèle arrive ici. Avec le répertoire d'installation du jeu choisi dans les réglages, l'export l'y a déjà mis.",
  guideBuild: "Construire son art group",
  guideBuildText:
    "Dans goal_src/{gameId}/game.gp, à côté des autres lignes build-actor : l'art group contient le squelette et les animations du modèle.",
  guideMesh: "Mettre son maillage dans le niveau",
  guideMeshText:
    "Dans {level}/test-zone.jsonc, ajoutez le modèle à la liste custom_models : le fichier du niveau contient son maillage.",
  guideCode: "Écrire le code de l'entité",
  guideCodeText:
    "Un nouveau fichier, goal_src/{gameId}/levels/test-zone/{type}.gc. def-actor nomme les animations dans l'ordre du fichier : chaque symbole est un emplacement de l'art group. L'entité joue sa première animation en boucle, et joue {anim} une fois quand elle reçoit l'événement '{event}.",
  guideEventAnim: "Animation jouée par l'événement",
  guideCompile: "La compiler avec le jeu",
  guideCompileText:
    "Dans goal_src/{gameId}/game.gp : le code devient un fichier objet, {type}.o.",
  guideLoad: "Charger les deux avec le niveau",
  guideLoadText:
    "Dans {level}/testzone.gd, la liste des fichiers du niveau : le code de l'entité en premier, l'art group avant le fichier du niveau lui-même (les lignes déjà présentes restent à la place de ...).",
  guidePlace: "Placer l'entité dans le niveau",
  guidePlaceText:
    "Dans {level}/test-zone.jsonc, ajoutez-la à la liste actors : son etype est le type de l'entité, son nom celui par lequel le code la trouve, trans sa place en mètres.",
  guidePlay: "Compiler et jouer",
  guidePlayText:
    "Dans goalc : compilez le jeu avec (mi), lancez-le, connectez-vous avec (lt), allez au niveau de test, puis envoyez l'événement : l'entité joue {anim}.",
  guideNotBorn: "L'entité n'apparaît pas ?",
  guideNotBornText:
    "Un niveau fait à la main n'a pas de données de visibilité : avec la visibilité PS2 des acteurs activée dans le jeu (PS2 actor vis dans le menu de debug), ses entités ne naissent jamais. Désactivez-la, ou dans goalc :",
  guideMemory: "Chaque animation prend de la mémoire du jeu",
  guideMemoryText:
    "Un niveau charge chacun de ses fichiers, l'art group en entier, dans un tampon de 2 Mo dans Jak and Daxter, 1 750 Ko dans Jak II et Jak 3, et ne vérifie rien : un art group plus gros écrit par-dessus la mémoire qui suit, et le jeu plante. La Vérification pour le jeu donne sa taille une fois construit : gardez les animations que votre entité joue. Au-delà, c'est le jeu lui-même qu'il faut changer, sans doute un mod de votre part : les tampons dgo-level-buf-2 de goal_src/{gameId}/engine/level/level.gc, et le tas du niveau d'où ils viennent (LEVEL_HEAP_SIZE).",
  guideJak1:
    "Jak and Daxter se ferme quand un niveau charge une entité avec un squelette ajoutée ainsi (jak-project d'octobre 2026) : essayez vos modèles dans Jak II ou Jak 3.",
  exportedTo: "Écrit dans {path}",
  openReadme: "Ouvrir le README",
  savedToCreations: "Enregistré dans les créations.",
  play: "Lecture",
  pause: "Pause",
  skeleton: "Squelette",
  speed: "Vitesse",
  fromVideo: "Vidéo",
  fromMocap: "Fichier mocap",
  looks: "Apparence",
  looksHint:
    "Les variantes du modèle, telles que le code du jeu les dessine : Tenue en choisit une, les listes montrent ou cachent ses pièces. Ce qui est montré ici ne change jamais le modèle : le studio ne modifie que les animations.",
  lookJak: "Jak",
  lookDark: "Dark Jak",
  lookLight: "Light Jak",
  morphGun: "Arme (Morph Gun)",
  gunMod_red: "Rouge",
  gunMod_yellow: "Jaune",
  gunMod_blue: "Bleu",
  gunMod_dark: "Sombre",
  armor: "Armure",
  armorTag: "armure",
  parts: "{n} pièces",
  segments: "{n} segments (comme le jeu les active)",
  seg_goatee: "Bouc",
  lightWings: "Ailes (vol plané)",
  seg_scarfUp: "Écharpe sur la bouche",
  outfit: "Tenue {n}",
  outfits: "sur {n}",
  outfitRandom: "Au hasard",
  shapes: "{n} formes de mélange",
  shape: "Forme {n}",
  shapeDark: "Dark Jak {n}",
  shape_goggles: "Lunettes",
  shape_scarf: "Écharpe",
  noWings:
    "{name}.glb n'est ni dans le dossier du modèle ni dans le dossier des modèles du jeu.",
  noGameTextures:
    "Les yeux et la peau de Dark Jak sont dessinés avec les textures du jeu (decompiler_out/<jeu>/textures), à côté du dossier de modèles ou dans le répertoire d'installation du jeu des réglages.",
  fromManual: "Manuel",
  manualHint:
    "Posez vous-même les jointures à la souris, clé par clé, depuis la pose de repos.",
  openEditor: "Ouvrir l'éditeur d'animation",
  refine: "Retoucher à la main",
  editor: "Éditeur d'animation",
  editorHint:
    "Cliquez une jointure (un point dans la vue) et tournez-la avec ses anneaux : une clé est posée à l'instant affiché. Avancez sur la barre de temps et posez-la à nouveau : le jeu passe d'une clé à l'autre.",
  duration: "Durée (s)",
  joint: "Jointure",
  noJoint: "Choisir une jointure",
  rotateTool: "Tourner (R)",
  moveTool: "Déplacer (T)",
  addKey: "Poser une clé sur la jointure telle quelle (K)",
  deleteKey: "Supprimer la clé de la jointure ici (Suppr)",
  restPose: "Revenir à la pose de repos ici",
  prevKey: "Clé précédente",
  nextKey: "Clé suivante",
  undo: "Annuler (Ctrl+Z)",
  jointKeys: "{n} clés sur cette jointure",
  keyedJoints: "{n} jointures avec des clés",
  editKeys: "Modifier les clés",
  keyframes: "{n} instants clés",
  videoHint:
    "Une vidéo d'une seule personne, le corps entier visible. La première fois, le modèle de pose (environ 9 Mo) est téléchargé.",
  mocapHint: "BVH, FBX ou glTF avec un squelette animé (Mixamo, CMU...).",
  chooseFile: "Choisir un fichier",
  clip: "Clip",
  name: "Nom",
  fps: "Clés par seconde",
  fpsHint:
    "Les animations du jeu ont 15 clés par seconde. Plus garde le mouvement plus fin dans un fichier plus lourd ; dans le jeu, la taille d'une animation dépend de sa durée et des joints qui bougent (build-actor fait 60 images par seconde quelles que soient les clés).",
  smoothing: "Lissage",
  faceForward: "Face à l'avant",
  inPlace: "Sur place",
  inPlaceHint:
    "Décoché : le modèle avance pendant l'animation, puis revient au départ à chaque boucle.",
  mapping: "Correspondance des os",
  extract: "Extraire le mouvement",
  extracting: "Lecture de la vidéo... {p} %",
  stop: "Arrêter",
  frames: "{n} images, {s} s",
  save: "Enregistrer l'animation",
  cancel: "Fermer",
  replaceConfirm: "Remplacer l'animation {name} ?",
  none: "Aucun",
  slot_hips: "Bassin",
  slot_chest: "Torse",
  slot_neck: "Cou",
  slot_head: "Tête",
  slot_upperArmL: "Bras gauche",
  slot_foreArmL: "Avant-bras gauche",
  slot_handL: "Main gauche",
  slot_upperArmR: "Bras droit",
  slot_foreArmR: "Avant-bras droit",
  slot_handR: "Main droite",
  slot_thighL: "Cuisse gauche",
  slot_shinL: "Jambe gauche",
  slot_footL: "Pied gauche",
  slot_thighR: "Cuisse droite",
  slot_shinR: "Jambe droite",
  slot_footR: "Pied droit",
  modelsDirs: "Dossiers des modèles",
  modelsDirHint:
    "Pour chaque jeu, les fichiers .glb de ses entités (modèles avec une armature). Ils sont seulement lus.",
  creationsDir: "Dossier des créations",
  creationsDirHint:
    "Chaque modèle que vous changez est copié ici avec ses animations, prêt à exporter.",
  game: "Jeu",
  gameDir: "Répertoire d'installation du jeu : {game} (facultatif)",
  gameDirHint:
    "Là où le jeu est installé : le studio y essaie les modèles quand il est lancé seul (depuis l'OpenGOAL Launcher, il prend les jeux du Launcher), et les exports vont dans son dossier custom_assets. Répertoires attendus, par exemple :",
  gameDirLauncher:
    "le dossier d'installation de l'OpenGOAL Launcher, celui qui contient active : C:\\Jeux\\OpenGOAL",
  gameDirRelease:
    "une version d'OpenGOAL décompressée, le dossier qui contient gk.exe : C:\\Jeux\\opengoal-v0.3.9",
  gameDirProject:
    "un jak-project compilé avec son jeu, celui de votre mod : C:\\dev\\jak-project",
  language: "Langue",
  choose: "Choisir",
  clear: "Retirer",
  notSet: "Non défini",
  mcpTitle: "Assistant IA (MCP)",
  mcpHint:
    "Permet à Claude Code, ou à tout client MCP, de définir des animations prêtes pour le jeu. Enregistrez-le une fois :",
  copy: "Copier",
  copied: "Copié",
  welcome: "Choisissez le dossier des modèles d'un jeu pour commencer.",
  videoUnreadable: "{file} : la vidéo ne peut pas être lue.",
  noBody: "{file} : aucun corps n'a été trouvé dans la vidéo.",
  mocapNoModel: "{model} n'a pas pu être ouvert.",
  tryNoAnimation: "Choisissez l'animation à essayer dans le jeu.",
  holdWeapon: "Mains sur l'arme",
  holdWeaponHint:
    "Les bras gardent la pose de la première animation du modèle, l'arme dans les deux mains ; le reste du corps suit la capture. Décoché : les bras suivent la capture, l'arme la main qui la tient.",
  noBodyMocap:
    "{file} : aucun de ses os n'a été reconnu comme hanches et jambes (d'après leurs noms).",
  notMocap:
    "{file} : un fichier de capture de mouvement est un .bvh, .fbx, .glb ou .gltf.",
  "core_no-skin":
    "Le modèle n'a pas d'armature (skin) : ce n'est pas une entité.",
  "core_several-skins":
    "Le modèle a {count} armatures (skins), build-actor n'en prend qu'une.",
  core_skeleton: "{error} : ce n'est pas une entité.",
  "core_joint-order":
    "Le joint « {joint} » vient avant son parent dans l'armature : build-actor veut chaque parent en premier.",
  "core_joint-child":
    "Le joint « {joint} » a un enfant qui n'est pas un joint (« {child} ») : build-actor s'arrête dessus. Rattachez plutôt cet objet à l'armature.",
  "core_no-align":
    "Le premier joint n'est pas « align » : build-actor en ajoute un, les numéros des joints dans le jeu sont donc décalés d'un, et « align » (qui déplace l'entité dans le jeu) ne peut pas être animé.",
  "core_align-origin": "Le joint « align » n'est pas à l'origine.",
  "core_too-many-joints":
    "L'armature a {count} joints, build-actor en prend {max} au plus.",
  "core_too-few-joints": "build-actor a besoin d'au moins deux joints.",
  "core_skin-joints":
    "{outside} poids de sommets vont à des joints que l'armature n'a pas (jusqu'au joint {highest}, elle en a {joints}) : build-actor les transmet, et le jeu déplace ces sommets avec d'autres joints.",
  "core_model-name":
    "Le jeu nomme l'art group d'après le fichier : « {suggested} » serait plus sûr que « {name} » (minuscules, chiffres et tirets).",
  "core_no-animation":
    "Le modèle n'a pas d'animation : build-actor en fait une pose fixe.",
  "core_duplicate-name": "Deux animations s'appellent « {name} ».",
  core_empty: "L'animation n'a pas de clés.",
  core_weights:
    "Elle a des clés de morphing (weights) sur {target} : build-actor s'arrête dessus. L'export les retire.",
  "core_not-a-joint":
    "Elle déplace le nœud {target}, qui n'est pas un joint de l'armature : build-actor s'arrête dessus. L'export retire ces clés.",
  "core_cubic-spline":
    "Ses clés sur {target} sont des splines cubiques : build-actor ne lit que des clés linéaires. L'export les rend linéaires.",
  core_step:
    "Ses clés sur {target} sont en paliers : le jeu interpole entre elles.",
  core_keys: "Ses clés sur {target} ne peuvent pas être lues : {error}.",
  "core_starts-late":
    "Ses clés sur {target} commencent à {time} s : le jeu commence à 0.",
  "core_missing-keys":
    "{count} canaux de joints n'ont pas de clés ({list}) : le jeu les met à l'origine au lieu de leur pose de repos. L'export leur donne leur pose de repos.",
  "core_frame-too-big":
    "Une image prend {bytes} octets, build-actor en permet {max} : animez moins de joints (un canal dont les clés ne changent jamais ne coûte rien par image).",
  "core_fixed-too-big":
    "Sa partie constante prend {bytes} octets, build-actor en permet {max}.",
  "core_weights-removed": "{anim} : clés de morphing (weights) retirées",
  "core_node-dropped": "{anim} : clés du nœud {node} (pas un joint) retirées",
  "core_made-linear": "{anim} : clés {kind} rendues linéaires",
  "core_rest-keyed": "{anim} : joints sans clés mis à leur pose de repos",
  "core_still-pose-held":
    "{anim} : une pose fixe, avec une seconde clé 1/60 s plus tard (build-actor s'y arrête sinon)",
  "core_no-game-folder":
    "Choisissez le répertoire d'installation du jeu dans les réglages, ou lancez le studio depuis l'OpenGOAL Launcher.",
  "core_game-not-in-launcher":
    "Ce jeu n'est pas installé avec l'outillage actif de l'OpenGOAL Launcher ({version}) : installez-le ou mettez-le à jour dans le Launcher.",
  "core_game-not-found":
    "{folder} ne contient pas de jeu où essayer : choisissez une installation de l'OpenGOAL Launcher, une version d'OpenGOAL ou un jak-project avec le jeu compilé.",
  "core_game-incomplete":
    "Il manque {file} au jeu de {folder} : réinstallez-le ou recompilez-le.",
  "core_game-port-taken":
    "Un autre jeu écoute sur le port {port} : fermez-le d'abord.",
  "core_game-step-failed": "{step} a échoué : {output}",
  "core_game-closed": "Le jeu s'est fermé pendant l'essai : {output}",
  "core_game-not-playing":
    "Le jeu ne joue pas {anim} : l'élément {slot} du groupe d'art est une autre animation.",
  "core_try-animation-missing": "{model} n'a pas d'animation {anim} à essayer.",
  "core_try-jak1":
    "Jak and Daxter se ferme quand un niveau charge un personnage ajouté de cette façon : essayez les modèles dans Jak II ou Jak 3.",
  "core_art-group-too-big":
    "Le modèle prend {size} Ko une fois construit, plus que les {max} Ko dans lesquels un niveau du jeu le charge : retirez ou raccourcissez des animations.",
  "core_animations-too-big":
    "Les animations prennent {size} Ko une fois construites, sans compter le maillage : plus que ce qu'un niveau de n'importe quel jeu charge pour un art group (2048 Ko dans Jak and Daxter, 1750 Ko dans Jak II et Jak 3). Retirez ou raccourcissez des animations.",
  "core_animations-too-big-23":
    "Les animations prennent {size} Ko une fois construites, sans compter le maillage : plus que ce qu'un niveau de Jak II ou Jak 3 charge pour un art group (1750 Ko ; 2048 Ko dans Jak and Daxter). Retirez ou raccourcissez des animations pour ces jeux.",
  core_unreadable: "{file} n'est pas un .glb utilisable",
  "core_not-an-entity":
    "{file} n'est pas une entité : une entité a une seule armature (skin).",
  "core_creation-exists": "Une création s'appelle déjà « {name} ».",
  "core_animation-unnamed": "Une animation a besoin d'un nom.",
  "core_animation-exists": "Une animation s'appelle déjà « {name} ».",
  "core_no-such-animation": "Il n'y a pas d'animation « {name} ».",
  "core_game-name-shared":
    "Dans le jeu, « {anim} » s'appellerait {name}, comme « {other} » : donnez-lui un autre nom.",
};

export const LANGS = { en: "English", fr: "Français" } as const;
export type Lang = keyof typeof LANGS;

export const i18n = $state({
  lang: (navigator.language.startsWith("fr") ? "fr" : "en") as Lang,
});

export function t(key: Key, vars: Record<string, string | number> = {}) {
  const text = (i18n.lang === "fr" ? fr : en)[key];
  return text.replace(/\{(\w+)\}/g, (_, k) => String(vars[k] ?? `{${k}}`));
}

/** A text of the core (a game check issue, an export note, an error): its code picks the words in
 * the interface's language; its message, in English, stands in for a code missing here. */
export function coreText(text: {
  code?: string;
  args?: Record<string, string>;
  message?: string;
}) {
  const key = `core_${text.code}`;
  return key in en ? t(key as Key, text.args) : (text.message ?? "");
}

/** What went wrong, from the core or the interface, in the interface's language. */
export function errorText(e: unknown) {
  return e && typeof e === "object" && "message" in e
    ? coreText(e as { message: string })
    : String(e);
}
