//! What OpenGOAL's build-actor (goalc/build_actor/, the same for Jak 1, 2 and 3) needs of a model
//! and its animations, checked before the game's build stops on it or plays it wrong.

use std::collections::{BTreeMap, HashSet};

use serde::Serialize;

use crate::{glb::Glb, model};

/// The joints after the first two are described by 14 words of 8 joints each.
pub const MAX_JOINTS: usize = 2 + 14 * 8;
/// One frame of compressed data, and the constant part, each fit in 133 quadwords.
pub const MAX_FRAME_BYTES: usize = 133 * 16;
/// A level loads each file of its DGO, an art group whole, into one of two buffers (level.gc): 2 MB
/// in Jak 1, #x1b5800 bytes in Jak 2 and Jak 3 (less in Jak 3's smallest levels). Nothing checks it:
/// a bigger file overwrites the memory after the buffer.
pub const JAK1_DGO_BUFFER: usize = 2 * 1024 * 1024;
pub const DGO_BUFFER: usize = 0x1b5800;
/// build-actor samples every animation at 60 frames per second (`framerate` of build_actor.h).
const FRAMERATE: f32 = 60.0;
/// Beyond, a joint's translation takes the big encoding (s16 steps of 4/4096 m otherwise).
const SMALL_TRANS_LIMIT: f32 = 32767.0 * 4.0 / 4096.0;

/// Words for the user: the interface picks its own text (in its language) by `code`, filled with
/// `args`; `message` is the English one, for the MCP server. It is also an error.
#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct Text {
  pub code: &'static str,
  #[serde(skip_serializing_if = "BTreeMap::is_empty")]
  pub args: BTreeMap<&'static str, String>,
  pub message: String,
}

impl Text {
  /// `template` with each `{name}` replaced by its value in `args`.
  pub fn new(code: &'static str, template: &str, args: &[(&'static str, String)]) -> Text {
    let message = args.iter().fold(template.to_owned(), |s, (k, v)| {
      s.replace(&format!("{{{k}}}"), v)
    });
    Text {
      code,
      args: args.iter().cloned().collect(),
      message,
    }
  }
}

impl std::fmt::Display for Text {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(&self.message)
  }
}

impl std::error::Error for Text {}

#[derive(Serialize, Clone, Copy, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Level {
  Error,
  Warning,
}

#[derive(Serialize, Clone, Debug)]
pub struct Issue {
  pub level: Level,
  #[serde(flatten)]
  pub text: Text,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub animation: Option<String>,
}

/// A name the game can use for an art group or an animation: lowercase, digits and dashes.
pub fn goal_name(s: &str) -> String {
  let mut out = String::new();
  for c in s.to_lowercase().chars() {
    if c.is_ascii_alphanumeric() {
      out.push(c);
    } else if !out.ends_with('-') {
      out.push('-');
    }
  }
  out.trim_matches('-').to_owned()
}

/// An animation's name without its model's prefix, which may lack the model's `-lod0`:
/// "idle" for "babak-idle" of babak-lod0.glb.
pub fn short_name<'a>(name: &'a str, actor: &str) -> &'a str {
  let base = actor
    .rsplit_once("-lod")
    .filter(|(_, n)| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()))
    .map_or(actor, |(base, _)| base);
  [actor, base]
    .iter()
    .find_map(|prefix| name.strip_prefix(prefix)?.strip_prefix('-'))
    .unwrap_or(name)
}

pub(crate) fn is_identity(path: &str, v: &[f32]) -> bool {
  let near = |a: f32, b: f32| (a - b).abs() < 1e-5;
  match path {
    "translation" => v.iter().all(|x| near(*x, 0.0)),
    "rotation" => near(v[3].abs(), 1.0),
    _ => v.iter().all(|x| near(*x, 1.0)),
  }
}

/// Bytes of compressed data per frame and of the constant part, as build-actor lays them out.
fn data_bytes(glb: &Glb, joints: &[model::Joint], anim: &serde_json::Value) -> (usize, usize) {
  let channels = model::channels(glb, anim);
  // without an `align` first, build-actor puts one in front: the file's joint i is the game's i + 1
  let offset = usize::from(joints.first().is_none_or(|j| j.name != "align"));
  let (mut frame, mut fixed) = (0, if offset == 1 { 64 } else { 0 });
  for (i, joint) in joints.iter().enumerate() {
    let keys = |path: &str| {
      channels
        .iter()
        .find(|c| c.node == joint.node && c.path == path)
        .and_then(|c| c.keys.as_ref().ok())
    };
    let n = |path: &str| {
      model::PATHS
        .iter()
        .find(|(p, _)| *p == path)
        .map_or(3, |(_, n)| *n)
    };
    let animated =
      |path: &str| keys(path).is_some_and(|(t, v)| varies(t, v, n(path), path == "rotation"));
    let mut add = |bytes: usize, moving: bool| {
      if moving {
        frame += bytes
      } else {
        fixed += bytes
      }
    };
    if i + offset < 2 {
      add(64, model::PATHS.iter().any(|(p, _)| animated(p)));
      continue;
    }
    let big = keys("translation")
      .map_or(&joint.translation[..], |(_, v)| v)
      .iter()
      .any(|x| x.abs() > SMALL_TRANS_LIMIT);
    add(if big { 12 } else { 6 }, animated("translation"));
    add(8, animated("rotation"));
    add(6, animated("scale"));
  }
  (frame, fixed)
}

/// Whether the frames build-actor makes of a channel change, as it makes them (compute_keyframes of
/// animation_processing.cpp): a sample each 1/60 s blending the keys around it, in 32-bit floats. A
/// blend of equal keys can come out a rounding apart: then the channel takes room in every frame.
fn varies(times: &[f32], values: &[f32], n: usize, quaternion: bool) -> bool {
  let last = times[times.len() - 1];
  if times.len() == 1 || last <= 0.0 {
    return false;
  }
  let (mut i, mut t, mut first) = (0, 0.0f32, None);
  while t < last {
    while i + 1 < times.len() && times[i + 1] < t {
      i += 1;
    }
    let f = (t - times[i]) / (times[i + 1] - times[i]);
    let (a, b) = (&values[i * n..i * n + n], &values[(i + 1) * n..(i + 2) * n]);
    let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let sign = if quaternion && dot < 0.0 { -1.0 } else { 1.0 };
    let frame: Vec<f32> = (0..n).map(|c| a[c] * (1.0 - f) + b[c] * f * sign).collect();
    match &first {
      None => first = Some(frame),
      Some(first) if *first != frame => return true,
      _ => {}
    }
    t += 1.0 / FRAMERATE;
  }
  false
}

/// How many frames build-actor makes of an animation: one each 1/60 s up to its last key, at least
/// one (compute_keyframes of animation_processing.cpp).
fn frames(glb: &Glb, anim: &serde_json::Value) -> usize {
  let last = model::channels(glb, anim)
    .into_iter()
    .filter_map(|c| c.keys.ok()?.0.last().copied())
    .fold(0.0, f32::max);
  let (mut t, mut n) = (0.0, 0);
  while t < last {
    n += 1;
    t += 1.0 / FRAMERATE;
  }
  n.max(1)
}

/// How many weights of the skinned meshes go to a joint past the skin's last, and the highest one.
fn weights_outside(glb: &Glb, joints: usize) -> (usize, usize) {
  let meshes: HashSet<u64> = glb
    .array("nodes")
    .iter()
    .filter(|n| n.get("skin").is_some())
    .filter_map(|n| n["mesh"].as_u64())
    .collect();
  let mut seen = HashSet::new();
  let (mut outside, mut highest) = (0, 0);
  for mesh in meshes {
    for prim in glb
      .array("meshes")
      .get(mesh as usize)
      .and_then(|m| m["primitives"].as_array())
      .into_iter()
      .flatten()
    {
      let attrs = &prim["attributes"];
      if !seen.insert((attrs["JOINTS_0"].as_u64(), attrs["WEIGHTS_0"].as_u64())) {
        continue;
      }
      let (Ok(index), Ok(weight)) = (
        glb.numbers(&attrs["JOINTS_0"]),
        glb.numbers(&attrs["WEIGHTS_0"]),
      ) else {
        continue;
      };
      for (j, w) in index.iter().zip(&weight) {
        if *w > 0.0 && *j as usize >= joints {
          outside += 1;
          highest = highest.max(*j as usize);
        }
      }
    }
  }
  (outside, highest)
}

pub fn check(glb: &Glb, model_name: &str) -> Vec<Issue> {
  let mut issues = Vec::new();
  let mut push = |level, text: Text, animation: Option<&str>| {
    issues.push(Issue {
      level,
      text,
      animation: animation.map(str::to_owned),
    })
  };
  let text = Text::new;
  use Level::*;

  let joints = match model::skeleton(glb) {
    Ok(j) => j,
    Err(e) => {
      let why = e.downcast::<Text>().unwrap_or_else(|e| {
        text(
          "skeleton",
          "{error}: it is not an entity.",
          &[("error", e.to_string())],
        )
      });
      push(Error, why, None);
      return issues;
    }
  };
  for (i, j) in joints.iter().enumerate() {
    if j.parent.is_some_and(|p| p > i) {
      push(
        Error,
        text(
          "joint-order",
          "Joint '{joint}' comes before its parent in the armature: build-actor needs every parent first.",
          &[("joint", j.name.clone())],
        ),
        None,
      );
    }
    for c in &j.foreign_children {
      push(
        Error,
        text(
          "joint-child",
          "Joint '{joint}' has a child that is not a joint ('{child}'): build-actor stops on it. Parent that object to the armature instead.",
          &[
            ("joint", j.name.clone()),
            (
              "child",
              glb.array("nodes")[*c]["name"]
                .as_str()
                .unwrap_or("?")
                .to_owned(),
            ),
          ],
        ),
        None,
      );
    }
  }
  let has_align = joints.first().is_some_and(|j| j.name == "align");
  if !has_align {
    push(
      Warning,
      text(
        "no-align",
        "The first joint is not 'align': build-actor adds one, so the game's joint numbers are one more than the file's, and 'align' (which moves the entity in game) cannot be animated.",
        &[],
      ),
      None,
    );
  } else if joints[0].translation.iter().any(|x| x.abs() > 1e-5) {
    push(
      Warning,
      text("align-origin", "Joint 'align' is not at the origin.", &[]),
      None,
    );
  }
  let count = joints.len() + usize::from(!has_align);
  if count > MAX_JOINTS {
    push(
      Error,
      text(
        "too-many-joints",
        "The armature has {count} joints, build-actor takes {max} at most.",
        &[
          ("count", count.to_string()),
          ("max", MAX_JOINTS.to_string()),
        ],
      ),
      None,
    );
  }
  if count < 2 {
    push(
      Error,
      text(
        "too-few-joints",
        "build-actor needs at least two joints.",
        &[],
      ),
      None,
    );
  }
  let (outside, highest) = weights_outside(glb, joints.len());
  if outside > 0 {
    push(
      Error,
      text(
        "skin-joints",
        "{outside} vertex weights go to joints the armature does not have (up to joint {highest}, it has {joints}): build-actor passes them on, and the game moves them with other joints.",
        &[
          ("outside", outside.to_string()),
          ("highest", highest.to_string()),
          ("joints", joints.len().to_string()),
        ],
      ),
      None,
    );
  }
  if goal_name(model_name) != model_name {
    push(
      Warning,
      text(
        "model-name",
        "The game names the art group after the file: '{suggested}' would be safer than '{name}' (lowercase, digits and dashes).",
        &[
          ("suggested", goal_name(model_name)),
          ("name", model_name.to_owned()),
        ],
      ),
      None,
    );
  }

  let anims = glb.array("animations");
  if anims.is_empty() {
    push(
      Warning,
      text(
        "no-animation",
        "The model has no animation: build-actor makes a still pose of it.",
        &[],
      ),
      None,
    );
  }
  let mut names = HashSet::new();
  let mut animation_bytes = 0;
  for (i, anim) in anims.iter().enumerate() {
    let name = model::animation_name(anim, i);
    let a = Some(name.as_str());
    if !names.insert(name.clone()) {
      push(
        Error,
        text(
          "duplicate-name",
          "Two animations are named '{name}'.",
          &[("name", name.clone())],
        ),
        a,
      );
    }
    let channels = model::channels(glb, anim);
    if channels.is_empty() {
      push(Error, text("empty", "The animation has no keys.", &[]), a);
      continue;
    }
    for c in &channels {
      let joint = joints.iter().find(|j| j.node == c.node);
      let target = (
        "target",
        joint.map_or_else(|| format!("#{}", c.node), |j| format!("'{}'", j.name)),
      );
      if c.path == "weights" {
        push(
          Error,
          text(
            "weights",
            "It has morph target (weights) keys on {target}: build-actor stops on them. Export removes them.",
            std::slice::from_ref(&target),
          ),
          a,
        );
      } else if joint.is_none() {
        push(
          Error,
          text(
            "not-a-joint",
            "It moves node {target}, which is not a joint of the armature: build-actor stops on it. Export drops these keys.",
            std::slice::from_ref(&target),
          ),
          a,
        );
      }
      match c.interpolation.as_str() {
        "CUBICSPLINE" => push(
          Error,
          text(
            "cubic-spline",
            "Its keys on {target} are cubic splines: build-actor reads linear keys only. Export makes them linear.",
            std::slice::from_ref(&target),
          ),
          a,
        ),
        "STEP" => push(
          Warning,
          text(
            "step",
            "Its keys on {target} are steps: the game blends between them.",
            std::slice::from_ref(&target),
          ),
          a,
        ),
        _ => {}
      }
      match &c.keys {
        Err(e) => push(
          Error,
          text(
            "keys",
            "Its keys on {target} cannot be read: {error}.",
            &[target, ("error", e.to_string())],
          ),
          a,
        ),
        Ok((t, _)) if t[0] > 1e-4 => push(
          Warning,
          text(
            "starts-late",
            "Its keys on {target} start at {time} s: the game starts at 0.",
            &[target, ("time", format!("{:.3}", t[0]))],
          ),
          a,
        ),
        Ok(_) => {}
      }
    }

    let mut missing = Vec::new();
    for j in &joints {
      for (path, _) in model::PATHS {
        let keyed = channels.iter().any(|c| c.node == j.node && c.path == path);
        if !keyed && !is_identity(path, j.rest(path)) {
          missing.push(format!("{} {path}", j.name));
        }
      }
    }
    if !missing.is_empty() {
      let shown: Vec<_> = missing.iter().take(4).cloned().collect();
      push(
        Error,
        text(
          "missing-keys",
          "{count} joint channels have no keys ({list}): the game puts them at the origin instead of their rest pose. Export gives them their rest pose.",
          &[
            ("count", missing.len().to_string()),
            (
              "list",
              format!(
                "{}{}",
                shown.join(", "),
                if missing.len() > 4 { ", ..." } else { "" }
              ),
            ),
          ],
        ),
        a,
      );
    }

    let (frame, fixed) = data_bytes(glb, &joints, anim);
    animation_bytes += fixed + frames(glb, anim) * frame;
    let max = ("max", MAX_FRAME_BYTES.to_string());
    if frame > MAX_FRAME_BYTES {
      push(
        Error,
        text(
          "frame-too-big",
          "One frame takes {bytes} bytes, build-actor allows {max}: move fewer joints (a channel whose keys never change costs nothing per frame).",
          &[("bytes", frame.to_string()), max.clone()],
        ),
        a,
      );
    }
    if fixed > MAX_FRAME_BYTES {
      push(
        Error,
        text(
          "fixed-too-big",
          "Its constant part takes {bytes} bytes, build-actor allows {max}.",
          &[("bytes", fixed.to_string()), max],
        ),
        a,
      );
    }
  }
  let size = ("size", (animation_bytes / 1024).to_string());
  if animation_bytes > JAK1_DGO_BUFFER {
    push(
      Error,
      text(
        "animations-too-big",
        "The animations take {size} KB once built, the mesh not counted: more than a level of any game loads an art group into (2048 KB in Jak and Daxter, 1750 KB in Jak II and Jak 3). Remove or shorten animations.",
        &[size],
      ),
      None,
    );
  } else if animation_bytes > DGO_BUFFER {
    push(
      Warning,
      text(
        "animations-too-big-23",
        "The animations take {size} KB once built, the mesh not counted: more than a level of Jak II or Jak 3 loads an art group into (1750 KB; 2048 KB in Jak and Daxter). Remove or shorten animations for those games.",
        &[size],
      ),
      None,
    );
  }
  issues
}
