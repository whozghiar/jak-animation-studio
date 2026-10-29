//! What a model holds: its skeleton (the joints of its single skin, in the order build-actor numbers
//! them) and its animations.

use std::collections::{BTreeSet, HashMap};

use anyhow::{Context, Result, bail};
use serde::Serialize;
use serde_json::Value;

use crate::{glb::Glb, math, rules::Text};

pub const PATHS: [(&str, usize); 3] = [("translation", 3), ("rotation", 4), ("scale", 3)];

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Joint {
  pub node: usize,
  pub name: String,
  /// index of the parent joint
  pub parent: Option<usize>,
  pub translation: [f32; 3],
  pub rotation: [f32; 4],
  pub scale: [f32; 3],
  /// nodes under it that are not joints of the skin
  #[serde(skip)]
  pub foreign_children: Vec<usize>,
}

impl Joint {
  pub fn rest(&self, path: &str) -> &[f32] {
    match path {
      "translation" => &self.translation,
      "rotation" => &self.rotation,
      _ => &self.scale,
    }
  }
}

/// Each joint's rest rotation and position in the armature's space.
pub fn rest_world(joints: &[Joint]) -> Vec<(math::Quat, [f32; 3])> {
  let mut world: Vec<(math::Quat, [f32; 3], [f32; 3])> = Vec::with_capacity(joints.len());
  for j in joints {
    let entry = match j.parent.and_then(|p| world.get(p)) {
      Some(&(rot, pos, scale)) => {
        let local = [0, 1, 2].map(|i| j.translation[i] * scale[i]);
        let moved = math::rotate(rot, local);
        (
          math::mul(rot, j.rotation),
          [0, 1, 2].map(|i| pos[i] + moved[i]),
          [0, 1, 2].map(|i| scale[i] * j.scale[i]),
        )
      }
      None => (j.rotation, j.translation, j.scale),
    };
    world.push(entry);
  }
  world.into_iter().map(|(r, p, _)| (r, p)).collect()
}

/// The keys of one channel. Cubic spline keys keep only their values (their tangents dropped).
#[derive(Debug)]
pub struct Channel {
  pub node: usize,
  pub path: String,
  pub interpolation: String,
  pub keys: Result<(Vec<f32>, Vec<f32>)>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AnimationInfo {
  pub name: String,
  pub duration: f32,
  pub keyframes: usize,
}

/// The skins the nodes use: build-actor takes the model's armature from them.
pub fn skins(glb: &Glb) -> BTreeSet<usize> {
  glb
    .array("nodes")
    .iter()
    .filter_map(|n| n["skin"].as_u64().map(|s| s as usize))
    .collect()
}

fn node_name(glb: &Glb, node: usize) -> String {
  glb.array("nodes")[node]["name"]
    .as_str()
    .map_or_else(|| format!("node {node}"), str::to_owned)
}

fn floats<const N: usize>(v: &Value, default: [f32; N]) -> [f32; N] {
  let mut out = default;
  if let Some(a) = v.as_array().filter(|a| a.len() == N) {
    for (o, x) in out.iter_mut().zip(a) {
      *o = x.as_f64().unwrap_or(0.0) as f32;
    }
  }
  out
}

pub fn skeleton(glb: &Glb) -> Result<Vec<Joint>> {
  let skins = skins(glb);
  let skin = match skins.len() {
    0 => bail!(Text::new(
      "no-skin",
      "The model has no armature (skin): it is not an entity.",
      &[]
    )),
    1 => *skins.first().expect("one skin"),
    n => bail!(Text::new(
      "several-skins",
      "The model has {count} armatures (skins), build-actor takes one.",
      &[("count", n.to_string())]
    )),
  };
  let nodes = glb.array("nodes");
  let skin_joints: Vec<usize> = glb.array("skins")[skin]["joints"]
    .as_array()
    .context("the armature lists no joints")?
    .iter()
    .map(|j| j.as_u64().map(|j| j as usize).filter(|j| *j < nodes.len()))
    .collect::<Option<_>>()
    .context("the armature names a node that does not exist")?;
  let joint_of: HashMap<usize, usize> = skin_joints
    .iter()
    .enumerate()
    .map(|(i, n)| (*n, i))
    .collect();
  let mut parent_node = HashMap::new();
  for (i, n) in nodes.iter().enumerate() {
    for c in n["children"].as_array().into_iter().flatten() {
      if let Some(c) = c.as_u64() {
        parent_node.insert(c as usize, i);
      }
    }
  }
  Ok(
    skin_joints
      .iter()
      .map(|&node| {
        let n = &nodes[node];
        let (translation, rotation, scale) = match n["matrix"].as_array() {
          Some(m) if m.len() == 16 => {
            let m: Vec<f32> = m.iter().map(|x| x.as_f64().unwrap_or(0.0) as f32).collect();
            let len =
              |c: usize| (m[c * 4].powi(2) + m[c * 4 + 1].powi(2) + m[c * 4 + 2].powi(2)).sqrt();
            (
              [m[12], m[13], m[14]],
              math::from_matrix(&m),
              [len(0), len(1), len(2)],
            )
          }
          _ => (
            floats(&n["translation"], [0.0; 3]),
            floats(&n["rotation"], [0.0, 0.0, 0.0, 1.0]),
            floats(&n["scale"], [1.0; 3]),
          ),
        };
        Joint {
          node,
          name: node_name(glb, node),
          parent: parent_node
            .get(&node)
            .and_then(|p| joint_of.get(p))
            .copied(),
          translation,
          rotation,
          scale,
          foreign_children: n["children"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|c| c.as_u64().map(|c| c as usize))
            .filter(|c| !joint_of.contains_key(c))
            .collect(),
        }
      })
      .collect(),
  )
}

/// Keyframe times and values of a sampler, as build-actor reads them.
pub fn sampler_keys(glb: &Glb, sampler: &Value) -> Result<(Vec<f32>, Vec<f32>)> {
  let times = glb.floats(&sampler["input"])?;
  let mut values = glb.floats(&sampler["output"])?;
  if times.is_empty() || values.len() % times.len() != 0 {
    bail!("its keyframe times and values do not match");
  }
  if sampler["interpolation"].as_str() == Some("CUBICSPLINE") {
    let n = values.len() / times.len() / 3;
    values = values
      .chunks(n * 3)
      .flat_map(|c| c[n..n * 2].to_vec())
      .collect();
  }
  Ok((times, values))
}

pub fn channels(glb: &Glb, anim: &Value) -> Vec<Channel> {
  anim["channels"]
    .as_array()
    .into_iter()
    .flatten()
    .filter_map(|c| {
      let sampler = &anim["samplers"][c["sampler"].as_u64()? as usize];
      Some(Channel {
        node: c["target"]["node"].as_u64()? as usize,
        path: c["target"]["path"].as_str()?.to_owned(),
        interpolation: sampler["interpolation"]
          .as_str()
          .unwrap_or("LINEAR")
          .to_owned(),
        keys: sampler_keys(glb, sampler),
      })
    })
    .collect()
}

pub fn animation_name(anim: &Value, index: usize) -> String {
  anim["name"]
    .as_str()
    .map_or_else(|| format!("animation {index}"), str::to_owned)
}

pub fn animations(glb: &Glb) -> Vec<AnimationInfo> {
  glb
    .array("animations")
    .iter()
    .enumerate()
    .map(|(i, anim)| {
      let keys: Vec<_> = channels(glb, anim)
        .into_iter()
        .filter_map(|c| c.keys.ok())
        .collect();
      AnimationInfo {
        name: animation_name(anim, i),
        duration: keys
          .iter()
          .filter_map(|(t, _)| t.last().copied())
          .fold(0.0, f32::max),
        keyframes: keys.iter().map(|(t, _)| t.len()).max().unwrap_or(0),
      }
    })
    .collect()
}
