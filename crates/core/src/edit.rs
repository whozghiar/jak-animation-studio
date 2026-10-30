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
  rules::{self, Text, is_identity},
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
  let mut symbols = HashMap::new();
  for (i, anim) in glb.array("animations").to_vec().iter().enumerate() {
    let name = model::animation_name(anim, i);
    let symbol = rules::game_name(actor, &name);
    if let Some(other) = symbols.insert(symbol.clone(), name.clone()) {
      bail!(rules::game_name_shared(&symbol, &other, &name));
    }
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

/// The value of linear keys at time `t`, held before the first key and after the last.
pub(crate) fn sample(times: &[f32], values: &[f32], t: f32) -> Vec<f32> {
  let n = values.len() / times.len();
  let i = times.partition_point(|&k| k <= t);
  if i == 0 || i == times.len() {
    let k = i.saturating_sub(1);
    return values[k * n..k * n + n].to_vec();
  }
  let f = (t - times[i - 1]) / (times[i] - times[i - 1]);
  let (a, b) = (&values[(i - 1) * n..i * n], &values[i * n..i * n + n]);
  // a rotation and its opposite are the same: blend towards the one on a's side
  let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
  let sign = if n == 4 && dot < 0.0 { -1.0 } else { 1.0 };
  let v: Vec<f32> = a
    .iter()
    .zip(b)
    .map(|(x, y)| x + (sign * y - x) * f)
    .collect();
  if n == 4 {
    math::normalize([v[0], v[1], v[2], v[3]]).to_vec()
  } else {
    v
  }
}

/// The pose of `align` (its rest and its keys) carried by its children, in the animations where
/// it is not the identity: the model moves the same once `align` has left the skin.
fn lift_align(glb: &mut Glb, joints: &[Joint]) -> Result<()> {
  let children: Vec<usize> = (1..joints.len())
    .filter(|&j| joints[j].parent == Some(0))
    .collect();
  let at_rest = PATHS.iter().all(|(p, _)| is_identity(p, joints[0].rest(p)));
  for i in 0..glb.array("animations").len() {
    let anim = glb.array("animations")[i].clone();
    let channels: Vec<Channel> = model::channels(glb, &anim)
      .into_iter()
      .filter_map(|c| {
        let joint = joints.iter().position(|j| j.node == c.node)?;
        let (times, values) = c.keys.ok()?;
        Some(Channel {
          joint,
          path: c.path,
          times,
          values,
        })
      })
      .collect();
    let moves = channels.iter().any(|c| {
      let n = if c.path == "rotation" { 4 } else { 3 };
      c.joint == 0 && c.values.chunks(n).any(|v| !is_identity(&c.path, v))
    });
    if at_rest && !moves {
      continue;
    }
    let at = |j: usize, path: &str, t: f32| {
      channels
        .iter()
        .find(|c| c.joint == j && c.path == path)
        .map_or_else(
          || joints[j].rest(path).to_vec(),
          |c| sample(&c.times, &c.values, t),
        )
    };
    let quat = |v: Vec<f32>| [v[0], v[1], v[2], v[3]];
    let mut out: Vec<Channel> = channels
      .iter()
      .filter(|c| c.joint != 0 && !children.contains(&c.joint))
      .cloned()
      .collect();
    for &child in &children {
      let mut times: Vec<f32> = channels
        .iter()
        .filter(|c| c.joint == 0 || c.joint == child)
        .flat_map(|c| c.times.iter().copied())
        .collect();
      times.sort_by(f32::total_cmp);
      times.dedup();
      if times.is_empty() {
        times.push(0.0);
      }
      let mut keys = [Vec::new(), Vec::new(), Vec::new()];
      for &t in &times {
        let (ta, ra, sa) = (
          at(0, "translation", t),
          quat(at(0, "rotation", t)),
          at(0, "scale", t),
        );
        let (tc, rc, sc) = (
          at(child, "translation", t),
          quat(at(child, "rotation", t)),
          at(child, "scale", t),
        );
        let moved = math::rotate(ra, [0, 1, 2].map(|k| sa[k] * tc[k]));
        keys[0].extend([0, 1, 2].map(|k| ta[k] + moved[k]));
        keys[1].extend(math::mul(ra, rc));
        keys[2].extend([0, 1, 2].map(|k| sa[k] * sc[k]));
      }
      for ((path, _), values) in PATHS.iter().zip(keys) {
        out.push(Channel {
          joint: child,
          path: path.to_string(),
          times: times.clone(),
          values,
        });
      }
    }
    let name = model::animation_name(&anim, i);
    glb.array_mut("animations")[i] = build(glb, joints, &name, &out)?;
  }
  Ok(())
}

/// Every point's joint numbers (JOINTS_0) raised by one.
fn raise_joint_numbers(glb: &mut Glb) -> Result<()> {
  let mut accessors: Vec<u64> = glb
    .array("meshes")
    .iter()
    .flat_map(|m| m["primitives"].as_array().into_iter().flatten())
    .filter_map(|p| p["attributes"]["JOINTS_0"].as_u64())
    .collect();
  accessors.sort_unstable();
  accessors.dedup();
  for a in accessors {
    let acc = &glb.array("accessors")[a as usize];
    let size = match acc["componentType"].as_u64() {
      Some(5121) => 1,
      Some(5123) => 2,
      _ => bail!("the joint numbers of accessor {a} are neither bytes nor shorts"),
    };
    let count = acc["count"].as_u64().unwrap_or(0) as usize;
    let view =
      &glb.array("bufferViews")[acc["bufferView"].as_u64().context("no buffer view")? as usize];
    let start = view["byteOffset"].as_u64().unwrap_or(0) as usize
      + acc["byteOffset"].as_u64().unwrap_or(0) as usize;
    let stride = view["byteStride"].as_u64().map_or(4 * size, |s| s as usize);
    for at in (0..count).flat_map(|i| (0..4).map(move |c| start + i * stride + c * size)) {
      let b = glb
        .bin
        .get_mut(at..at + size)
        .context("joint numbers past the end of the data")?;
      if size == 1 {
        b[0] = b[0].checked_add(1).context("more than 255 joints")?;
      } else {
        let v = u16::from_le_bytes([b[0], b[1]]) + 1;
        b.copy_from_slice(&v.to_le_bytes());
      }
    }
  }
  Ok(())
}

/// Sets the skin so that any build-actor builds the model right. build-actor puts an `align` of its
/// own in front of the skeleton (OpenGOAL v0.3.9 always, whozghiar's tooling when the skin does not
/// start with one), while the mesh gives each point the matrix of its joint's number + 1, which the
/// game moves with the joint before it (common/util/gltf_util.cpp; bones.gc: matrix k follows joint
/// k - 1): every point would follow the joint listed before its own, and the model looks stretched.
/// So the model's `align` leaves the skin, its pose carried by its children: every build-actor then
/// adds its own, and the joints and the points keep their numbers. A skin without `align` gets its
/// points' joint numbers raised by one instead. Written for build-actor only: other tools read the
/// points' joints one joint off.
pub fn for_build_actor(glb: &mut Glb) -> Result<()> {
  let joints = model::skeleton(glb)?;
  let skin = *model::skins(glb).first().context("no skin")?;
  if joints[0].name != "align" {
    raise_joint_numbers(glb)?;
    return Ok(());
  }
  lift_align(glb, &joints)?;
  let align = joints[0].node as u64;
  for anim in glb.array_mut("animations") {
    if let Some(channels) = anim["channels"].as_array_mut() {
      // build-actor stops on the keys of a node outside the skin
      channels.retain(|c| c["target"]["node"].as_u64() != Some(align));
    }
  }
  let ibm = glb.array("skins")[skin]["inverseBindMatrices"].clone();
  if !ibm.is_null() {
    let matrices = glb.floats(&ibm)?;
    let kept = glb.push_floats(&matrices[16..], "MAT4");
    glb.array_mut("skins")[skin]["inverseBindMatrices"] = json!(kept);
  }
  if let Some(j) = glb.array_mut("skins")[skin]["joints"].as_array_mut() {
    j.remove(0);
  }
  glb.compact();
  Ok(())
}

/// Drops the points no triangle draws. The extraction gives every primitive of a model its level's
/// whole buffer of points (Babak draws 3 545 of the 17 753 to 78 578 his copies hold), and
/// build-actor puts them all in the level. The primitives that share points keep sharing them,
/// numbered again.
pub fn trim_points(glb: &mut Glb) -> Result<()> {
  // the primitives by the accessors of their points: attributes, then morph targets
  let mut groups: BTreeMap<Vec<u64>, Vec<(usize, usize)>> = BTreeMap::new();
  for (m, mesh) in glb.array("meshes").iter().enumerate() {
    for (p, prim) in mesh["primitives"]
      .as_array()
      .into_iter()
      .flatten()
      .enumerate()
    {
      let points = |v: &Value| -> Vec<u64> {
        let mut a: Vec<(&String, u64)> = v
          .as_object()
          .into_iter()
          .flatten()
          .filter_map(|(k, i)| Some((k, i.as_u64()?)))
          .collect();
        a.sort();
        a.into_iter().map(|(_, i)| i).collect()
      };
      let mut key = points(&prim["attributes"]);
      for t in prim["targets"].as_array().into_iter().flatten() {
        key.extend(points(t));
      }
      groups.entry(key).or_default().push((m, p));
    }
  }
  for (key, prims) in groups {
    let prim = |glb: &Glb, (m, p): (usize, usize)| glb.array("meshes")[m]["primitives"][p].clone();
    let Some(&first) = key.first() else {
      continue;
    };
    let count = glb.array("accessors")[first as usize]["count"]
      .as_u64()
      .unwrap_or(0) as usize;
    let mut indices = Vec::new();
    for &at in &prims {
      // a primitive without indices draws all its points
      let Ok(i) = glb.numbers(&prim(glb, at)["indices"]) else {
        indices.clear();
        break;
      };
      indices.push(i.into_iter().map(|v| v as usize).collect::<Vec<_>>());
    }
    let used: std::collections::BTreeSet<usize> = indices.iter().flatten().copied().collect();
    if indices.is_empty() || used.len() == count || used.iter().any(|&i| i >= count) {
      continue;
    }
    let keep: Vec<usize> = used.into_iter().collect();
    let mut number = vec![0u32; count];
    for (new, &old) in keep.iter().enumerate() {
      number[old] = new as u32;
    }
    let mut picked = HashMap::new();
    for &a in &key {
      if let std::collections::hash_map::Entry::Vacant(e) = picked.entry(a) {
        e.insert(glb.pick(a as usize, &keep)? as u64);
      }
    }
    for (&(m, p), old) in prims.iter().zip(&indices) {
      // the indices keep their size: the new numbers are no bigger than the old ones
      let old_accessor = prim(glb, (m, p))["indices"].as_u64().unwrap_or(0) as usize;
      let kind = glb.array("accessors")[old_accessor]["componentType"]
        .as_u64()
        .unwrap_or(5125);
      let renumbered: Vec<u32> = old.iter().map(|&i| number[i]).collect();
      let new = glb.push_indices(&renumbered, kind);
      let target = &mut glb.array_mut("meshes")[m]["primitives"][p];
      target["indices"] = json!(new);
      for (_, a) in target["attributes"].as_object_mut().into_iter().flatten() {
        if let Some(n) = a.as_u64().and_then(|i| picked.get(&i)) {
          *a = json!(n);
        }
      }
      for t in target["targets"].as_array_mut().into_iter().flatten() {
        for (_, a) in t.as_object_mut().into_iter().flatten() {
          if let Some(n) = a.as_u64().and_then(|i| picked.get(&i)) {
            *a = json!(n);
          }
        }
      }
    }
  }
  glb.compact();
  Ok(())
}
