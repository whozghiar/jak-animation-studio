//! Animations written the way build-actor reads them: linear keys of 32-bit floats, and every joint
//! keyed in translation, rotation and scale (a joint without keys would go to the origin in game).

use std::collections::{BTreeMap, HashMap, HashSet};

use anyhow::{Context, Result, bail, ensure};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::{
  glb::Glb,
  math,
  model::{self, Joint, PATHS},
  rules::{Text, goal_name, is_identity, short_name},
};

/// The keys of one joint channel: `values` holds 3 numbers per time (4 for a rotation, x y z w),
/// in the joint's parent space, as glTF stores them.
#[derive(Deserialize, Debug, Clone)]
pub struct Channel {
  pub joint: usize,
  pub path: String,
  pub times: Vec<f32>,
  pub values: Vec<f32>,
}

/// A pose at a time, given joint by joint relative to the rest pose (for the MCP server).
#[derive(Deserialize, Debug)]
pub struct Pose {
  pub time: f32,
  pub joints: BTreeMap<String, JointPose>,
}

#[derive(Deserialize, Debug, Default)]
pub struct JointPose {
  /// degrees about X, then Y, then Z of the joint's own rest axes
  pub rotate: Option<[f32; 3]>,
  /// the local rotation itself (x, y, z, w), instead of `rotate`
  pub rotation: Option<[f32; 4]>,
  /// degrees about X, then Y, then Z of the armature's axes as they are at rest, after `rotate`;
  /// the joint's parents carry it when they move
  pub turn: Option<[f32; 3]>,
  /// metres added to the rest translation, in the parent's space
  pub translate: Option<[f32; 3]>,
}

fn index_of(glb: &Glb, name: &str) -> Option<usize> {
  glb
    .array("animations")
    .iter()
    .position(|a| a["name"].as_str() == Some(name))
}

/// Keys that never change stored once: build-actor then keeps them out of every frame's data.
fn collapse(times: &[f32], values: &[f32], n: usize) -> (Vec<f32>, Vec<f32>) {
  if values.chunks(n).all(|k| k == &values[..n]) {
    (vec![0.0], values[..n].to_vec())
  } else {
    (times.to_vec(), values.to_vec())
  }
}

fn build(glb: &mut Glb, joints: &[Joint], name: &str, channels: &[Channel]) -> Result<Value> {
  for c in channels {
    let n = PATHS
      .iter()
      .find(|(p, _)| *p == c.path)
      .with_context(|| {
        format!(
          "'{}' is not a joint channel (translation, rotation, scale)",
          c.path
        )
      })?
      .1;
    ensure!(c.joint < joints.len(), "there is no joint {}", c.joint);
    ensure!(
      !c.times.is_empty() && c.values.len() == c.times.len() * n,
      "the {} keys of '{}' need {n} numbers per time",
      c.path,
      joints[c.joint].name
    );
    ensure!(
      c.times[0] >= 0.0 && c.times.windows(2).all(|w| w[0] < w[1]),
      "the key times of '{}' must start at 0 or later and go up",
      joints[c.joint].name
    );
  }

  let mut samplers = Vec::new();
  let mut targets = Vec::new();
  let mut inputs: HashMap<Vec<u32>, usize> = HashMap::new();
  for (j, joint) in joints.iter().enumerate() {
    for (path, n) in PATHS {
      let (times, mut values) = match channels.iter().find(|c| c.joint == j && c.path == path) {
        Some(c) => collapse(&c.times, &c.values, n),
        None => (vec![0.0], joint.rest(path).to_vec()),
      };
      if path == "rotation" {
        for q in values.chunks_mut(4) {
          q.copy_from_slice(&math::normalize([q[0], q[1], q[2], q[3]]));
        }
      }
      let input = *inputs
        .entry(times.iter().map(|t| t.to_bits()).collect())
        .or_insert_with(|| glb.push_floats(&times, "SCALAR"));
      let output = glb.push_floats(&values, if n == 4 { "VEC4" } else { "VEC3" });
      samplers.push(json!({ "input": input, "output": output, "interpolation": "LINEAR" }));
      targets.push(json!({
        "sampler": samplers.len() - 1, "target": { "node": joint.node, "path": path }
      }));
    }
  }
  Ok(json!({ "name": name, "samplers": samplers, "channels": targets }))
}

/// Writes animation `name` (replacing the one of that name, in its place). Joints without keys in
/// `channels` keep their rest pose.
pub fn set_animation(glb: &mut Glb, name: &str, channels: &[Channel]) -> Result<()> {
  ensure!(
    !name.trim().is_empty(),
    Text::new("animation-unnamed", "An animation needs a name.", &[])
  );
  let joints = model::skeleton(glb)?;
  let anim = build(glb, &joints, name, channels)?;
  match index_of(glb, name) {
    // the place of an animation is its slot in the art group: kept when it is replaced
    Some(i) => glb.array_mut("animations")[i] = anim,
    None => glb.array_mut("animations").push(anim),
  }
  glb.compact();
  Ok(())
}

pub fn channels_from_poses(joints: &[Joint], poses: &[Pose]) -> Result<Vec<Channel>> {
  let mut poses: Vec<&Pose> = poses.iter().collect();
  poses.sort_by(|a, b| a.time.total_cmp(&b.time));
  let mut channels: Vec<Channel> = Vec::new();
  let mut key = |joint: usize, path: &str, time: f32, value: &[f32]| match channels
    .iter_mut()
    .find(|c| c.joint == joint && c.path == path)
  {
    Some(c) if c.times.last() == Some(&time) => {
      let n = value.len();
      let at = c.values.len() - n;
      c.values[at..].copy_from_slice(value);
    }
    Some(c) => {
      c.times.push(time);
      c.values.extend_from_slice(value);
    }
    None => channels.push(Channel {
      joint,
      path: path.into(),
      times: vec![time],
      values: value.to_vec(),
    }),
  };
  let world = model::rest_world(joints);
  for pose in poses {
    for (name, p) in &pose.joints {
      let j = joints
        .iter()
        .position(|j| &j.name == name)
        .with_context(|| {
          format!("there is no joint named '{name}' (inspect_model lists the joints)")
        })?;
      let rest = &joints[j];
      let mut rotation = p.rotation.or_else(|| {
        p.rotate
          .map(|e| math::mul(rest.rotation, math::from_euler_xyz_degrees(e)))
      });
      if let Some(e) = p.turn {
        // the turn in the parent's rest space: parent⁻¹ × turn × parent × local
        let parent = rest.parent.map_or([0.0, 0.0, 0.0, 1.0], |p| world[p].0);
        let local = rotation.unwrap_or(rest.rotation);
        rotation = Some(math::mul(
          math::mul(math::inverse(parent), math::from_euler_xyz_degrees(e)),
          math::mul(parent, local),
        ));
      }
      if let Some(q) = rotation {
        key(j, "rotation", pose.time, &math::normalize(q));
      }
      if let Some(t) = p.translate {
        let v = [0, 1, 2].map(|i| rest.translation[i] + t[i]);
        key(j, "translation", pose.time, &v);
      }
    }
  }
  if channels.is_empty() {
    bail!("the poses move no joint");
  }
  Ok(channels)
}

/// Adds the animations of `source`, a model with the same skeleton, that `glb` lacks (by name): the
/// keys of each joint go to the joint of the same name, made linear. Morph (weights) keys are left
/// out: the blend shapes are each mesh's own. Returns how many were added.
pub fn copy_animations(glb: &mut Glb, source: &Glb) -> Result<usize> {
  let ours = model::skeleton(glb)?;
  let theirs = model::skeleton(source)?;
  let have: HashSet<String> = glb
    .array("animations")
    .iter()
    .enumerate()
    .map(|(i, a)| model::animation_name(a, i))
    .collect();
  let mut added = 0;
  for (i, anim) in source.array("animations").iter().enumerate() {
    let name = model::animation_name(anim, i);
    if have.contains(&name) {
      continue;
    }
    let mut samplers = Vec::new();
    let mut targets = Vec::new();
    let mut inputs: HashMap<Vec<u32>, usize> = HashMap::new();
    for c in model::channels(source, anim) {
      let joint = theirs.iter().find(|j| j.node == c.node);
      let node = joint.and_then(|j| ours.iter().find(|o| o.name == j.name));
      let (Some(node), Ok((times, values)), Some((_, n))) =
        (node, c.keys, PATHS.iter().find(|(p, _)| *p == c.path))
      else {
        continue;
      };
      let input = *inputs
        .entry(times.iter().map(|t| t.to_bits()).collect())
        .or_insert_with(|| glb.push_floats(&times, "SCALAR"));
      let output = glb.push_floats(&values, if *n == 4 { "VEC4" } else { "VEC3" });
      samplers.push(json!({ "input": input, "output": output, "interpolation": "LINEAR" }));
      targets.push(json!({
        "sampler": samplers.len() - 1, "target": { "node": node.node, "path": c.path }
      }));
    }
    if targets.is_empty() {
      continue;
    }
    glb
      .array_mut("animations")
      .push(json!({ "name": name, "samplers": samplers, "channels": targets }));
    added += 1;
  }
  Ok(added)
}

pub fn delete_animation(glb: &mut Glb, name: &str) -> Result<()> {
  let i = index_of(glb, name).with_context(|| {
    Text::new(
      "no-such-animation",
      "There is no animation '{name}'.",
      &[("name", name.to_owned())],
    )
  })?;
  glb.array_mut("animations").remove(i);
  if glb.array("animations").is_empty()
    && let Some(root) = glb.json.as_object_mut()
  {
    root.remove("animations");
  }
  glb.compact();
  Ok(())
}

pub fn rename_animation(glb: &mut Glb, name: &str, new_name: &str) -> Result<()> {
  ensure!(
    !new_name.trim().is_empty(),
    Text::new("animation-unnamed", "An animation needs a name.", &[])
  );
  ensure!(
    index_of(glb, new_name).is_none(),
    Text::new(
      "animation-exists",
      "An animation is already named '{name}'.",
      &[("name", new_name.to_owned())]
    )
  );
  let i = index_of(glb, name).with_context(|| {
    Text::new(
      "no-such-animation",
      "There is no animation '{name}'.",
      &[("name", name.to_owned())],
    )
  })?;
  glb.array_mut("animations")[i]["name"] = json!(new_name);
  Ok(())
}

/// Makes what build-actor needs of the animations that lack it: morph (weights) keys removed, keys
/// on nodes outside the armature dropped, cubic spline and step keys made linear, every joint keyed.
/// The others, their names and the rest of the model are left as they are. Returns what it changed.
pub fn normalize(glb: &mut Glb, actor: &str) -> Result<Vec<Text>> {
  let joints = model::skeleton(glb)?;
  let mut notes = Vec::new();
  let mut symbols = HashSet::new();
  for (i, anim) in glb.array("animations").to_vec().iter().enumerate() {
    let name = model::animation_name(anim, i);
    // def-actor names each slot <actor>-<short name>-ja
    let symbol = format!("{actor}-{}", goal_name(short_name(&name, actor)));
    ensure!(
      symbols.insert(symbol.clone()),
      Text::new(
        "game-name-taken",
        "Two animations would be named '{name}' in game: rename one.",
        &[("name", symbol)]
      )
    );
    let before = notes.len();
    let mut channels = Vec::new();
    let mut keyed = HashSet::new();
    for c in model::channels(glb, anim) {
      if c.path == "weights" {
        notes.push(Text::new(
          "weights-removed",
          "{anim}: morph target (weights) keys removed",
          &[("anim", name.clone())],
        ));
        continue;
      }
      let Some(joint) = joints.iter().position(|j| j.node == c.node) else {
        notes.push(Text::new(
          "node-dropped",
          "{anim}: keys of node {node} (not a joint) dropped",
          &[("anim", name.clone()), ("node", c.node.to_string())],
        ));
        continue;
      };
      if c.interpolation != "LINEAR" {
        notes.push(Text::new(
          "made-linear",
          "{anim}: {kind} keys made linear",
          &[
            ("anim", name.clone()),
            ("kind", c.interpolation.to_lowercase()),
          ],
        ));
      }
      let (times, values) = c.keys.with_context(|| format!("{name} cannot be read"))?;
      keyed.insert((joint, c.path.clone()));
      channels.push(Channel {
        joint,
        path: c.path,
        times,
        values,
      });
    }
    // build-actor gives a joint without keys identity values: only a joint resting elsewhere needs them
    let unkeyed = joints.iter().enumerate().any(|(j, joint)| {
      PATHS
        .iter()
        .any(|(p, _)| !keyed.contains(&(j, p.to_string())) && !is_identity(p, joint.rest(p)))
    });
    if unkeyed {
      notes.push(Text::new(
        "rest-keyed",
        "{anim}: joints without keys keyed at their rest pose",
        &[("anim", name.clone())],
      ));
    }
    if notes.len() > before {
      glb.array_mut("animations")[i] = build(glb, &joints, &name, &channels)?;
    }
  }
  if !notes.is_empty() {
    glb.compact();
  }
  notes.dedup();
  Ok(notes)
}
