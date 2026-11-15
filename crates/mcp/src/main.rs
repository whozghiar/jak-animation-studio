//! jak-animation-studio-mcp [--models <folder>] [--creations <folder>]: an MCP server (Model
//! Context Protocol: JSON-RPC 2.0, one message per line on stdin and stdout) that lets an assistant
//! such as Claude define the animations of an entity so that they follow what OpenGOAL's
//! build-actor needs, and play them in the studio's window. It works on the files, with the
//! studio's settings (read again on every call, so the game chosen in the window is the default):
//! every change goes to the model's creation (never to the original), and the studio's window shows
//! it as it lands. Nothing but the protocol goes to stdout.

use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

use animation_studio_core::{
  edit::{self, Pose},
  glb::Glb,
  library::{self, GAMES, MocapRequest, PlayRequest, Settings},
  model, names,
  rules::{self, Level, MAX_FRAME_BYTES, MAX_JOINTS},
};
use anyhow::{Context, Result, bail, ensure};
use serde_json::{Value, json};

const INSTRUCTIONS: &str = "Jak Animation Studio edits the animations of Jak and Daxter \
entities (skinned .glb models) for OpenGOAL's build-actor, game by game (jak1, jak2, jak3: the \
studio's game by default). Start with get_settings and list_models, then inspect_model: it lists the \
joints with their parent and rest position in the armature's space, where Y is up, the model faces \
+Z and its left is +X. An animation is a list of poses at times in seconds; each pose turns joints \
from their rest pose. 'turn' is the easy way: degrees about the armature's X, then Y, then Z axes, \
right-hand rule, as they are at rest (a joint's parents carry it when they move): [-90,0,0] swings \
a limb hanging down to the front, [0,0,90] swings it from down to the model's left (+X), [0,0,-90] \
to its right, [0,45,0] turns a head to its left, [20,0,0] bends a spine or a head forward. Read \
the rest positions to know where a limb points before turning it (a T-pose's arms point along X). \
'translate' moves a joint (metres in the parent's space); moving 'align' moves the entity in game. \
Joints not named keep their rest pose; between poses the game blends linearly, so for a loop repeat \
the first pose at the end. To make an animation from a description: inspect_model, create_animation, \
then play_animation so that the user sees it in the studio. To put a motion capture (BVH, FBX such as Mixamo's, glTF) on a humanoid model: apply_mocap. To see that the game plays an animation \
as the studio does: check_in_game. The game's rules, checked on every \
write: one armature, joint 0 named 'align' at the origin, parents before children, at most 114 \
joints, at most 2128 bytes of data per frame (about 8 bytes per rotating joint), only translation, \
rotation and scale keys. Changes go to the model's copy in the creations folder; import_model \
brings a model from elsewhere; export_for_game writes the files for a jak-project.";

fn tool(name: &str, description: &str, properties: Value, required: &[&str]) -> Value {
  json!({
    "name": name,
    "description": description,
    "inputSchema": { "type": "object", "properties": properties, "required": required }
  })
}

fn tools() -> Value {
  let model = json!({ "type": "string", "description": "the model's name or official name (as list_models gives them) or its path" });
  let anim = json!({ "type": "string", "description": "the animation's name" });
  let game = json!({ "type": "string", "enum": GAMES, "description": "the game whose models and creations are meant: the studio's game by default" });
  let vec3 = |d: &str| json!({ "type": "array", "items": { "type": "number" }, "minItems": 3, "maxItems": 3, "description": d });
  json!([
    tool(
      "get_settings",
      "The folders the studio works with (a models folder per game), the game chosen in the studio, and the game's limits. Call it first.",
      json!({}),
      &[]
    ),
    tool(
      "list_models",
      "The living entities (characters, creatures, enemies, bosses) of a game's models folder, its objects left out; or all the models of its creations (the models already edited or imported).",
      json!({ "folder": { "type": "string", "enum": ["models", "creations"], "description": "models by default" }, "game": game }),
      &[],
    ),
    tool(
      "inspect_model",
      "A model's joints (index, name, parent, rest rotation, position in metres in the armature's space: Y up, facing +Z, its left on +X), its animations (name, duration, keyframes) and what the game would refuse in it.",
      json!({ "model": model, "game": game }),
      &["model"],
    ),
    tool(
      "create_animation",
      "Writes an animation from poses into the model's creation (copied from the model the first time), replacing one of the same name in its place. Returns the problems the game would have with the model.",
      json!({
        "model": model,
        "game": game,
        "name": anim,
        "poses": {
          "type": "array",
          "description": "the poses, at times in seconds from 0",
          "items": {
            "type": "object",
            "required": ["time", "joints"],
            "properties": {
              "time": { "type": "number" },
              "joints": {
                "type": "object",
                "description": "joint name -> its pose",
                "additionalProperties": {
                  "type": "object",
                  "properties": {
                    "turn": vec3("degrees about the armature's axes as they are at rest (X the model's left, Y up, Z forward): X, then Y, then Z, right-hand rule"),
                    "rotate": vec3("degrees about the joint's own rest axes: X, then Y, then Z (before turn)"),
                    "rotation": { "type": "array", "items": { "type": "number" }, "minItems": 4, "maxItems": 4, "description": "the local rotation itself, x y z w (instead of rotate)" },
                    "translate": vec3("metres added to the rest translation, in the parent's space")
                  }
                }
              }
            }
          }
        }
      }),
      &["model", "name", "poses"],
    ),
    tool(
      "play_animation",
      "Shows the model in the studio's window and plays one of its animations (the first by default). Starts the studio when it is not open.",
      json!({ "model": model, "game": game, "animation": anim }),
      &["model"],
    ),
    tool(
      "apply_mocap",
      "Puts the movement of a motion capture file (BVH, FBX such as Mixamo's, glTF with an animated skeleton) on a humanoid model, the bones of both matched by their names (hips, chest, neck, head, arms, legs), and saves it as an animation of the model's creation. The studio's window does it, as its New animation panel does (it starts the studio when it is not open, and plays the result there); waits for it, up to 2 minutes. Returns the joints that play each body part and what the game would refuse in the model.",
      json!({
        "model": model,
        "game": game,
        "file": { "type": "string", "description": "the motion capture file: .bvh, .fbx, .glb or .gltf" },
        "name": { "type": "string", "description": "the animation's name (the file's name by default)" },
        "in_place": { "type": "boolean", "description": "true (by default): the hips go up and down but do not travel; false: the travel goes to align, which moves the entity in game" },
        "fps": { "type": "integer", "enum": [15, 24, 30, 60], "description": "keys per second (30 by default)" },
        "hold_weapon": { "type": "boolean", "description": "true (by default): when the model's hands hold a weapon in its first animation (a guard's gun), the arms keep that pose and the weapon stays in both hands; false: the arms follow the capture, the weapon the hand holding it" }
      }),
      &["model", "file"],
    ),
    tool(
      "check_in_game",
      "Tries the model in the game (the studio's Try in the game: test-zone rebuilt with it, the game started, the animation playing again and again in front of Jak), then checks what the game plays: the art group's element must be the animation asked, its length in frames (60 a second) the studio's, and four screenshots the game takes over one loop show it (look at them: the pose must be the studio's). The studio's window does it (it starts the studio when it is not open); the first try takes a few minutes. The game stays open: the user closes it in the studio.",
      json!({
        "model": model,
        "game": game,
        "animation": { "type": "string", "description": "the animation to check (the last of the model, the one added last, by default)" }
      }),
      &["model"],
    ),
    tool(
      "delete_animation",
      "Deletes an animation from the model's creation.",
      json!({ "model": model, "game": game, "name": anim }),
      &["model", "name"]
    ),
    tool(
      "rename_animation",
      "Renames an animation of the model's creation.",
      json!({ "model": model, "game": game, "name": anim, "new_name": { "type": "string" } }),
      &["model", "name", "new_name"],
    ),
    tool(
      "import_model",
      "Copies an entity (a .glb with an armature) from anywhere into the game's creations, where it can be animated.",
      json!({ "path": { "type": "string", "description": "the .glb file" }, "game": game, "replace": { "type": "boolean", "description": "overwrite the creation of the same name, its animations with it (false by default)" } }),
      &["path"],
    ),
    tool(
      "export_for_game",
      "Writes the creation as build-actor wants it (in the animations that need it: keys made linear, joints keyed; the skin set so that any build-actor builds it right) at custom_assets/<game>/models/custom_levels/<model>.glb, in the jak-project folder of the settings or in the creation's export folder, with a README giving the lines to add to the game.",
      json!({ "model": model, "game": game }),
      &["model"],
    ),
  ])
}

struct Server {
  /// --models (for the settings' game) and --creations
  models: Option<PathBuf>,
  creations: Option<PathBuf>,
}

fn arg<'a>(args: &'a Value, key: &str) -> Result<&'a str> {
  args[key]
    .as_str()
    .with_context(|| format!("'{key}' is missing"))
}

/// The studio's program, next to this one.
fn studio() -> Option<PathBuf> {
  let exe = std::env::current_exe().ok()?;
  let path = exe.parent()?.join(format!(
    "jak-animation-studio{}",
    std::env::consts::EXE_SUFFIX
  ));
  path.is_file().then_some(path)
}

impl Server {
  fn settings(&self) -> Settings {
    let mut settings = Settings::load();
    if let Some(dir) = &self.models {
      let game = settings.game().to_owned();
      settings.models_dirs.insert(game, dir.clone());
    }
    if self.creations.is_some() {
      settings.creations_dir.clone_from(&self.creations);
    }
    settings
  }

  fn resolve(&self, settings: &Settings, game: &str, model: &str) -> Result<PathBuf> {
    let path = Path::new(model);
    if path.is_file() {
      return Ok(path.to_path_buf());
    }
    let wanted = model.trim_end_matches(".glb").to_lowercase();
    let dirs = [
      Some(settings.creations().join(game)),
      settings.models_dir(game).map(Path::to_path_buf),
    ];
    dirs
      .iter()
      .flatten()
      .flat_map(|d| library::list_models(d, game, true))
      .find(|m| {
        m.name.to_lowercase() == wanted
          || rules::goal_name(&m.name) == wanted
          || m.title.as_ref().is_some_and(|t| {
            t.en.to_lowercase() == wanted
              || t.fr.as_ref().is_some_and(|f| f.to_lowercase() == wanted)
          })
      })
      .map(|m| m.path)
      .with_context(|| {
        format!("no entity named '{model}' in the {game} creations nor its models folder")
      })
  }

  fn issues(&self, path: &Path) -> Result<Value> {
    let info = library::inspect(path)?;
    Ok(json!({ "creation": path, "animations": info.animations, "issues": info.issues }))
  }

  fn edit(
    &self,
    settings: &Settings,
    game: &str,
    args: &Value,
    change: impl FnOnce(&mut Glb) -> Result<()>,
  ) -> Result<Value> {
    let model = self.resolve(settings, game, arg(args, "model")?)?;
    let creation = library::edit_creation(&model, &settings.creations(), game, change)?;
    self.issues(&creation)
  }

  /// Asks the studio's window to play, starting the studio when no window takes the request.
  fn play(&self, request: PlayRequest) -> Result<&'static str> {
    let dir = Settings::dir();
    request.send(&dir)?;
    // an open window takes it within a third of a second
    for _ in 0..20 {
      std::thread::sleep(Duration::from_millis(100));
      if !PlayRequest::file(&dir).exists() {
        return Ok("playing in the studio");
      }
    }
    let Some(studio) = studio() else {
      return Ok("the studio is not open: it plays it if it is opened within a minute");
    };
    Command::new(studio)
      .stdin(Stdio::null())
      .stdout(Stdio::null())
      .stderr(Stdio::null())
      .spawn()
      .context("Unable to start the studio")?;
    Ok("the studio was not open: it is starting and plays it once open")
  }

  /// A request the window answers (a movement put on, a check in the game): its answer, waited for
  /// up to `wait`.
  fn ask(&self, request: PlayRequest, wait: Duration) -> Result<Value> {
    let dir = Settings::dir();
    let answer = library::answer_file(&dir);
    let _ = std::fs::remove_file(&answer);
    let at = request.at;
    self.play(request)?;
    let end = std::time::Instant::now() + wait;
    while std::time::Instant::now() < end {
      std::thread::sleep(Duration::from_millis(100));
      let Some(a) = std::fs::read(&answer)
        .ok()
        .and_then(|b| serde_json::from_slice::<Value>(&b).ok())
        .filter(|a| a["at"] == at)
      else {
        continue;
      };
      let _ = std::fs::remove_file(&answer);
      if let Some(error) = a["error"].as_str() {
        bail!("{error}");
      }
      return Ok(a);
    }
    bail!(
      "the studio did not answer within {} s: is its window open?",
      wait.as_secs()
    )
  }

  fn call(&self, name: &str, args: &Value) -> Result<Value> {
    let settings = self.settings();
    let game = match args["game"].as_str() {
      Some(g) => {
        ensure!(
          GAMES.contains(&g),
          "'{g}' is not a game ({})",
          GAMES.join(", ")
        );
        g
      }
      None => settings.game(),
    };
    match name {
      "get_settings" => Ok(json!({
        "modelsFolders": settings.models_dirs,
        "creationsFolder": settings.creations(),
        "jakProjectFolder": settings.jak_project_dir,
        "game": settings.game(),
        "games": GAMES,
        "limits": { "joints": MAX_JOINTS, "bytesPerFrame": MAX_FRAME_BYTES },
      })),
      "list_models" => {
        let dir = match args["folder"].as_str() {
          Some("creations") => settings.creations().join(game),
          _ => settings
            .models_dir(game)
            .with_context(|| {
              format!("no models folder is set for {game}: choose one in the studio's settings")
            })?
            .to_path_buf(),
        };
        Ok(json!(library::list_models(
          &dir,
          game,
          args["folder"] == "creations"
        )))
      }
      "inspect_model" => {
        let info = library::inspect(&self.resolve(&settings, game, arg(args, "model")?)?)?;
        let joints: Vec<Value> = info
          .joints
          .iter()
          .zip(model::rest_world(&info.joints))
          .enumerate()
          .map(|(i, (j, (_, p)))| {
            json!({
              "index": i, "name": j.name, "parent": j.parent.map(|p| &info.joints[p].name),
              "rotation": j.rotation, "position": p.map(|x| (x * 1000.0).round() / 1000.0),
            })
          })
          .collect();
        Ok(json!({
          "name": info.name, "title": names::title(game, &info.name), "path": info.path, "joints": joints,
          "animations": info.animations, "issues": info.issues,
        }))
      }
      "create_animation" => {
        let anim = arg(args, "name")?.to_owned();
        let poses: Vec<Pose> = serde_json::from_value(args["poses"].clone())
          .context("'poses' must be a list of { time, joints }")?;
        self.edit(&settings, game, args, |glb| {
          let joints = model::skeleton(glb)?;
          let channels = edit::channels_from_poses(&joints, &poses)?;
          edit::set_animation(glb, &anim, &channels)
        })
      }
      "play_animation" => {
        let model = self.resolve(&settings, game, arg(args, "model")?)?;
        let info = library::inspect(&model)?;
        let names: Vec<&str> = info.animations.iter().map(|a| a.name.as_str()).collect();
        let animation = match args["animation"].as_str() {
          Some(a) if !names.contains(&a) => bail!(
            "{} has no animation '{a}'; it has: {}",
            info.name,
            names.join(", ")
          ),
          Some(a) => Some(a.to_owned()),
          None => names.first().map(|a| (*a).to_owned()),
        };
        let status = self.play(PlayRequest {
          model: model.clone(),
          animation: animation.clone(),
          game: game.to_owned(),
          at: PlayRequest::now(),
          mocap: None,
          check: false,
        })?;
        Ok(json!({ "model": model, "animation": animation, "status": status }))
      }
      "apply_mocap" => {
        let model = self.resolve(&settings, game, arg(args, "model")?)?;
        let file = std::path::absolute(arg(args, "file")?)?;
        let ext = file
          .extension()
          .and_then(|e| e.to_str())
          .map(str::to_lowercase);
        ensure!(file.is_file(), "no file at {}", file.display());
        ensure!(
          matches!(ext.as_deref(), Some("bvh" | "fbx" | "glb" | "gltf")),
          "{}: a motion capture file is a .bvh, .fbx, .glb or .gltf",
          file.display()
        );
        let name = match args["name"].as_str() {
          Some(n) => n.to_owned(),
          None => rules::goal_name(&file.file_stem().unwrap_or_default().to_string_lossy()),
        };
        let fps = args["fps"].as_u64().unwrap_or(30) as u32;
        ensure!([15, 24, 30, 60].contains(&fps), "fps is 15, 24, 30 or 60");
        let answer = self.ask(
          PlayRequest {
            model,
            animation: Some(name.clone()),
            game: game.to_owned(),
            at: PlayRequest::now(),
            mocap: Some(MocapRequest {
              file,
              name,
              fps,
              in_place: args["in_place"].as_bool().unwrap_or(true),
              hold_weapon: args["hold_weapon"].as_bool().unwrap_or(true),
            }),
            check: false,
          },
          // the window loads the model, reads the file and retargets it: seconds, more as it starts
          Duration::from_secs(120),
        )?;
        let creation = PathBuf::from(answer["creation"].as_str().unwrap_or_default());
        let mut out = self.issues(&creation)?;
        out["animation"] = answer["animation"].clone();
        out["mapping"] = answer["mapping"].clone();
        out["duration"] = answer["duration"].clone();
        Ok(out)
      }
      "check_in_game" => {
        let model = self.resolve(&settings, game, arg(args, "model")?)?;
        let info = library::inspect(&model)?;
        let animation = match args["animation"].as_str() {
          Some(a) => info.animations.iter().find(|x| x.name == a),
          None => info.animations.last(),
        }
        .with_context(|| {
          format!(
            "{} has no animation {}",
            info.name,
            args["animation"].as_str().unwrap_or_default()
          )
        })?;
        let answer = self.ask(
          PlayRequest {
            model: model.clone(),
            animation: Some(animation.name.clone()),
            game: game.to_owned(),
            at: PlayRequest::now(),
            mocap: None,
            check: true,
          },
          // a first try compiles the game's code with goalc: minutes
          Duration::from_secs(900),
        )?;
        let check = &answer["check"];
        let frames = check["frames"].as_u64().unwrap_or(0);
        // build-actor samples the keys at 60 a second up to the last one
        let expected = (animation.duration * 60.0).ceil() as u64;
        let mut out = json!({
          "model": model,
          "animation": animation.name,
          "playsIt": check["animation"] == animation.name.as_str(),
          "slot": check["slot"],
          "framesInGame": frames,
          "framesExpected": expected,
          "lengthMatches": frames.abs_diff(expected) <= 1,
          "screenshots": check["screenshots"],
        });
        out["note"] = json!(
          "the game keeps playing it in front of Jak; the screenshots follow as images (deleted from the disk once sent)"
        );
        Ok(out)
      }
      "delete_animation" => {
        let anim = arg(args, "name")?.to_owned();
        self.edit(&settings, game, args, |glb| {
          edit::delete_animation(glb, &anim)
        })
      }
      "rename_animation" => {
        let (anim, new_name) = (
          arg(args, "name")?.to_owned(),
          arg(args, "new_name")?.to_owned(),
        );
        self.edit(&settings, game, args, |glb| {
          edit::rename_animation(glb, &anim, &new_name)
        })
      }
      "import_model" => {
        let path = library::import(
          Path::new(arg(args, "path")?),
          &settings.creations(),
          game,
          args["replace"].as_bool().unwrap_or(false),
        )?;
        self.issues(&path)
      }
      "export_for_game" => {
        let model = self.resolve(&settings, game, arg(args, "model")?)?;
        let creation = library::creation_for(&model, &settings.creations(), game)?;
        let export = library::export(
          &creation,
          game,
          settings.jak_project_dir.as_deref(),
          settings.lang.as_deref().unwrap_or("en"),
        )?;
        Ok(json!({
          "ready": !export.issues.iter().any(|i| i.level == Level::Error),
          "export": export,
        }))
      }
      _ => bail!("unknown tool '{name}'"),
    }
  }

  fn handle(&self, msg: &Value) -> Option<Value> {
    let id = msg.get("id")?.clone();
    let result = match msg["method"].as_str().unwrap_or("") {
      "initialize" => json!({
        "protocolVersion": msg["params"]["protocolVersion"].as_str().unwrap_or("2025-06-18"),
        "capabilities": { "tools": {} },
        "serverInfo": { "name": "jak-animation-studio", "version": env!("CARGO_PKG_VERSION") },
        "instructions": INSTRUCTIONS,
      }),
      "ping" => json!({}),
      "tools/list" => json!({ "tools": tools() }),
      "tools/call" => {
        let params = &msg["params"];
        let (content, is_error) = match self
          .call(params["name"].as_str().unwrap_or(""), &params["arguments"])
        {
          Ok(mut v) => {
            // the screenshots of a check: sent as images after the text, then deleted, so that
            // nothing of the check stays on the disk
            let mut images = Vec::new();
            if let Some(paths) = v["screenshots"].as_array().cloned() {
              for path in paths.iter().filter_map(Value::as_str) {
                if let Ok(bytes) = std::fs::read(path) {
                  images.push(
                    json!({ "type": "image", "data": base64(&bytes), "mimeType": "image/png" }),
                  );
                }
                let _ = std::fs::remove_file(path);
              }
              v["screenshots"] = json!(images.len());
            }
            let mut content = vec![
              json!({ "type": "text", "text": serde_json::to_string_pretty(&v).unwrap_or_default() }),
            ];
            content.extend(images);
            (content, false)
          }
          Err(e) => (
            vec![json!({ "type": "text", "text": format!("{e:#}") })],
            true,
          ),
        };
        json!({ "content": content, "isError": is_error })
      }
      method => {
        return Some(json!({
          "jsonrpc": "2.0", "id": id,
          "error": { "code": -32601, "message": format!("unknown method {method}") }
        }));
      }
    };
    Some(json!({ "jsonrpc": "2.0", "id": id, "result": result }))
  }
}

/// Standard base64 (RFC 4648), the encoding of an MCP image.
fn base64(bytes: &[u8]) -> String {
  const ABC: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
  let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
  for chunk in bytes.chunks(3) {
    let n = chunk
      .iter()
      .enumerate()
      .fold(0u32, |n, (i, b)| n | u32::from(*b) << (16 - 8 * i));
    for i in 0..4 {
      out.push(if i <= chunk.len() {
        ABC[(n >> (18 - 6 * i) & 63) as usize] as char
      } else {
        '='
      });
    }
  }
  out
}

#[test]
fn base64_is_rfc_4648() {
  assert_eq!(base64(b""), "");
  assert_eq!(base64(b"f"), "Zg==");
  assert_eq!(base64(b"fo"), "Zm8=");
  assert_eq!(base64(b"foo"), "Zm9v");
  assert_eq!(base64(b"foobar"), "Zm9vYmFy");
}

fn main() {
  let mut server = Server {
    models: None,
    creations: None,
  };
  let mut args = std::env::args().skip(1);
  while let Some(a) = args.next() {
    match a.as_str() {
      "--models" => server.models = args.next().map(PathBuf::from),
      "--creations" => server.creations = args.next().map(PathBuf::from),
      _ => {}
    }
  }
  let mut out = std::io::stdout().lock();
  for line in std::io::stdin().lock().lines() {
    let Ok(line) = line else { break };
    if line.trim().is_empty() {
      continue;
    }
    let reply = match serde_json::from_str::<Value>(&line) {
      Ok(msg) => server.handle(&msg),
      Err(e) => Some(json!({
        "jsonrpc": "2.0", "id": null,
        "error": { "code": -32700, "message": e.to_string() }
      })),
    };
    if let Some(reply) = reply {
      let _ = writeln!(out, "{reply}");
      let _ = out.flush();
    }
  }
}
