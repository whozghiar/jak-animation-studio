//! A model's looks: the parts of its skinned meshes that can be shown or hidden, and its blend shapes
//! (morph targets). The games' extracted models hold every variant of a character at once: Jak's
//! armor pieces, his scarves, the shapes that make him Dark Jak. In game, `setup-masks` turns merc
//! effects on and off and `skel override` sets the blend weights; the extraction keeps the primitives
//! and the targets but not which effect a primitive belonged to, so a part here is a run of
//! primitives drawing the same vertices (an effect's piece, as the extraction lays them out).

use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use serde::Serialize;
use serde_json::{Value, json};

use crate::{glb::Glb, rules::Text};

#[derive(Serialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Part {
  pub mesh: usize,
  pub primitives: Vec<usize>,
  /// its materials' names (textures), which tell what it is
  pub materials: Vec<String>,
}

#[derive(Serialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Looks {
  pub parts: Vec<Part>,
  /// blend shapes of the skinned meshes, by index (the most any of them has)
  pub targets: usize,
}

/// The meshes the skins draw.
fn skinned_meshes(glb: &Glb) -> BTreeSet<usize> {
  glb
    .array("nodes")
    .iter()
    .filter(|n| n.get("skin").is_some())
    .filter_map(|n| n["mesh"].as_u64().map(|m| m as usize))
    .collect()
}

/// The first and last vertex a primitive draws.
fn vertex_range(glb: &Glb, primitive: &Value) -> Option<(f32, f32)> {
  let drawn = match primitive.get("indices") {
    Some(indices) => glb.numbers(indices).ok()?,
    None => {
      let count =
        glb.array("accessors")[primitive["attributes"]["POSITION"].as_u64()? as usize]["count"]
          .as_f64()? as f32;
      vec![0.0, count - 1.0]
    }
  };
  drawn.iter().fold(None, |r, &i| {
    Some(r.map_or((i, i), |(lo, hi): (f32, f32)| (lo.min(i), hi.max(i))))
  })
}

pub fn looks(glb: &Glb) -> Looks {
  let mut parts: Vec<Part> = Vec::new();
  let mut targets = 0;
  for mesh in skinned_meshes(glb) {
    let mut range: Option<(f32, f32)> = None;
    let primitives = glb.array("meshes")[mesh]["primitives"]
      .as_array()
      .cloned()
      .unwrap_or_default();
    for (i, primitive) in primitives.iter().enumerate() {
      targets = targets.max(primitive["targets"].as_array().map_or(0, Vec::len));
      let material = primitive["material"]
        .as_u64()
        .and_then(|m| glb.array("materials").get(m as usize))
        .and_then(|m| m["name"].as_str())
        .unwrap_or("")
        .to_owned();
      let drawn = vertex_range(glb, primitive);
      let overlaps = match (range, drawn) {
        (Some((lo, hi)), Some((a, b))) => a <= hi && b >= lo,
        _ => false,
      };
      match parts.last_mut() {
        Some(part) if overlaps && part.mesh == mesh => {
          part.primitives.push(i);
          if !part.materials.contains(&material) {
            part.materials.push(material);
          }
          let ((lo, hi), (a, b)) = (range.unwrap(), drawn.unwrap());
          range = Some((lo.min(a), hi.max(b)));
        }
        _ => {
          parts.push(Part {
            mesh,
            primitives: vec![i],
            materials: vec![material],
          });
          range = drawn;
        }
      }
    }
  }
  Looks { parts, targets }
}

/// A float accessor of positions or normals with the glTF bounds a POSITION needs.
fn push_vec3(glb: &mut Glb, values: &[f32]) -> usize {
  let accessor = glb.push_floats(values, "VEC3");
  let mut lo = [f32::MAX; 3];
  let mut hi = [f32::MIN; 3];
  for v in values.chunks(3) {
    for k in 0..3 {
      lo[k] = lo[k].min(v[k]);
      hi[k] = hi[k].max(v[k]);
    }
  }
  glb.array_mut("accessors")[accessor]["min"] = json!(lo);
  glb.array_mut("accessors")[accessor]["max"] = json!(hi);
  accessor
}

/// Makes the look the model's own: the `hidden` primitives (mesh, primitive) removed, and the blend
/// shapes baked into the vertices with `weights` (build-actor does not play morph targets), the
/// targets and their animation channels then removed.
pub fn apply(glb: &mut Glb, hidden: &[(usize, usize)], weights: &[f32]) -> Result<()> {
  for mesh in skinned_meshes(glb) {
    let mut primitives = glb.array("meshes")[mesh]["primitives"]
      .as_array()
      .cloned()
      .unwrap_or_default();
    primitives = primitives
      .into_iter()
      .enumerate()
      .filter(|(i, _)| !hidden.contains(&(mesh, *i)))
      .map(|(_, p)| p)
      .collect();
    ensure!(
      !primitives.is_empty(),
      Text::new(
        "every-part-hidden",
        "A model needs at least one part shown.",
        &[]
      )
    );

    // the targets add to the vertices: each POSITION and NORMAL accessor gets its baked copy once
    let mut baked: Vec<(u64, usize)> = Vec::new();
    for primitive in &mut primitives {
      let targets = primitive["targets"].as_array().cloned().unwrap_or_default();
      for attribute in ["POSITION", "NORMAL"] {
        let Some(base) = primitive["attributes"][attribute].as_u64() else {
          continue;
        };
        let used: Vec<(f32, &Value)> = targets
          .iter()
          .enumerate()
          .filter_map(|(t, target)| {
            let w = weights.get(t).copied().unwrap_or(0.0);
            (w != 0.0).then_some((w, target.get(attribute)?))
          })
          .collect();
        if used.is_empty() {
          continue;
        }
        let accessor = match baked.iter().find(|(b, _)| *b == base) {
          Some(&(_, done)) => done,
          None => {
            let mut values = glb.floats(&json!(base))?;
            for (w, target) in &used {
              let delta = glb
                .floats(target)
                .with_context(|| format!("a blend shape's {attribute} cannot be read"))?;
              for (v, d) in values.iter_mut().zip(delta) {
                *v += w * d;
              }
            }
            if attribute == "NORMAL" {
              for n in values.chunks_mut(3) {
                let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt().max(1e-12);
                n.iter_mut().for_each(|c| *c /= len);
              }
            }
            let done = push_vec3(glb, &values);
            baked.push((base, done));
            done
          }
        };
        primitive["attributes"][attribute] = json!(accessor);
      }
      if let Some(p) = primitive.as_object_mut() {
        p.remove("targets");
      }
    }
    let m = &mut glb.array_mut("meshes")[mesh];
    m["primitives"] = json!(primitives);
    if let Some(m) = m.as_object_mut() {
      m.remove("weights");
      if let Some(extras) = m.get_mut("extras").and_then(Value::as_object_mut) {
        extras.remove("targetNames");
      }
    }
  }

  // the weights channels drove targets that are gone
  for anim in glb.array_mut("animations") {
    let channels: Vec<Value> = anim["channels"]
      .as_array()
      .cloned()
      .unwrap_or_default()
      .into_iter()
      .filter(|c| c["target"]["path"] != "weights")
      .collect();
    let samplers = anim["samplers"].as_array().cloned().unwrap_or_default();
    let mut kept = Vec::new();
    let channels: Vec<Value> = channels
      .into_iter()
      .map(|mut c| {
        let s = c["sampler"].as_u64().unwrap_or(0) as usize;
        let at = kept.iter().position(|&k| k == s).unwrap_or_else(|| {
          kept.push(s);
          kept.len() - 1
        });
        c["sampler"] = json!(at);
        c
      })
      .collect();
    anim["samplers"] = json!(
      kept
        .iter()
        .map(|&s| samplers[s].clone())
        .collect::<Vec<_>>()
    );
    anim["channels"] = json!(channels);
  }
  glb
    .array_mut("animations")
    .retain(|a| a["channels"].as_array().is_some_and(|c| !c.is_empty()));
  if glb.array("animations").is_empty()
    && let Some(root) = glb.json.as_object_mut()
  {
    root.remove("animations");
  }
  glb.compact();
  Ok(())
}
