#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

// OpenGOAL Animation Studio, with its command line:
//   opengoal-animation-studio [--lang <code>] [--default-library=<folder>] [--models=<folder>]
// --lang and --default-library are what the OpenGOAL Launcher gives the programs it starts (Jak
// Level Editor's way): the default library becomes the creations folder when none is chosen yet.
// --models is the models folder of the game chosen last. Options it does not know
// (--game-install=...) are skipped.

use std::path::{Path, PathBuf};
use std::time::Duration;

use animation_studio_core::{
  edit::{self, Channel},
  library::{self, Export, ModelEntry, ModelInfo, PlayRequest, Settings, Twin},
  looks::{self, Looks},
  rules::Text,
};
use serde_json::{Value, json};
use tauri::{Emitter, Manager};

struct Launch {
  lang: Option<String>,
}

/// An error for the interface: with the code and arguments of its text when the core gave them, so
/// that the interface says it in its language.
fn err(e: anyhow::Error) -> Value {
  let message = format!("{e:#}");
  match e.downcast_ref::<Text>() {
    Some(text) => json!({ "code": text.code, "args": text.args, "message": message }),
    None => json!({ "message": message }),
  }
}

#[tauri::command]
fn get_settings(launch: tauri::State<'_, Launch>) -> Settings {
  let mut settings = Settings::load();
  if launch.lang.is_some() {
    settings.lang.clone_from(&launch.lang);
  }
  settings
}

#[tauri::command]
fn save_settings(settings: Settings) -> Result<(), Value> {
  settings.save().map_err(err)
}

#[tauri::command]
async fn list_models(dir: PathBuf, game: String, objects: bool) -> Vec<ModelEntry> {
  library::list_models(&dir, &game, objects)
}

#[tauri::command]
async fn inspect_model(path: PathBuf) -> Result<ModelInfo, Value> {
  library::inspect(&path).map_err(err)
}

#[tauri::command]
async fn read_model(path: PathBuf) -> Result<tauri::ipc::Response, Value> {
  std::fs::read(&path)
    .map(tauri::ipc::Response::new)
    .map_err(|e| json!({ "message": format!("Unable to read {}: {e}", path.display()) }))
}

/// When the file last changed (ms), so the window follows the edits of the MCP server.
#[tauri::command]
async fn modified(path: PathBuf) -> Option<u64> {
  let time = std::fs::metadata(path).ok()?.modified().ok()?;
  Some(time.duration_since(std::time::UNIX_EPOCH).ok()?.as_millis() as u64)
}

/// The command that registers the MCP server (the program next to this one) with Claude Code.
#[tauri::command]
fn mcp_command() -> String {
  let name = if cfg!(windows) {
    "opengoal-animation-studio-mcp.exe"
  } else {
    "opengoal-animation-studio-mcp"
  };
  let path = std::env::current_exe()
    .ok()
    .and_then(|exe| exe.parent().map(|dir| dir.join(name)))
    .unwrap_or_else(|| name.into());
  format!(
    "claude mcp add opengoal-animation-studio -- \"{}\"",
    path.display()
  )
}

fn edit_creation(
  model: PathBuf,
  change: impl FnOnce(&mut animation_studio_core::glb::Glb) -> anyhow::Result<()>,
) -> Result<PathBuf, Value> {
  let settings = Settings::load();
  library::edit_creation(&model, &settings.creations(), settings.game(), change).map_err(err)
}

#[tauri::command]
async fn set_animation(
  model: PathBuf,
  name: String,
  channels: Vec<Channel>,
) -> Result<PathBuf, Value> {
  edit_creation(model, |glb| edit::set_animation(glb, &name, &channels))
}

#[tauri::command]
async fn delete_animation(model: PathBuf, name: String) -> Result<PathBuf, Value> {
  edit_creation(model, |glb| edit::delete_animation(glb, &name))
}

#[tauri::command]
async fn rename_animation(
  model: PathBuf,
  name: String,
  new_name: String,
) -> Result<PathBuf, Value> {
  edit_creation(model, |glb| edit::rename_animation(glb, &name, &new_name))
}

#[tauri::command]
async fn model_looks(path: PathBuf) -> Result<Looks, Value> {
  Ok(looks::looks(
    &animation_studio_core::glb::Glb::read(&path).map_err(err)?,
  ))
}

/// The look made the creation's own: `hidden` (mesh, primitive) removed, blend shapes baked.
#[tauri::command]
async fn apply_looks(
  model: PathBuf,
  hidden: Vec<(usize, usize)>,
  weights: Vec<f32>,
) -> Result<PathBuf, Value> {
  edit_creation(model, |glb| looks::apply(glb, &hidden, &weights))
}

/// The models with the same skeleton as `path` and animations it lacks, in its folder and the game's
/// models folder.
#[tauri::command]
async fn twin_animations(path: PathBuf, game: String) -> Vec<Twin> {
  let mut dirs: Vec<PathBuf> = path.parent().map(PathBuf::from).into_iter().collect();
  dirs.extend(Settings::load().models_dir(&game).map(PathBuf::from));
  library::twins(&path, &dirs)
}

/// The animations of `sources` (twins of `model`) added to its creation.
#[tauri::command]
async fn add_animations(model: PathBuf, sources: Vec<PathBuf>) -> Result<PathBuf, Value> {
  edit_creation(model, |glb| {
    for source in &sources {
      edit::copy_animations(glb, &animation_studio_core::glb::Glb::read(source)?)?;
    }
    Ok(())
  })
}

/// The game's textures near a model's: the .png files of the decompiler's page
/// (decompiler_out/<game>/textures/<page>/) holding `beside`, one of the model's textures (its
/// level loads that page with it), then those of common. The pages every level loads (common,
/// level-default-pris) are looked at first; the textures folder above the model first, then the
/// jak-project's.
#[tauri::command]
async fn game_textures(game: String, near: PathBuf, beside: String) -> Vec<PathBuf> {
  let settings = Settings::load();
  let mut roots: Vec<PathBuf> = near.ancestors().map(|d| d.join("textures")).collect();
  if let Some(project) = &settings.jak_project_dir {
    roots.push(project.join("decompiler_out").join(&game).join("textures"));
  }
  let pngs = |page: &Path| -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(page) else {
      return Vec::new();
    };
    entries
      .flatten()
      .map(|e| e.path())
      .filter(|p| p.extension().is_some_and(|x| x == "png"))
      .collect()
  };
  let file = format!("{beside}.png");
  for root in roots.iter().filter(|r| r.is_dir()) {
    let mut pages = vec![root.join("common"), root.join("level-default-pris")];
    let mut others: Vec<PathBuf> = std::fs::read_dir(root)
      .into_iter()
      .flatten()
      .flatten()
      .map(|e| e.path())
      .collect();
    others.sort();
    pages.extend(others);
    if let Some(page) = pages.iter().find(|p| p.join(&file).is_file()) {
      let mut out = pngs(page);
      if !page.ends_with("common") {
        out.extend(pngs(&root.join("common")));
      }
      return out;
    }
  }
  Vec::new()
}

/// A model that goes with `near` (Light Jak's wings with Jak): `<name>.glb` in its folder, else in
/// the game's models folder.
#[tauri::command]
async fn find_model(near: PathBuf, game: String, name: String) -> Option<PathBuf> {
  let file = format!("{name}.glb");
  let beside = near.parent()?.join(&file);
  if beside.is_file() {
    return Some(beside);
  }
  let dir = Settings::load().models_dir(&game)?.to_path_buf();
  let mut stack = vec![dir];
  while let Some(d) = stack.pop() {
    for e in std::fs::read_dir(&d).ok()?.flatten() {
      let path = e.path();
      if path.is_dir() {
        stack.push(path);
      } else if e.file_name().to_string_lossy() == file {
        return Some(path);
      }
    }
  }
  None
}

#[tauri::command]
async fn import_model(path: PathBuf, game: String, replace: bool) -> Result<PathBuf, Value> {
  library::import(&path, &Settings::load().creations(), &game, replace).map_err(err)
}

/// What the MCP server asked the window to play, taken so that it plays once.
#[tauri::command]
fn take_play() -> Option<PlayRequest> {
  PlayRequest::take(&Settings::dir())
}

#[tauri::command]
async fn export_for_game(model: PathBuf, lang: String) -> Result<Export, Value> {
  let settings = Settings::load();
  let creation =
    library::creation_for(&model, &settings.creations(), settings.game()).map_err(err)?;
  library::export(
    &creation,
    settings.game(),
    settings.jak_project_dir.as_deref(),
    &lang,
  )
  .map_err(err)
}

fn main() {
  let mut lang = None;
  let mut default_library = None;
  let mut models = None;
  let mut args = std::env::args().skip(1);
  while let Some(a) = args.next() {
    let (key, value) = match a.split_once('=') {
      Some((k, v)) => (k.to_owned(), Some(v.to_owned())),
      None => (a.clone(), None),
    };
    let mut value = || value.clone().or_else(|| args.next());
    match key.as_str() {
      "--lang" => lang = value(),
      "--default-library" => default_library = value().map(PathBuf::from),
      "--models" => models = value().map(PathBuf::from),
      _ => {}
    }
  }

  // the creations folder written down, so the MCP server works in the same one
  let mut settings = Settings::load();
  if settings.creations_dir.is_none() {
    settings.creations_dir = Some(default_library.unwrap_or_else(|| settings.creations()));
  }
  if let Some(dir) = models {
    let game = settings.game().to_owned();
    settings.models_dirs.insert(game, dir);
  }
  let _ = settings.save();

  tauri::Builder::default()
    .plugin(tauri_plugin_opener::init())
    .plugin(tauri_plugin_dialog::init())
    .manage(Launch { lang })
    .setup(|app| {
      // the MCP server's play_animation leaves a request: the window comes up and takes it (its own
      // timers are slowed down while it is minimized)
      let handle = app.handle().clone();
      std::thread::spawn(move || {
        let file = PlayRequest::file(&Settings::dir());
        let mut seen = None;
        loop {
          std::thread::sleep(Duration::from_millis(250));
          let stamp = std::fs::metadata(&file).and_then(|m| m.modified()).ok();
          if stamp.is_some() && stamp != seen {
            if let Some(window) = handle.get_webview_window("main") {
              let _ = window.unminimize();
              let _ = window.set_focus();
            }
            let _ = handle.emit("play", ());
          }
          seen = stamp;
        }
      });
      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      get_settings,
      save_settings,
      list_models,
      inspect_model,
      read_model,
      modified,
      mcp_command,
      set_animation,
      delete_animation,
      rename_animation,
      import_model,
      model_looks,
      twin_animations,
      add_animations,
      game_textures,
      apply_looks,
      find_model,
      take_play,
      export_for_game,
    ])
    .run(tauri::generate_context!())
    .expect("OpenGOAL Animation Studio could not start");
}
