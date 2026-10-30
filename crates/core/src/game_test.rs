//! Try in the game (Jak Level Editor's way): the model shown in the game the user installed, without
//! installing or compiling the game. Only the game's own programs run. Its goalc builds, in a
//! project of the studio's (<prefs>/game-test/<game>/goalc/), the test level every OpenGOAL game has
//! (test-zone, TSZ.DGO) with the model as a custom actor and a viewer of a few lines; gk plays it;
//! goalc, connected to it, brings Jak to test-zone and puts the model in front of him, playing the
//! animation picked. The only files written into the game are test-zone's DGO and .fr3: the game's
//! are kept in the try's folder and put back when the try stops.
//!
//! An installation is a folder laid out one of three ways:
//!   OpenGOAL Launcher   <folder>/active/bin/gk(.exe), goalc; <folder>/active/<game>/data/
//!   an OpenGOAL release <folder>/gk(.exe), goalc; <folder>/data/
//!   a jak-project       <folder>/out/build/<preset>/bin/gk(.exe), goalc; <folder>/ itself

use std::io::{BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail};
use serde_json::{Value, json};

use crate::{edit, glb::Glb, library::Settings, rules};

const EXE: &str = std::env::consts::EXE_SUFFIX;
/// the model's name in the game: never one of the game's art groups
const ACTOR: &str = "ogas-model";
const LEVEL: &str = "test-zone";

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
  m.lock().unwrap_or_else(|e| e.into_inner())
}

fn text(code: &'static str, message: &str, args: &[(&'static str, String)]) -> rules::Text {
  rules::Text::new(code, message, args)
}

/// An installed game: its programs and its data folder (goal_src/, iso_data/, out/<game>/).
#[derive(Clone, Debug, PartialEq)]
pub struct Install {
  pub game: String,
  pub gk: PathBuf,
  pub goalc: PathBuf,
  pub data: PathBuf,
}

impl Install {
  fn out(&self) -> PathBuf {
    self.data.join("out").join(&self.game)
  }
}

fn exe(dir: &Path, name: &str) -> PathBuf {
  dir.join(format!("{name}{EXE}"))
}

/// The installation of `game` in `root`, or what it lacks.
pub fn find_install(game: &str, root: &Path) -> Result<Install> {
  let mut layouts = vec![
    (
      root.join("active/bin"),
      root.join("active").join(game).join("data"),
    ),
    (root.to_path_buf(), root.join("data")),
  ];
  for preset in std::fs::read_dir(root.join("out/build"))
    .into_iter()
    .flatten()
    .flatten()
  {
    layouts.push((preset.path().join("bin"), root.to_path_buf()));
  }
  let (bin, data) = layouts
    .into_iter()
    .find(|(bin, _)| exe(bin, "gk").is_file())
    .with_context(|| {
      text(
        "game-not-found",
        "{folder} holds no game to try in: choose an OpenGOAL Launcher installation, an OpenGOAL release or a jak-project with the game built.",
        &[("folder", root.display().to_string())],
      )
    })?;
  let level = data
    .join("custom_assets")
    .join(game)
    .join("levels")
    .join(LEVEL);
  for needed in [
    exe(&bin, "goalc"),
    data.join("goal_src").join(game).join("game.gp"),
    level.join("testzone.gd"),
    data.join("out").join(game).join("iso/GAME.CGO"),
  ] {
    if !needed.exists() {
      bail!(text(
        "game-incomplete",
        "The game in {folder} lacks {file}: install or build it again.",
        &[
          ("folder", root.display().to_string()),
          ("file", needed.display().to_string())
        ],
      ));
    }
  }
  Ok(Install {
    game: game.into(),
    gk: exe(&bin, "gk"),
    goalc: exe(&bin, "goalc"),
    data,
  })
}

/// A program the try started, its output (stdout and stderr) gathered by a thread.
struct Program {
  child: Child,
  /// kept open: goalc quits at the end of its input
  _stdin: Option<ChildStdin>,
  out: Arc<Mutex<String>>,
}

impl Program {
  fn start(mut cmd: Command) -> Result<Self> {
    #[cfg(windows)]
    {
      use std::os::windows::process::CommandExt;
      cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW: no console (gk opens its own window)
    }
    let (reader, writer) = std::io::pipe()?;
    cmd
      .stdin(Stdio::piped())
      .stdout(writer.try_clone()?)
      .stderr(writer);
    let mut child = cmd
      .spawn()
      .with_context(|| format!("{:?} could not start", cmd.get_program()))?;
    drop(cmd);
    let out = Arc::new(Mutex::new(String::new()));
    let o = out.clone();
    std::thread::spawn(move || {
      let mut r = BufReader::new(reader);
      let mut buf = [0u8; 8192];
      while let Ok(n) = r.read(&mut buf) {
        if n == 0 {
          break;
        }
        let mut o = lock(&o);
        o.push_str(&strip_ansi(&String::from_utf8_lossy(&buf[..n])));
        // the last megabytes are enough
        if o.len() > 4 << 20 {
          let cut = o.len() - (2 << 20);
          let cut = (cut..o.len()).find(|&i| o.is_char_boundary(i)).unwrap_or(0);
          o.drain(..cut);
        }
      }
    });
    Ok(Self {
      _stdin: child.stdin.take(),
      child,
      out,
    })
  }

  fn alive(&mut self) -> bool {
    matches!(self.child.try_wait(), Ok(None))
  }

  fn len(&self) -> usize {
    lock(&self.out).len()
  }

  fn since(&self, mark: usize) -> String {
    let o = lock(&self.out);
    o.get(mark.min(o.len())..).unwrap_or_default().to_string()
  }

  /// Waits until its output after `mark` holds one of `texts`: which one, else None (timeout, or it
  /// exited).
  fn wait_for(&mut self, mark: usize, texts: &[&str], timeout: Duration) -> Option<usize> {
    let end = Instant::now() + timeout;
    loop {
      let since = self.since(mark);
      if let Some(i) = texts.iter().position(|t| since.contains(t)) {
        return Some(i);
      }
      if Instant::now() > end || !self.alive() {
        return None;
      }
      std::thread::sleep(Duration::from_millis(50));
    }
  }

  fn kill(&mut self) {
    let _ = self.child.kill();
    let _ = self.child.wait();
  }
}

// the colors of a terminal taken out (goalc writes them whatever its options)
fn strip_ansi(s: &str) -> String {
  let mut out = String::with_capacity(s.len());
  let mut chars = s.chars().peekable();
  while let Some(c) = chars.next() {
    if c != '\x1b' {
      out.push(c);
      continue;
    }
    if chars.peek() == Some(&'[') {
      chars.next();
      for c in chars.by_ref() {
        if c.is_ascii_alphabetic() {
          break;
        }
      }
    }
  }
  out
}

fn last_lines(text: &str, n: usize) -> String {
  let lines: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
  lines[lines.len().saturating_sub(n)..].join("\n")
}

fn failed(step: &str, output: &str) -> rules::Text {
  text(
    "game-step-failed",
    "{step} failed: {output}",
    &[("step", step.into()), ("output", last_lines(output, 8))],
  )
}

/// goalc, kept between tries: it learned the game's types once.
struct Goalc {
  program: Program,
  /// its nREPL port (the forms it compiles come there)
  port: u16,
  data: PathBuf,
  /// the forms sent: each one's mark of the end
  evals: u64,
}

impl Goalc {
  /// Compiles `form` (run by the game when connected): what goalc printed for it. Its prompt comes
  /// before its answers: a mark printed by goalc itself after the form (goos' `fmt`, flushed, unlike
  /// its `print`) says it is done.
  fn eval(&mut self, form: &str, timeout: Duration) -> Result<String> {
    self.evals += 1;
    let done = format!("ogas-done-{}", self.evals);
    let mark = self.program.len();
    let mut s = TcpStream::connect(("127.0.0.1", self.port))?;
    for body in [form.to_string(), format!("(seval (fmt #t \"{done}\n\"))")] {
      let mut msg = (body.len() as u32).to_le_bytes().to_vec();
      msg.extend_from_slice(&10u32.to_le_bytes()); // an expression to evaluate
      msg.extend_from_slice(body.as_bytes());
      s.write_all(&msg)?;
    }
    let head: String = form.chars().take(60).collect();
    if self.program.wait_for(mark, &[&done], timeout).is_none() {
      bail!(failed(&head, &self.program.since(mark)));
    }
    let answer = self.program.since(mark);
    let answer = answer.split(&done).next().unwrap_or_default().to_string();
    if ["REPL Error", "Compilation Error"]
      .iter()
      .any(|bad| answer.contains(bad))
    {
      bail!(failed(&head, &answer));
    }
    Ok(answer)
  }

  /// goalc makes `target`, a file of its project's outputs.
  fn make(&mut self, target: &str) -> Result<()> {
    let a = self.eval(&format!("(make \"{target}\")"), Duration::from_secs(900))?;
    if !a.contains("Successfully built") {
      bail!(failed(target, &a));
    }
    Ok(())
  }

  /// The value of `form` in the game, as goalc prints it after its prompt ("gc> 1  #x1 ..."): its
  /// first word.
  fn value(&mut self, form: &str) -> Result<String> {
    let a = self.eval(form, Duration::from_secs(20))?;
    Ok(
      a.lines()
        .filter_map(|l| l.trim_start().strip_prefix("gc> "))
        .filter_map(|l| l.split_whitespace().next())
        .next_back()
        .unwrap_or_default()
        .to_string(),
    )
  }

  fn connected(&self) -> bool {
    let o = lock(&self.program.out);
    o.rfind("gc> ") > o.rfind("g > ")
  }

  /// Asks `form` until the game answers 1.
  fn wait_value(&mut self, form: &str, timeout: Duration) -> Result<()> {
    let end = Instant::now() + timeout;
    while Instant::now() < end {
      if self.value(form)? == "1" {
        return Ok(());
      }
      if !self.connected() {
        bail!(failed(form, "the game stopped answering"));
      }
      std::thread::sleep(Duration::from_millis(500));
    }
    bail!(failed(form, "no answer in time"))
  }
}

/// a port of this computer no one listens on
fn free_port() -> u16 {
  TcpListener::bind(("127.0.0.1", 0))
    .and_then(|l| l.local_addr())
    .map(|a| a.port())
    .unwrap_or(0)
}

fn port_taken(port: u16) -> bool {
  TcpListener::bind(("0.0.0.0", port)).is_err() || TcpListener::bind(("127.0.0.1", port)).is_err()
}

/// gk's listener: 8112 for Jak 1, 8113 for Jak 2, 8114 for Jak 3, whatever its --port says
/// (DECI2_PORT - 1 + the game's version, game/runtime.cpp).
fn game_port(game: &str) -> u16 {
  match game {
    "jak2" => 8113,
    "jak3" => 8114,
    _ => 8112,
  }
}

/// `link` shows the folder `target` (a junction on Windows, which needs no rights; a symbolic link
/// elsewhere); an older link there is replaced.
fn link_folder(link: &Path, target: &Path) -> Result<()> {
  if std::fs::symlink_metadata(link).is_ok() {
    // a junction is removed as a folder, without what it shows
    std::fs::remove_dir(link).or_else(|_| std::fs::remove_file(link))?;
  }
  #[cfg(windows)]
  {
    use std::os::windows::process::CommandExt;
    let ok = Command::new("cmd")
      .raw_arg(format!(
        "/C mklink /J \"{}\" \"{}\"",
        link.display(),
        target.display()
      ))
      .creation_flags(0x0800_0000)
      .output()
      .is_ok_and(|o| o.status.success());
    anyhow::ensure!(
      ok,
      "{}: cannot link to {}",
      link.display(),
      target.display()
    );
    Ok(())
  }
  #[cfg(not(windows))]
  Ok(std::os::unix::fs::symlink(target, link)?)
}

fn copy_folder(from: &Path, to: &Path) -> Result<()> {
  std::fs::create_dir_all(to)?;
  for e in std::fs::read_dir(from)?.flatten() {
    let target = to.join(e.file_name());
    if e.path().is_dir() {
      copy_folder(&e.path(), &target)?;
    } else {
      std::fs::copy(e.path(), &target)?;
    }
  }
  Ok(())
}

/// A JSON file with comments (`//`, `/* */`), as OpenGOAL's configuration and levels are written.
fn read_jsonc(path: &Path) -> Result<Value> {
  let text = std::fs::read_to_string(path)?;
  let mut out = String::with_capacity(text.len());
  let mut chars = text.chars().peekable();
  let mut in_string = false;
  while let Some(c) = chars.next() {
    if in_string {
      out.push(c);
      match c {
        '\\' => out.extend(chars.next()),
        '"' => in_string = false,
        _ => {}
      }
      continue;
    }
    match (c, chars.peek()) {
      ('"', _) => {
        in_string = true;
        out.push(c);
      }
      ('/', Some('/')) => {
        while chars.peek().is_some_and(|&c| c != '\n') {
          chars.next();
        }
      }
      ('/', Some('*')) => {
        chars.next();
        let mut last = ' ';
        for c in chars.by_ref() {
          if last == '*' && c == '/' {
            break;
          }
          last = c;
        }
      }
      _ => out.push(c),
    }
  }
  serde_json::from_str(&out).with_context(|| path.display().to_string())
}

/// The quoted names of a DGO's list (`goal_src/<game>/dgos/*.gd`, a level's .gd): the DGO's, then its
/// files'.
fn quoted(gd: &str) -> Vec<String> {
  gd.lines()
    .map(|l| l.split(';').next().unwrap_or_default())
    .flat_map(|l| {
      l.split('"')
        .skip(1)
        .step_by(2)
        .map(str::to_owned)
        .collect::<Vec<_>>()
    })
    .collect()
}

/// The DGOs (`DGO/VI1.DGO`) holding the art groups `wanted` (`plat-ag`), as the game's .gd files
/// list them: the level builder reads their textures, and no other DGO than those and GAME.CGO.
fn art_group_dgos(install: &Install, wanted: &[String]) -> Vec<String> {
  let dir = install
    .data
    .join("goal_src")
    .join(&install.game)
    .join("dgos");
  let mut out = vec!["CGO/GAME.CGO".to_owned()];
  for e in std::fs::read_dir(dir).into_iter().flatten().flatten() {
    let names = quoted(&std::fs::read_to_string(e.path()).unwrap_or_default());
    let Some(dgo) = names.first().filter(|n| n.ends_with(".DGO")) else {
      continue;
    };
    if wanted.iter().any(|w| names.contains(&format!("{w}.go"))) {
      out.push(format!("DGO/{dgo}"));
    }
  }
  out
}

/// The viewer, in the game's language: the model (art group ogas-model-ag) in front of Jak, playing
/// its animation `anim` again and again (`ogas-play`).
fn viewer_code(game: &str) -> String {
  // Jak 2 and 3 look for a process's art groups in its level only (skeleton-group->draw-control)
  let level = if game == "jak1" {
    String::new()
  } else {
    format!(
      "
  (set! (-> self level) (level-get *level* '{LEVEL}))"
    )
  };
  format!(
    ";; OpenGOAL Animation Studio's try in the game: the model tried, in front of Jak.

(def-actor {ACTOR} :bounds (0 0 0 10))

(deftype ogas-viewer (process-drawable)
  ((anim int32))
  (:state-methods idle))

(defstate idle (ogas-viewer)
  :virtual #t
  :code
    (behavior ()
      (loop
        (ja-no-eval :group! (-> self draw art-group data (+ (-> self anim) 2)) :num! (seek!) :frame-num 0.0)
        (until (ja-done? 0)
          (suspend)
          (ja :num! (seek!)))))
  :post ja-post)

(defbehavior ogas-viewer-init-by-other ogas-viewer ((pos vector) (anim int))
  (set! (-> self root) (new 'process 'trsqv)){level}
  (vector-copy! (-> self root trans) pos)
  (quaternion-rotate-y! (-> self root quat) (-> *target* control quat) (degrees 180))
  (vector-identity! (-> self root scale))
  (initialize-skeleton self *{ACTOR}-sg* '())
  ;; its animations are the art group's elements after the skeleton and the mesh
  (set! (-> self anim) (max 0 (min anim (+ -3 (-> self draw art-group length)))))
  (logclear! (-> self mask) (process-mask actor-pause))
  (go-virtual idle))

(define *ogas-viewer* (the-as (pointer ogas-viewer) #f))

(defun ogas-play ((anim int))
  \"The model 4 m in front of Jak and 2 m aside (the camera behind him sees it), facing him,
   playing its animation `anim` (0 the first).\"
  (let ((old (if *ogas-viewer* (ppointer->process *ogas-viewer*))))
    (if old (deactivate old)))
  (let ((pos (new 'stack-no-clear 'vector))
        (side (new 'stack-no-clear 'vector)))
    (vector-z-quaternion! pos (-> *target* control quat))
    (vector-float*! pos pos (meters 4))
    (vector-x-quaternion! side (-> *target* control quat))
    (vector-float*! side side (meters 2))
    (vector+! pos pos side)
    (vector+! pos pos (-> *target* control trans))
    (set! *ogas-viewer* (process-spawn ogas-viewer :init ogas-viewer-init-by-other pos anim :to *entity-pool*)))
  (if *ogas-viewer* 1 0))
"
  )
}

/// What a try keeps: goalc, the game started, the game's files it replaced (each one's place and
/// where it is kept meanwhile).
#[derive(Default)]
struct Session {
  goalc: Option<Goalc>,
  gk: Option<Program>,
  kept: Vec<(PathBuf, PathBuf)>,
}

impl Session {
  fn stop(&mut self) {
    if let Some(mut gk) = self.gk.take() {
      gk.kill();
    }
    if let Some(mut g) = self.goalc.take() {
      // killed, not quit: goalc quitting resets the game it is connected to
      g.program.kill();
    }
    for (place, copy) in self.kept.drain(..) {
      if copy.is_file() {
        if std::fs::copy(&copy, &place).is_ok() {
          let _ = std::fs::remove_file(&copy);
        }
      } else {
        let _ = std::fs::remove_file(&place);
      }
    }
  }
}

/// The tries of the studio: one game at a time.
#[derive(Default)]
pub struct GameTest {
  session: Mutex<Session>,
}

impl Drop for GameTest {
  fn drop(&mut self) {
    self.stop();
  }
}

impl GameTest {
  /// The game closed, goalc stopped, the game's files put back.
  pub fn stop(&self) {
    lock(&self.session).stop();
  }

  /// Whether the game of the last try is still open.
  pub fn running(&self) -> bool {
    lock(&self.session).gk.as_mut().is_some_and(Program::alive)
  }

  /// The model in the running game plays its animation `anim` (0 the first).
  pub fn play(&self, anim: usize) -> Result<()> {
    let mut s = lock(&self.session);
    let goalc = s.goalc.as_mut().context("no game is open")?;
    goalc.wait_value(&format!("(ogas-play {anim})"), Duration::from_secs(20))
  }

  /// `model` tried in `game`, installed in `root`, playing its animation `anim`; `step` is told each
  /// step ("export", "compiler", "build", "start", "load", "play").
  /// What goalc and the game printed go to <prefs>/game-test/<game>/goalc.log and gk.log.
  pub fn try_model(
    &self,
    model: &Path,
    root: &Path,
    game: &str,
    anim: usize,
    step: impl Fn(&str),
  ) -> Result<()> {
    let mut s = lock(&self.session);
    let mut r = self.run(&mut s, model, root, game, anim, step);
    if r.is_err()
      && let Some(gk) = s.gk.as_mut()
      && !gk.alive()
    {
      r = Err(
        text(
          "game-closed",
          "The game closed during the try: {output}",
          &[("output", last_lines(&gk.since(0), 6))],
        )
        .into(),
      );
    }
    let dir = Settings::dir().join("game-test").join(game);
    for (name, p) in [
      ("goalc.log", s.goalc.as_ref().map(|g| &g.program)),
      ("gk.log", s.gk.as_ref()),
    ] {
      if let Some(p) = p {
        let _ = std::fs::write(dir.join(name), p.since(0));
      }
    }
    if r.is_err()
      && let Some(mut gk) = s.gk.take()
    {
      gk.kill();
    }
    r
  }

  fn run(
    &self,
    s: &mut Session,
    model: &Path,
    root: &Path,
    game: &str,
    anim: usize,
    step: impl Fn(&str),
  ) -> Result<()> {
    let install = find_install(game, root)?;
    let ours = s.gk.as_mut().is_some_and(Program::alive);
    let port = game_port(game);
    if !ours && port_taken(port) {
      bail!(text(
        "game-port-taken",
        "Another game listens on port {port}: close it first.",
        &[("port", port.to_string())]
      ));
    }
    if s.goalc.as_ref().is_some_and(|g| g.data != install.data) {
      s.stop();
    }
    let dir = Settings::dir().join("game-test").join(game);
    let proj = project(&install, &dir)?;

    step("export");
    let mut glb = Glb::read(model)?;
    // what the game would show wrong, it shows: a try is there to see it
    edit::normalize(&mut glb, ACTOR)?;
    hold_still_poses(&mut glb)?;
    let models = proj
      .join("custom_assets")
      .join(game)
      .join("models/custom_levels");
    glb.write(&models.join(format!("{ACTOR}.glb")))?;
    let viewer = dir.join("ogas-viewer.gc");
    std::fs::write(&viewer, viewer_code(game))?;
    let wanted = level_files(&install, &proj)?;

    step("compiler");
    let mut kept = s.goalc.take();
    if !kept.as_mut().is_some_and(|g| g.program.alive()) {
      kept = Some(start_goalc(&install, &proj)?);
    }
    let goalc = s.goalc.insert(kept.expect("kept or started"));
    // goalc learns the game's types: the code of GAME.CGO compiled (about a minute the first time)
    let engine = format!(
      "\"$OUT/obj/{}\"",
      engine_files(&install).join("\" \"$OUT/obj/")
    );
    let inputs = |root: &Path| {
      root
        .join("decompiler/config")
        .join(game)
        .join("ntsc_v1/inputs.jsonc")
    };
    let mut config = read_jsonc(&inputs(&install.data))?;
    config["dgo_names"] = json!(art_group_dgos(&install, &wanted));
    std::fs::write(inputs(&proj), serde_json::to_string_pretty(&config)?)?;
    let gp = proj.join("ogas-test.gp");
    std::fs::write(
      &gp,
      format!(
        ";; the game's project, the model as a custom actor and its viewer (OpenGOAL Animation Studio)\n\
         (load-file \"goal_src/{game}/game.gp\")\n\
         (build-actor \"{ACTOR}\")\n\
         (defstep :in \"{}\" :tool 'goalc :out '(\"$OUT/obj/ogas-viewer.o\") :dep '({engine}))\n",
        viewer.to_string_lossy().replace('\\', "/")
      ),
    )?;
    goalc.eval(
      &format!(
        "(load-project \"{}\")",
        gp.to_string_lossy().replace('\\', "/")
      ),
      Duration::from_secs(120),
    )?;

    step("build");
    goalc.make("$OUT/iso/TSZ.DGO")?;
    let built = proj.join("out").join(game);
    let size = std::fs::metadata(built.join(format!("obj/{ACTOR}-ag.go")))?.len() as usize;
    let max = if game == "jak1" {
      rules::JAK1_DGO_BUFFER
    } else {
      rules::DGO_BUFFER
    };
    if size > max {
      bail!(text(
        "art-group-too-big",
        "The model takes {size} KB once built, more than the {max} KB a level of the game loads it into: remove or shorten animations.",
        &[
          ("size", (size / 1024).to_string()),
          ("max", (max / 1024).to_string())
        ]
      ));
    }
    if let Some(mut old) = s.gk.take() {
      old.kill();
    }
    // the game's own kept in the try's folder: gk reads every file of its iso/ folder when it starts
    // (names of 15 letters at most)
    let kept = dir.join("kept");
    std::fs::create_dir_all(&kept)?;
    for file in ["iso/TSZ.DGO", "fr3/test-zone.fr3"] {
      let place = install.out().join(file);
      let copy = kept.join(file.replace('/', "-"));
      if !s.kept.iter().any(|(p, _)| *p == place) {
        // a copy left by a try the studio could not end is the game's own
        if place.is_file() && !copy.exists() {
          std::fs::copy(&place, &copy)?;
        }
        s.kept.push((place.clone(), copy));
      }
      std::fs::copy(built.join(file), &place).with_context(|| place.display().to_string())?;
    }

    step("start");
    if port_taken(port) {
      bail!(text(
        "game-port-taken",
        "Another game listens on port {port}: close it first.",
        &[("port", port.to_string())]
      ));
    }
    let mut cmd = Command::new(&install.gk);
    cmd
      .args(["--game", game, "--proj-path"])
      .arg(&install.data)
      .args(["--", "-boot", "-fakeiso", "-debug"]);
    let gk = s.gk.insert(Program::start(cmd)?);
    if gk
      .wait_for(0, &["calling play"], Duration::from_secs(90))
      .is_none()
    {
      bail!(failed("gk", &gk.since(0)));
    }
    let goalc = s.goalc.as_mut().expect("started above");
    for _ in 0..20 {
      goalc.eval("(lt)", Duration::from_secs(15))?;
      if goalc.connected() {
        break;
      }
      std::thread::sleep(Duration::from_millis(500));
    }
    goalc.wait_value("(if *target* 1 0)", Duration::from_secs(90))?;

    step("load");
    goalc.eval(
      &format!("(start 'play (get-continue-by-name *game-info* \"{LEVEL}-start\"))"),
      Duration::from_secs(30),
    )?;
    let status = if game == "jak3" {
      format!("(level-status? *level* '{LEVEL} #f)")
    } else {
      format!("(level-status *level* '{LEVEL})")
    };
    goalc.wait_value(
      &format!("(if (= {status} 'active) 1 0)"),
      Duration::from_secs(120),
    )?;
    goalc.wait_value("(if *target* 1 0)", Duration::from_secs(60))?;

    step("play");
    goalc.wait_value(&format!("(ogas-play {anim})"), Duration::from_secs(20))
  }
}

/// The animations whose keys are all at 0 s (a still pose, as the extraction writes the game's
/// one-frame animations) given a second key 1/60 s later: the build-actor of OpenGOAL's releases
/// (v0.3.9 tried) makes no frame of them, then dies.
fn hold_still_poses(glb: &mut Glb) -> Result<()> {
  for i in 0..glb.array("animations").len() {
    let samplers = glb.array("animations")[i]["samplers"]
      .as_array()
      .cloned()
      .unwrap_or_default();
    let times: Vec<Vec<f32>> = samplers
      .iter()
      .map(|s| glb.floats(&s["input"]))
      .collect::<Result<_>>()?;
    if times.iter().any(|t| t.last().is_some_and(|t| *t > 0.0)) {
      continue;
    }
    let input = glb.push_floats(&[0.0, 1.0 / 60.0], "SCALAR");
    for (j, s) in samplers.iter().enumerate() {
      let values = glb.floats(&s["output"])?;
      let n = values.len() / times[j].len().max(1);
      let key = values[..n].to_vec();
      let ty = if n == 4 { "VEC4" } else { "VEC3" };
      let output = glb.push_floats(&[key.clone(), key].concat(), ty);
      let sampler = &mut glb.json["animations"][i]["samplers"][j];
      sampler["input"] = json!(input);
      sampler["output"] = json!(output);
      sampler["interpolation"] = json!("LINEAR");
    }
  }
  Ok(())
}

/// goalc's project, made again for another installation: links to the installed game's sources
/// (goal_src, game/assets), disc (iso_data) and art (decompiler_out/<game>/raw_obj, which its project copies the
/// art groups from), a copy of the decompiler's configuration (its inputs.jsonc written for each
/// try), its own out/ and custom_assets/.
fn project(install: &Install, dir: &Path) -> Result<PathBuf> {
  let proj = dir.join("goalc");
  let data = &install.data;
  let game = &install.game;
  let marker = proj.join("installation.txt");
  let made_for = format!(
    "{}
2",
    data.display()
  );
  if std::fs::read_to_string(&marker).ok().as_deref() != Some(made_for.as_str()) {
    let _ = std::fs::remove_file(&marker);
    let _ = std::fs::remove_dir_all(proj.join("out"));
    let _ = std::fs::remove_dir_all(proj.join("custom_assets"));
    std::fs::create_dir_all(&proj)?;
    // game/assets/<game>/: the patches Jak 1's configuration applies to the disc's text files
    for f in ["goal_src", "iso_data", "game"] {
      if data.join(f).is_dir() {
        link_folder(&proj.join(f), &data.join(f))?;
      }
    }
    let config = proj.join("decompiler/config");
    copy_folder(
      &data.join("decompiler/config").join(game),
      &config.join(game),
    )?;
    // the other games' types, which the configuration names
    for other in ["jak1", "jak2"] {
      let from = data.join("decompiler/config").join(other);
      if other != game.as_str() && from.is_dir() {
        link_folder(&config.join(other), &from)?;
      }
    }
    let art = proj.join("decompiler_out").join(game);
    std::fs::create_dir_all(art.join("textures"))?;
    link_folder(
      &art.join("raw_obj"),
      &data.join("decompiler_out").join(game).join("raw_obj"),
    )?;
    std::fs::write(&marker, &made_for)?;
  }
  // the folders the game's build writes into (its builder does not make them)
  for f in ["obj", "iso", "fr3"] {
    std::fs::create_dir_all(proj.join("out").join(game).join(f))?;
  }
  std::fs::create_dir_all(
    proj
      .join("custom_assets")
      .join(game)
      .join("models/custom_levels"),
  )?;
  Ok(proj)
}

/// test-zone in goalc's project: the game's, with the model among its custom models (the level's
/// .fr3 holds their meshes) and in its DGO, laid out as the game's: the code first (the viewer), the
/// textures, the art groups (the model's last), the level itself. Returns the art groups the level
/// names.
fn level_files(install: &Install, proj: &Path) -> Result<Vec<String>> {
  let game = &install.game;
  let rel = |root: &Path| root.join("custom_assets").join(game);
  let (theirs, ours) = (rel(&install.data), rel(proj));
  copy_folder(
    &theirs.join("levels").join(LEVEL),
    &ours.join("levels").join(LEVEL),
  )?;
  // the game's own custom actors of test-zone (test-actor)
  for e in std::fs::read_dir(theirs.join("models/custom_levels"))?.flatten() {
    if e.path().extension().is_some_and(|x| x == "glb") {
      std::fs::copy(
        e.path(),
        ours.join("models/custom_levels").join(e.file_name()),
      )?;
    }
  }
  let level = ours.join("levels").join(LEVEL);
  let jsonc = level.join(format!("{LEVEL}.jsonc"));
  let mut json = read_jsonc(&jsonc)?;
  let models = json["custom_models"]
    .as_array()
    .cloned()
    .unwrap_or_default();
  if !models.iter().any(|m| m == ACTOR) {
    let mut models = models;
    models.push(json!(ACTOR));
    json["custom_models"] = json!(models);
  }
  std::fs::write(&jsonc, serde_json::to_string_pretty(&json)?)?;
  let gd = level.join("testzone.gd");
  let mut files = quoted(&std::fs::read_to_string(&gd)?);
  let dgo = files.remove(0);
  files.retain(|f| *f != format!("{ACTOR}-ag.go") && f != "ogas-viewer.o");
  files.insert(0, "ogas-viewer.o".to_owned());
  let level_go = files.len() - 1;
  files.insert(level_go, format!("{ACTOR}-ag.go"));
  std::fs::write(
    &gd,
    format!(
      "(\"{dgo}\"\n ({}\n ))\n",
      files
        .iter()
        .map(|f| format!("\"{f}\""))
        .collect::<Vec<_>>()
        .join("\n  ")
    ),
  )?;
  Ok(
    json["art_groups"]
      .as_array()
      .into_iter()
      .flatten()
      .filter_map(|a| a.as_str().map(str::to_owned))
      .collect(),
  )
}

/// The code files of GAME.CGO (goal_src/<game>/dgos/game.gd): compiled, goalc knows the game's
/// types.
fn engine_files(install: &Install) -> Vec<String> {
  let gd = install
    .data
    .join("goal_src")
    .join(&install.game)
    .join("dgos/game.gd");
  quoted(&std::fs::read_to_string(gd).unwrap_or_default())
    .into_iter()
    .filter(|f| f.ends_with(".o"))
    .collect()
}

fn start_goalc(install: &Install, proj: &Path) -> Result<Goalc> {
  let port = free_port();
  let mut cmd = Command::new(&install.goalc);
  cmd
    .args(["-g", &install.game, "--proj-path"])
    .arg(proj)
    .args([
      "-p",
      &port.to_string(),
      "--debug-port",
      &free_port().to_string(),
    ]);
  let mut program = Program::start(cmd)?;
  if program
    .wait_for(0, &["Listening on"], Duration::from_secs(30))
    .is_none()
  {
    program.kill();
    bail!(failed("goalc", &program.since(0)));
  }
  Ok(Goalc {
    program,
    port,
    data: install.data.clone(),
    evals: 0,
  })
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn dgo_lists_and_compiler_output_read() {
    assert_eq!(
      quoted("(\"TSZ.DGO\" ; the level\n (\"a.o\" ;; \"not.o\"\n \"b-ag.go\"))"),
      ["TSZ.DGO", "a.o", "b-ag.go"]
    );
    assert_eq!(
      strip_ansi("\x1b[0m\x1b[1m\x1b[38;2;000;255;255mg > \x1b[0m"),
      "g > "
    );
    assert_eq!(game_port("jak2"), 8113);
  }

  #[test]
  fn a_still_pose_gets_a_second_key() {
    let mut glb = Glb::default();
    glb.json = json!({ "asset": { "version": "2.0" } });
    let t = glb.push_floats(&[0.0], "SCALAR");
    let q = glb.push_floats(&[0.0, 0.0, 0.0, 1.0], "VEC4");
    let t2 = glb.push_floats(&[0.0, 1.0], "SCALAR");
    let q2 = glb.push_floats(&[0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0], "VEC4");
    glb.json["animations"] = json!([
      { "name": "still", "samplers": [{ "input": t, "output": q }], "channels": [] },
      { "name": "moving", "samplers": [{ "input": t2, "output": q2 }], "channels": [] }
    ]);
    hold_still_poses(&mut glb).unwrap();
    let keys = |a: usize| {
      let s = &glb.json["animations"][a]["samplers"][0];
      (
        glb.floats(&s["input"]).unwrap(),
        glb.floats(&s["output"]).unwrap(),
      )
    };
    assert_eq!(keys(0).0, [0.0, 1.0 / 60.0]);
    assert_eq!(keys(0).1, [0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0]);
    assert_eq!(keys(1).0, [0.0, 1.0], "a moving one is left as it is");
  }

  /// The whole try, in a game installed: `OGAS_GAME_INSTALL=<folder> OGAS_TRY_GAME=jak1
  /// OGAS_TRY_MODEL=<model.glb> cargo test -p animation-studio-core -- --ignored --nocapture
  /// try_in_the_game` (OGAS_TRY_SECONDS: how long the game stays open).
  #[test]
  #[ignore = "needs a game installed: OGAS_GAME_INSTALL=<folder>"]
  fn try_in_the_game() {
    let var = |k: &str| std::env::var(k).unwrap_or_else(|_| panic!("{k}"));
    let test = GameTest::default();
    let r = test.try_model(
      Path::new(&var("OGAS_TRY_MODEL")),
      Path::new(&var("OGAS_GAME_INSTALL")),
      &var("OGAS_TRY_GAME"),
      0,
      |s| println!("step {s}"),
    );
    let seconds = std::env::var("OGAS_TRY_SECONDS").map_or(5, |s| s.parse().unwrap_or(5));
    if r.is_ok() {
      for anim in [1, 0] {
        std::thread::sleep(Duration::from_secs(seconds));
        test.play(anim).unwrap();
      }
      std::thread::sleep(Duration::from_secs(seconds));
    }
    let log = lock(&test.session)
      .goalc
      .as_ref()
      .map(|g| g.program.since(0))
      .unwrap_or_default();
    std::fs::write(std::env::temp_dir().join("ogas-try-goalc.log"), log).unwrap();
    test.stop();
    r.unwrap();
  }
}
