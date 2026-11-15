#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

// Jak Animation Studio, with its command line:
//   jak-animation-studio [--lang <code>] [--default-library=<folder>] [--models=<folder>]
//                             [--game-install=<game>=<folder>]... [--tooling-version=<version>]
// --lang, --default-library and --game-install are what the OpenGOAL Launcher gives the programs it
// starts (Jak Level Editor's way): the default library becomes the creations folder when none is
// chosen yet; a game installed is where models are tried (else the jak-project of the settings).
// --tooling-version, the Launcher's active tooling, says the Launcher started the studio: models are
// then tried in its games only. --models is the models folder of the game chosen last. Options it
// does not know are skipped.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use animation_studio_core::{
  edit::{self, Channel},
  game_test::{self, Check, GameTest},
  library::{self, Export, GameNames, ModelEntry, ModelInfo, PlayRequest, Settings, Twin},
  looks::{self, Looks},
  rules::Text,
};
use serde_json::{Value, json};
use tauri::{Emitter, Manager};

struct Launch {
  lang: Option<String>,
  /// the games the Launcher installed, by game
  game_installs: BTreeMap<String, PathBuf>,
  /// the Launcher's active tooling, when the Launcher started the studio
  tooling_version: Option<String>,
}

/// A line added to the error log, `<prefs>/errors.log` (`%APPDATA%/OpenGOAL/animation-studio` on
/// Windows), after the time (UTC); past 1 MB, the log becomes errors.old.log and a new one starts.
fn log_error(line: &str) {
  use std::io::Write;
  let dir = Settings::dir();
  let file = dir.join("errors.log");
  if std::fs::metadata(&file).is_ok_and(|m| m.len() > 1 << 20) {
    let _ = std::fs::rename(&file, dir.join("errors.old.log"));
  }
  let _ = std::fs::create_dir_all(&dir);
  if let Ok(mut f) = std::fs::OpenOptions::new()
    .create(true)
    .append(true)
    .open(&file)
  {
    let _ = writeln!(f, "{} {line}", utc_now());
  }
}

/// The time now: "2026-11-15 14:03:27 UTC".
fn utc_now() -> String {
  let secs = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .map_or(0, |d| d.as_secs()) as i64;
  // days since 1970 to a date: Howard Hinnant's civil_from_days
  let z = secs.div_euclid(86400) + 719_468;
  let era = z.div_euclid(146_097);
  let doe = z - era * 146_097;
  let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
  let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
  let mp = (5 * doy + 2) / 153;
  let (d, m) = (
    doy - (153 * mp + 2) / 5 + 1,
    if mp < 10 { mp + 3 } else { mp - 9 },
  );
  let y = yoe + era * 400 + i64::from(m <= 2);
  let s = secs.rem_euclid(86400);
  format!(
    "{y}-{m:02}-{d:02} {:02}:{:02}:{:02} UTC",
    s / 3600,
    s / 60 % 60,
    s % 60
  )
}

/// The studio's folder: its settings, errors.log, and the try's game-test/<game>/goalc.log, gk.log.
#[tauri::command]
fn prefs_folder() -> PathBuf {
  Settings::dir()
}

/// What went wrong in the window (a script error), written to the error log.
#[tauri::command]
fn log_window_error(message: String) {
  log_error(&format!("[window] {message}"));
}

/// An error for the interface: with the code and arguments of its text when the core gave them, so
/// that the interface says it in its language. It goes to the error log too.
fn err(e: anyhow::Error) -> Value {
  let message = format!("{e:#}");
  log_error(&message);
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

/// The models of `dir`; reading a game's thousands of files takes a while: "list-progress" events
/// tell the window how far it went ({ dir, done, total }), a hundred times at most.
#[tauri::command]
async fn list_models(
  app: tauri::AppHandle,
  dir: PathBuf,
  game: String,
  objects: bool,
) -> Vec<ModelEntry> {
  library::list_models_with(&dir, &game, objects, |done, total| {
    if done == total || done % (total / 100).max(1) == 0 {
      let _ = app.emit(
        "list-progress",
        json!({ "dir": dir, "done": done, "total": total }),
      );
    }
  })
}

#[tauri::command]
async fn inspect_model(path: PathBuf) -> Result<ModelInfo, Value> {
  library::inspect(&path).map_err(err)
}

#[tauri::command]
async fn read_model(path: PathBuf) -> Result<tauri::ipc::Response, Value> {
  let unreadable = |e: &dyn std::fmt::Display| json!({ "message": format!("Unable to read {}: {e}", path.display()) });
  // a skin plays the animations of its twin (Jak 3's jakb-c, jakb's)
  if path
    .extension()
    .is_some_and(|x| x.eq_ignore_ascii_case("glb"))
  {
    return library::read_with_skin_animations(&path)
      .map(|glb| tauri::ipc::Response::new(glb.to_bytes()))
      .map_err(|e| unreadable(&e));
  }
  std::fs::read(&path)
    .map(tauri::ipc::Response::new)
    .map_err(|e| unreadable(&e))
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
    "jak-animation-studio-mcp.exe"
  } else {
    "jak-animation-studio-mcp"
  };
  let path = std::env::current_exe()
    .ok()
    .and_then(|exe| exe.parent().map(|dir| dir.join(name)))
    .unwrap_or_else(|| name.into());
  format!(
    "claude mcp add jak-animation-studio -- \"{}\"",
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
async fn model_looks(path: PathBuf, game: String) -> Result<Looks, Value> {
  let glb = animation_studio_core::glb::Glb::read(&path).map_err(err)?;
  let project = Settings::load().decompiled(&game);
  Ok(looks::game_looks(&glb, &path, &game, project.as_deref()))
}

/// Each animation's eye frames (looks::eye_frames), by name.
#[tauri::command]
async fn model_eye_frames(
  path: PathBuf,
  game: String,
) -> std::collections::HashMap<String, Vec<[looks::EyeFrame; 2]>> {
  let project = Settings::load().decompiled(&game);
  looks::eye_frames(&path, &game, project.as_deref())
}

/// The .png files of the textures the game draws the model's eyes with (looks::eye_textures), left
/// then right: iris, pupil, lid. Each from the decompiler's textures, in the page of the model's
/// level when it has it (Kleever's klever-eye from desliz-pris), else in the first page that has it.
#[tauri::command]
async fn model_eyes(path: PathBuf, game: String) -> Option<[[PathBuf; 3]; 2]> {
  let project = Settings::load().decompiled(&game);
  let eyes = looks::eye_textures(&path, &game, project.as_deref())?;
  let mut roots: Vec<PathBuf> = path.ancestors().map(|d| d.join("textures")).collect();
  roots.extend(project.map(|p| p.join("decompiler_out").join(&game).join("textures")));
  let level = path
    .parent()
    .and_then(|d| d.file_name())
    .map(|d| d.to_string_lossy().into_owned())
    .unwrap_or_default();
  let file = |name: &String| -> Option<PathBuf> {
    roots.iter().filter(|r| r.is_dir()).find_map(|root| {
      let mut pages: Vec<PathBuf> = std::fs::read_dir(root)
        .ok()?
        .flatten()
        .map(|e| e.path())
        .collect();
      pages.sort_by_key(|p| {
        let page = p.file_name().map(|n| n.to_string_lossy().into_owned());
        (!page.is_some_and(|n| n.starts_with(&level)), p.clone())
      });
      pages
        .into_iter()
        .map(|page| page.join(format!("{name}.png")))
        .find(|f| f.is_file())
    })
  };
  let side = |names: &[String; 3]| -> Option<[PathBuf; 3]> {
    Some([file(&names[0])?, file(&names[1])?, file(&names[2])?])
  };
  Some([side(&eyes.left)?, side(&eyes.right)?])
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
  if let Some(project) = settings.decompiled(&game) {
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

/// The window's answer to the MCP server's request (a movement put on, a check in the game), which
/// waits for this file.
#[tauri::command]
fn answer_mcp(result: Value) -> Result<(), Value> {
  let dir = Settings::dir();
  let tmp = dir.join("mcp-answer.json.tmp");
  std::fs::write(&tmp, result.to_string())
    .and_then(|()| std::fs::rename(&tmp, library::answer_file(&dir)))
    .map_err(|e| json!({ "message": format!("Unable to answer the MCP server: {e}") }))
}

/// The names the game's code gives a model's art (the guide's lines).
#[tauri::command]
async fn game_names(model: PathBuf) -> Result<GameNames, Value> {
  library::model_game_names(&model).map_err(err)
}

#[tauri::command]
async fn export_for_game(model: PathBuf, lang: String) -> Result<Export, Value> {
  let settings = Settings::load();
  let creation =
    library::creation_for(&model, &settings.creations(), settings.game()).map_err(err)?;
  library::export(
    &creation,
    settings.game(),
    settings.game_data(settings.game()).as_deref(),
    &lang,
  )
  .map_err(err)
}

/// The installation of `game` a try starts: the Launcher's, else the settings' one.
fn try_root(launch: &Launch, game: &str) -> anyhow::Result<PathBuf> {
  match (launch.game_installs.get(game), &launch.tooling_version) {
    (Some(root), _) => Ok(root.clone()),
    // started by the Launcher: a game built by another tooling than the active one is not tried
    (None, Some(version)) => Err(anyhow::Error::new(Text::new(
      "game-not-in-launcher",
      "This game is not installed with the OpenGOAL Launcher's active tooling ({version}): install or update it in the Launcher.",
      &[("version", version.clone())],
    ))),
    (None, None) => Settings::load().game_dirs.remove(game).ok_or_else(|| {
      anyhow::Error::new(Text::new(
        "no-game-folder",
        "Choose the game's installation folder in the settings, or start the studio from the OpenGOAL Launcher.",
        &[],
      ))
    }),
  }
}

/// Why `game` cannot be started for a try now, or None: no installation found or complete, Jak and
/// Daxter (it closes when a level loads a custom actor with a skeleton), or another game on its
/// port (the game of a try the studio runs is its own).
#[tauri::command]
async fn try_blocked(app: tauri::AppHandle, game: String) -> Option<Value> {
  if game == "jak1" {
    return Some(err_quiet(Text::new(
      "try-jak1",
      "Jak and Daxter closes when a level loads a character added this way: try models in Jak II or Jak 3.",
      &[],
    )));
  }
  let root = match try_root(&app.state::<Launch>(), &game) {
    Ok(root) => root,
    Err(e) => return Some(err_quiet(e.downcast::<Text>().ok()?)),
  };
  if let Err(e) = game_test::find_install(&game, &root) {
    return Some(err_quiet(e.downcast::<Text>().ok()?));
  }
  let ours = app.state::<GameTest>().running();
  let port = game_test::game_port(&game);
  (!ours && game_test::port_taken(port)).then(|| {
    err_quiet(Text::new(
      "game-port-taken",
      "Another game listens on port {port}: close it first.",
      &[("port", port.to_string())],
    ))
  })
}

/// A text of the core for the interface, not an error: not written to the error log.
fn err_quiet(text: Text) -> Value {
  json!({ "code": text.code, "args": text.args, "message": text.message })
}

/// The model tried in the game (test_in_game): built as a custom actor and shown in `game`'s test
/// level, playing its animation `animation` again and again; the steps go to the window
/// ("game-test" events).
#[tauri::command]
async fn test_in_game(
  app: tauri::AppHandle,
  model: PathBuf,
  game: String,
  animation: String,
) -> Result<(), Value> {
  let root = try_root(&app.state::<Launch>(), &game).map_err(err)?;
  tauri::async_runtime::spawn_blocking(move || {
    app
      .state::<GameTest>()
      .try_model(&model, &root, &game, &animation, |step| {
        let _ = app.emit("game-test", step);
      })
  })
  .await
  .map_err(|e| json!({ "message": e.to_string() }))?
  .map_err(err)
}

/// The model in the game of the last try plays its animation `animation`: false when the try did
/// not build it (the window tries again with it).
#[tauri::command]
async fn play_in_game(app: tauri::AppHandle, animation: String) -> Result<bool, Value> {
  tauri::async_runtime::spawn_blocking(move || app.state::<GameTest>().play(&animation))
    .await
    .map_err(|e| json!({ "message": e.to_string() }))?
    .map_err(err)
}

/// What the game of the last try shows of `animation`: the element it plays, its frames, and
/// screenshots over `seconds`.
#[tauri::command]
async fn check_in_game(
  app: tauri::AppHandle,
  game: String,
  animation: String,
  seconds: f32,
) -> Result<Check, Value> {
  tauri::async_runtime::spawn_blocking(move || {
    app.state::<GameTest>().check(&game, &animation, seconds, 4)
  })
  .await
  .map_err(|e| json!({ "message": e.to_string() }))?
  .map_err(err)
}

/// The game of the tries closed, the game's files put back.
#[tauri::command]
async fn stop_game_test(app: tauri::AppHandle) {
  let _ = tauri::async_runtime::spawn_blocking(move || app.state::<GameTest>().stop()).await;
}

fn main() {
  let mut lang = None;
  let mut default_library = None;
  let mut models = None;
  let mut game_installs = BTreeMap::new();
  let mut tooling_version = None;
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
      "--game-install" => {
        if let Some((game, folder)) = value().as_deref().and_then(|v| v.split_once('=')) {
          game_installs.insert(game.to_owned(), PathBuf::from(folder));
        }
      }
      "--tooling-version" => tooling_version = value(),
      _ => {}
    }
  }

  let default_hook = std::panic::take_hook();
  std::panic::set_hook(Box::new(move |info| {
    log_error(&format!("[panic] {info}"));
    default_hook(info);
  }));

  // the creations folder written down, so the MCP server works in the same one
  let mut settings = Settings::load();
  // started by the Launcher: its games are the folders not chosen yet
  settings.fill_from_launcher(&game_installs);
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
    .manage(Launch {
      lang,
      game_installs,
      tooling_version,
    })
    .manage(GameTest::default())
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
      model_eyes,
      model_eye_frames,
      twin_animations,
      add_animations,
      game_textures,
      find_model,
      take_play,
      answer_mcp,
      check_in_game,
      export_for_game,
      game_names,
      test_in_game,
      play_in_game,
      stop_game_test,
      try_blocked,
      log_window_error,
      prefs_folder,
    ])
    .build(tauri::generate_context!())
    .expect("Jak Animation Studio could not start")
    .run(|app, event| {
      // the game of the tries closes with the studio, and the game's files go back
      if let tauri::RunEvent::Exit = event {
        app.state::<GameTest>().stop();
      }
    });
}
