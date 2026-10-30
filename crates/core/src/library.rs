//! The folders the studio works with: a models folder per game, which it reads (only entities,
//! models with an armature, are listed), the creations folder it writes (a copy of each edited or
//! imported model, never the original, by game), the exports for a jak-project, and the requests
//! of the MCP server to the studio's window.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail, ensure};
use serde::{Deserialize, Serialize};

use crate::{
  edit,
  glb::Glb,
  model::{self, AnimationInfo, Joint},
  names::{self, Title},
  rules::{self, Issue, Text, goal_name},
};

pub const GAMES: [&str; 3] = ["jak1", "jak2", "jak3"];

/// The settings shared by the app and the MCP server.
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
#[serde(default, rename_all = "camelCase")]
pub struct Settings {
  /// the entities of each game ("jak1", "jak2", "jak3")
  pub models_dirs: BTreeMap<String, PathBuf>,
  pub creations_dir: Option<PathBuf>,
  /// a jak-project checkout: exports go into its custom_assets/ when set
  pub jak_project_dir: Option<PathBuf>,
  pub game: Option<String>,
  pub lang: Option<String>,
}

impl Settings {
  /// $OGAS_PREFS_DIR, else <data dir>/OpenGOAL/animation-studio, next to the other OpenGOAL tools.
  pub fn dir() -> PathBuf {
    std::env::var_os("OGAS_PREFS_DIR")
      .filter(|d| !d.is_empty())
      .map(PathBuf::from)
      .unwrap_or_else(|| {
        dirs::data_dir()
          .unwrap_or_default()
          .join("OpenGOAL")
          .join("animation-studio")
      })
  }

  pub fn load() -> Settings {
    std::fs::read(Self::dir().join("settings.json"))
      .ok()
      // Windows PowerShell and some editors start UTF-8 files with a BOM, which serde refuses
      .and_then(|b| serde_json::from_slice(b.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(&b)).ok())
      .unwrap_or_default()
  }

  pub fn save(&self) -> Result<()> {
    let dir = Self::dir();
    std::fs::create_dir_all(&dir)?;
    std::fs::write(dir.join("settings.json"), serde_json::to_vec_pretty(self)?)
      .context("Unable to save the settings")
  }

  pub fn game(&self) -> &str {
    self
      .game
      .as_deref()
      .filter(|g| GAMES.contains(g))
      .unwrap_or("jak1")
  }

  pub fn models_dir(&self, game: &str) -> Option<&Path> {
    self.models_dirs.get(game).map(PathBuf::as_path)
  }

  pub fn creations(&self) -> PathBuf {
    self.creations_dir.clone().unwrap_or_else(|| {
      dirs::document_dir()
        .unwrap_or_default()
        .join("Jak Animation Studio")
    })
  }
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ModelEntry {
  pub name: String,
  /// the official name of the entity, when the game gives it one
  pub title: Option<Title>,
  pub path: PathBuf,
  pub joints: usize,
  pub animations: usize,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ModelInfo {
  pub name: String,
  pub path: PathBuf,
  pub joints: Vec<Joint>,
  pub animations: Vec<AnimationInfo>,
  pub issues: Vec<Issue>,
}

pub fn model_name(path: &Path) -> String {
  path
    .file_stem()
    .map(|s| s.to_string_lossy().into_owned())
    .unwrap_or_default()
}

/// What tells a model's copies apart: its name, its joints, its animations.
type CopyKey = (String, Vec<String>, Vec<String>);

/// The entities (models with an armature) of a folder and its subfolders, with their official names
/// in `game`; the game's objects (props, platforms, vehicles...) only when `objects` is set. Exports
/// are skipped: they are copies of the creations next to them. The extraction writes a model out of
/// each level that loads it (Babak out of 11 levels of Jak and Daxter), each copy with that level's
/// unused vertices: one copy is listed per name, joints and animations, the smallest file.
pub fn list_models(dir: &Path, game: &str, objects: bool) -> Vec<ModelEntry> {
  let mut copies: HashMap<CopyKey, (u64, ModelEntry)> = HashMap::new();
  let mut stack = vec![dir.to_path_buf()];
  while let Some(d) = stack.pop() {
    let Ok(entries) = std::fs::read_dir(&d) else {
      continue;
    };
    for e in entries.flatten() {
      let path = e.path();
      if path.is_dir() {
        if e.file_name() != "export" {
          stack.push(path);
        }
      } else if path
        .extension()
        .is_some_and(|x| x.eq_ignore_ascii_case("glb"))
        && (objects || !names::is_object(game, &model_name(&path)))
        && let Ok(glb) = Glb::read_json(&path)
        && let Ok(joints) = model::skeleton(&glb)
      {
        let name = model_name(&path);
        let anims: Vec<String> = glb
          .array("animations")
          .iter()
          .enumerate()
          .map(|(i, a)| model::animation_name(a, i))
          .collect();
        let size = e.metadata().map_or(u64::MAX, |m| m.len());
        let entry = ModelEntry {
          title: names::title(game, &name),
          name: name.clone(),
          joints: joints.len(),
          animations: anims.len(),
          path,
        };
        let key = (name, joints.into_iter().map(|j| j.name).collect(), anims);
        match copies.get(&key) {
          Some((s, kept)) if (*s, &kept.path) <= (size, &entry.path) => {}
          _ => {
            copies.insert(key, (size, entry));
          }
        }
      }
    }
  }
  let mut out: Vec<ModelEntry> = copies.into_values().map(|(_, m)| m).collect();
  out.sort_by(|a, b| (a.name.to_lowercase(), &a.path).cmp(&(b.name.to_lowercase(), &b.path)));
  out
}

pub fn inspect(path: &Path) -> Result<ModelInfo> {
  let glb = Glb::read(path)?;
  let name = model_name(path);
  Ok(ModelInfo {
    joints: model::skeleton(&glb)?,
    animations: model::animations(&glb),
    issues: rules::check(&glb, &name),
    name,
    path: path.to_path_buf(),
  })
}

/// Where the creation of `model` goes: `<creations>/<game>/<name>/<name>.glb`.
fn creation_path(model: &Path, creations: &Path, game: &str) -> Result<PathBuf> {
  ensure!(
    GAMES.contains(&game),
    "'{game}' is not a game ({})",
    GAMES.join(", ")
  );
  let name = goal_name(&model_name(model));
  ensure!(!name.is_empty(), "{} has no usable name", model.display());
  Ok(creations.join(game).join(&name).join(format!("{name}.glb")))
}

fn copy_model(model: &Path, dest: &Path) -> Result<()> {
  std::fs::create_dir_all(dest.parent().expect("a file in a folder"))?;
  std::fs::copy(model, dest)
    .with_context(|| format!("Unable to copy {} into the creations", model.display()))?;
  Ok(())
}

/// The creation of a model, copied from the model the first time it is edited. A model already in
/// the creations folder is its own creation.
pub fn creation_for(model: &Path, creations: &Path, game: &str) -> Result<PathBuf> {
  if model.starts_with(creations) {
    return Ok(model.to_path_buf());
  }
  let dest = creation_path(model, creations, game)?;
  if !dest.exists() {
    copy_model(model, &dest)?;
  }
  Ok(dest)
}

/// Copies an entity from anywhere into the creations of `game`; `replace` overwrites the creation
/// of the same name, its animations with it.
pub fn import(model: &Path, creations: &Path, game: &str, replace: bool) -> Result<PathBuf> {
  let file = || ("file", model.display().to_string());
  model::skeleton(&Glb::read(model)?).with_context(|| {
    Text::new(
      "not-an-entity",
      "{file} is not an entity: an entity has one armature (skin).",
      &[file()],
    )
  })?;
  let dest = creation_path(model, creations, game)?;
  ensure!(
    replace || !dest.exists(),
    Text::new(
      "creation-exists",
      "A creation is already named '{name}'.",
      &[("name", model_name(&dest))]
    )
  );
  copy_model(model, &dest)?;
  Ok(dest)
}

/// Applies `change` to the creation of `model` and returns the creation's path.
pub fn edit_creation(
  model: &Path,
  creations: &Path,
  game: &str,
  change: impl FnOnce(&mut Glb) -> Result<()>,
) -> Result<PathBuf> {
  let path = creation_for(model, creations, game)?;
  let mut glb = Glb::read(&path)?;
  change(&mut glb)?;
  glb.write(&path)?;
  Ok(path)
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Export {
  pub glb: PathBuf,
  pub readme: PathBuf,
  pub notes: Vec<Text>,
  pub issues: Vec<Issue>,
}

/// Writes the creation as build-actor wants it, at the place a jak-project reads it
/// (custom_assets/<game>/models/custom_levels/<actor>.glb): into `jak_project` when given, else into
/// `<creation folder>/export/`. A README next to the creation gives the lines to add to the game,
/// in `lang` ("en" or "fr").
pub fn export(
  creation: &Path,
  game: &str,
  jak_project: Option<&Path>,
  lang: &str,
) -> Result<Export> {
  if !GAMES.contains(&game) {
    bail!(
      "'{game}' is not a game the studio exports for ({})",
      GAMES.join(", ")
    );
  }
  let actor = goal_name(&model_name(creation));
  let mut glb = Glb::read(creation)?;
  let notes = edit::normalize(&mut glb, &actor)?;
  let issues = rules::check(&glb, &actor);
  edit::for_build_actor(&mut glb)?;
  let folder = creation
    .parent()
    .context("a creation is in a folder")?
    .join("export");
  let root = jak_project.map_or_else(|| folder.clone(), Path::to_path_buf);
  let dest = root
    .join("custom_assets")
    .join(game)
    .join("models")
    .join("custom_levels")
    .join(format!("{actor}.glb"));
  std::fs::create_dir_all(dest.parent().expect("a file in a folder"))?;
  glb.write(&dest)?;

  let anims: Vec<String> = model::animations(&glb)
    .into_iter()
    .map(|a| a.name)
    .collect();
  let readme = folder.join("README.md");
  std::fs::create_dir_all(&folder)?;
  std::fs::write(
    &readme,
    readme_text(&actor, game, &anims, jak_project.is_some(), lang),
  )?;
  Ok(Export {
    glb: dest,
    readme,
    notes,
    issues,
  })
}

/// The README of an export, in English or, for `lang` "fr", in French.
fn readme_text(actor: &str, game: &str, anims: &[String], in_project: bool, lang: &str) -> String {
  let short = |a: &str| goal_name(rules::short_name(a, actor));
  let art: Vec<String> = anims.iter().map(|a| format!("{}-ja", short(a))).collect();
  let art = art.join(" ");
  let rows: String = anims
    .iter()
    .enumerate()
    .map(|(i, a)| format!("| {} | `{a}` | `{actor}-{}-ja` |\n", i + 2, short(a)))
    .collect();
  let first = anims.first().map_or("idle".to_owned(), |a| short(a));
  if lang.starts_with("fr") {
    let copy = if in_project {
      "Le modèle est déjà dans votre dossier jak-project.".to_owned()
    } else {
      format!(
        "Copiez le dossier `custom_assets` qui est à côté de ce fichier dans votre dossier jak-project : le modèle va dans `custom_assets/{game}/models/custom_levels/{actor}.glb`."
      )
    };
    return format!(
      "# {actor} pour {game}\n\n\
       Fait avec Jak Animation Studio. Le modèle et ses animations suivent ce que demande \
       build-actor d'OpenGOAL.\n\n\
       1. {copy}\n\
       2. Construisez son art group : ajoutez cette ligne à `goal_src/{game}/game.gp`.\n\n   \
       ```lisp\n   (build-actor \"{actor}\")\n   ```\n\n\
       3. Chargez-le avec le niveau : ajoutez `\"{actor}-ag.go\"` au fichier `.gd` du niveau, avant \
       le `.go` du niveau lui-même.\n\
       4. Déclarez l'art group et ses animations dans le code de l'entité (`:bounds` est une sphère : \
       x y z rayon, en mètres).\n\n   \
       ```lisp\n   (def-actor {actor}\n     :bounds (0 0 0 4)\n     :art ({art}))\n   ```\n\n\
       | Emplacement | Animation | Symbole |\n|---|---|---|\n{rows}\n\
       Jouez-en une dans un état avec `(ja-no-eval :group! {actor}-{first}-ja :num! (seek!) :frame-num 0.0)`.\n"
    );
  }
  let copy = if in_project {
    "The model is already in your jak-project folder.".to_owned()
  } else {
    format!(
      "Copy the `custom_assets` folder next to this file into your jak-project folder: the model goes to `custom_assets/{game}/models/custom_levels/{actor}.glb`."
    )
  };
  format!(
    "# {actor} for {game}\n\n\
     Made with Jak Animation Studio. The model and its animations follow what OpenGOAL's \
     build-actor needs.\n\n\
     1. {copy}\n\
     2. Build its art group: add this line to `goal_src/{game}/game.gp`.\n\n   \
     ```lisp\n   (build-actor \"{actor}\")\n   ```\n\n\
     3. Load it with the level: add `\"{actor}-ag.go\"` to the level's `.gd` file, before the \
     level's own `.go`.\n\
     4. Declare the art group and its animations in the entity's code (`:bounds` is a sphere: x y z \
     radius, in metres).\n\n   \
     ```lisp\n   (def-actor {actor}\n     :bounds (0 0 0 4)\n     :art ({art}))\n   ```\n\n\
     | Slot | Animation | Symbol |\n|---|---|---|\n{rows}\n\
     Play one in a state with `(ja-no-eval :group! {actor}-{first}-ja :num! (seek!) :frame-num 0.0)`.\n"
  )
}

/// Another model with the same skeleton (the same joints, in the same order) and animations a model
/// lacks: the extraction writes an art group's mesh and its animations to different files (Jak 3's
/// jakb-c-lod0 and jakb-lod0), and a cutscene model's animations to its copy in each level.
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Twin {
  pub path: PathBuf,
  pub animations: Vec<String>,
}

/// A model's joint names and animation names, read from its JSON only, kept while the file is
/// unchanged: looking for twins reads the header of every model of a folder.
fn outline(path: &Path) -> Option<(Vec<String>, Vec<String>)> {
  type Outlines = HashMap<PathBuf, (SystemTime, Option<(Vec<String>, Vec<String>)>)>;
  static CACHE: Mutex<Option<Outlines>> = Mutex::new(None);
  let modified = std::fs::metadata(path).and_then(|m| m.modified()).ok()?;
  let mut cache = CACHE.lock().ok()?;
  let cache = cache.get_or_insert_with(HashMap::new);
  if let Some((at, outline)) = cache.get(path)
    && *at == modified
  {
    return outline.clone();
  }
  let outline = Glb::read_json(path).ok().and_then(|glb| {
    let joints = model::skeleton(&glb).ok()?;
    Some((
      joints.into_iter().map(|j| j.name).collect(),
      glb
        .array("animations")
        .iter()
        .enumerate()
        .map(|(i, a)| model::animation_name(a, i))
        .collect(),
    ))
  });
  cache.insert(path.to_path_buf(), (modified, outline.clone()));
  outline
}

/// The twins of `model` in `dirs` and their subfolders, each with the animations it adds (a name
/// found in an earlier twin is not counted again). A twin is of the same entity: its art group is
/// the model's, or one is the other's variant ("jakb" and "jakb-c"); different entities can have
/// the same joints (keira-simple and torn-simple).
pub fn twins(model: &Path, dirs: &[PathBuf]) -> Vec<Twin> {
  let Some((joints, animations)) = outline(model) else {
    return Vec::new();
  };
  let name = model_name(model);
  let group = names::art_group(&name);
  let kin = |other: &str| {
    other == group
      || other.starts_with(&format!("{group}-"))
      || group.starts_with(&format!("{other}-"))
  };
  let mut have: HashSet<String> = animations.into_iter().collect();
  let mut seen = HashSet::new();
  let mut out = Vec::new();
  let mut stack = dirs.to_vec();
  while let Some(d) = stack.pop() {
    let Ok(entries) = std::fs::read_dir(&d) else {
      continue;
    };
    let mut entries: Vec<_> = entries.flatten().map(|e| e.path()).collect();
    entries.sort();
    for path in entries {
      if path.is_dir() {
        if path.file_name().is_some_and(|n| n != "export") {
          stack.push(path);
        }
        continue;
      }
      let glb = path
        .extension()
        .is_some_and(|x| x.eq_ignore_ascii_case("glb"));
      if !glb || path == model || !seen.insert(path.clone()) {
        continue;
      }
      let Some((theirs, their_animations)) = outline(&path) else {
        continue;
      };
      if theirs != joints || !kin(names::art_group(&model_name(&path))) {
        continue;
      }
      let new: Vec<String> = their_animations
        .into_iter()
        .filter(|a| have.insert(a.clone()))
        .collect();
      if !new.is_empty() {
        out.push(Twin {
          path,
          animations: new,
        });
      }
    }
  }
  out
}

/// What the MCP server asks the studio's window to show: a model playing one of its animations.
/// It goes through `play.json` in the settings' folder, which the window takes.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PlayRequest {
  pub model: PathBuf,
  pub animation: Option<String>,
  pub game: String,
  /// when it was asked, in ms since 1970: a window opened later than a minute after skips it
  pub at: u64,
}

impl PlayRequest {
  const FRESH: Duration = Duration::from_secs(60);

  pub fn file(dir: &Path) -> PathBuf {
    dir.join("play.json")
  }

  pub fn now() -> u64 {
    SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .map_or(0, |d| d.as_millis() as u64)
  }

  pub fn send(&self, dir: &Path) -> Result<()> {
    std::fs::create_dir_all(dir)?;
    let tmp = dir.join("play.json.tmp");
    std::fs::write(&tmp, serde_json::to_vec(self)?)?;
    std::fs::rename(&tmp, Self::file(dir)).context("Unable to ask the studio")
  }

  /// The request waiting, taken so that it plays once.
  pub fn take(dir: &Path) -> Option<PlayRequest> {
    let file = Self::file(dir);
    let bytes = std::fs::read(&file).ok()?;
    let _ = std::fs::remove_file(&file);
    serde_json::from_slice::<PlayRequest>(&bytes)
      .ok()
      .filter(|r| Self::now().saturating_sub(r.at) < Self::FRESH.as_millis() as u64)
  }
}
