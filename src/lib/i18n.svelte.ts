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
  tag_cutscene: "Cutscene",
  tag_lowres: "Low detail",
  tag_simple: "Simple",
  tag_rider: "In a vehicle",
  tag_escort: "Escort",
  tag_minigame: "Minigame",
  tag_main: "Main",
  import: "Import models (.glb) into the creations",
  importReplace:
    "A creation is already named {name}. Replace it? Its animations will be lost.",
  noModelsDir: "Choose the models folder of {game} in the settings.",
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
  twinAnimationsHint:
    "The extraction wrote this entity's animations to other files: {files}",
  rename: "Rename",
  delete: "Delete",
  deleteConfirm: "Delete the animation {name}?",
  gameCheck: "Game check",
  allGood: "Ready for build-actor.",
  exportGame: "Export for {game}",
  tryInGame: "Try in the game",
  tryInGameHint:
    "Builds the model as a custom actor and shows it in {game}, in its test level (test-zone), playing the animation picked. Needs the game installed: the OpenGOAL Launcher's, or the jak-project folder of the settings. The game's own files are put back when the game closes.",
  tryStep_export: "Writing the model",
  tryStep_compiler:
    "Starting the game's compiler (a minute or two the first time)",
  tryStep_build: "Building the test level",
  tryStep_start: "Starting the game",
  tryStep_load: "Taking Jak to the test level",
  tryStep_play: "Showing the model",
  inGame: "In the game: the animation picked here plays there too.",
  closeGame: "Close the game",
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
    "The model's variants: parts to show or hide, blend shapes. Applying makes the look the creation's own, the one the game builds (Light Jak's wings are a model of their own, shown here only).",
  lookJak: "Jak",
  lookDark: "Dark Jak",
  lookLight: "Light Jak",
  armor: "Armor",
  armorTag: "armor",
  parts: "{n} parts",
  shapes: "{n} blend shapes",
  shape: "Shape {n}",
  shapeDark: "Dark Jak {n}",
  shape_goggles: "Goggles",
  shape_scarf: "Scarf",
  resetLook: "Reset",
  applyLook: "Apply to the creation",
  applyLookConfirm:
    "The hidden parts will be removed from the creation and the blend shapes set in its vertices. Go on?",
  noWings:
    "{name}.glb is not in the model's folder nor in the game's models folder.",
  noGameTextures:
    "The eyes and Dark Jak's skin are drawn from the game's textures (decompiler_out/<game>/textures): set the jak-project folder in the settings.",
  "core_every-part-hidden": "A model needs at least one part shown.",
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
  jakProjectDir: "jak-project folder (optional)",
  jakProjectDirHint:
    "When set, exports go straight into its custom_assets folder.",
  game: "Game",
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
    "Choose the game to try in: the jak-project folder in the settings, or start the studio from the OpenGOAL Launcher.",
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

type Key = keyof typeof en;

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
  tag_cutscene: "Cinématique",
  tag_lowres: "Basse résolution",
  tag_simple: "Simplifié",
  tag_rider: "En véhicule",
  tag_escort: "Escorte",
  tag_minigame: "Mini-jeu",
  tag_main: "Principal",
  import: "Importer des modèles (.glb) dans les créations",
  importReplace:
    "Une création s'appelle déjà {name}. La remplacer ? Ses animations seront perdues.",
  noModelsDir:
    "Choisissez le dossier des modèles de {game} dans les paramètres.",
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
  twinAnimationsHint:
    "L'extraction a écrit les animations de cette entité dans d'autres fichiers : {files}",
  rename: "Renommer",
  delete: "Supprimer",
  deleteConfirm: "Supprimer l'animation {name} ?",
  gameCheck: "Vérification pour le jeu",
  allGood: "Prêt pour build-actor.",
  exportGame: "Exporter pour {game}",
  tryInGame: "Essayer dans le jeu",
  tryInGameHint:
    "Construit le modèle comme un acteur personnalisé et le montre dans {game}, dans son niveau de test (test-zone), en train de jouer l'animation choisie. Il faut le jeu installé : celui de l'OpenGOAL Launcher, ou le dossier jak-project des réglages. Les fichiers du jeu sont remis en place quand le jeu se ferme.",
  tryStep_export: "Écriture du modèle",
  tryStep_compiler:
    "Démarrage du compilateur du jeu (une à deux minutes la première fois)",
  tryStep_build: "Construction du niveau de test",
  tryStep_start: "Démarrage du jeu",
  tryStep_load: "Jak va au niveau de test",
  tryStep_play: "Affichage du modèle",
  inGame: "Dans le jeu : l'animation choisie ici y joue aussi.",
  closeGame: "Fermer le jeu",
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
    "Les variantes du modèle : pièces à montrer ou cacher, formes de mélange. Appliquer rend l'apparence propre à la création, celle que le jeu construit (les ailes de Light Jak sont un modèle à part, montré ici seulement).",
  lookJak: "Jak",
  lookDark: "Dark Jak",
  lookLight: "Light Jak",
  armor: "Armure",
  armorTag: "armure",
  parts: "{n} pièces",
  shapes: "{n} formes de mélange",
  shape: "Forme {n}",
  shapeDark: "Dark Jak {n}",
  shape_goggles: "Lunettes",
  shape_scarf: "Écharpe",
  resetLook: "Rétablir",
  applyLook: "Appliquer à la création",
  applyLookConfirm:
    "Les pièces cachées seront retirées de la création et les formes de mélange inscrites dans ses sommets. Continuer ?",
  noWings:
    "{name}.glb n'est ni dans le dossier du modèle ni dans le dossier des modèles du jeu.",
  noGameTextures:
    "Les yeux et la peau de Dark Jak sont dessinés avec les textures du jeu (decompiler_out/<jeu>/textures) : indiquez le dossier jak-project dans les réglages.",
  "core_every-part-hidden": "Un modèle garde au moins une pièce visible.",
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
  jakProjectDir: "Dossier jak-project (facultatif)",
  jakProjectDirHint:
    "S'il est choisi, les exports vont directement dans son dossier custom_assets.",
  game: "Jeu",
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
    "Choisissez le jeu où essayer : le dossier jak-project dans les réglages, ou lancez le studio depuis l'OpenGOAL Launcher.",
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
