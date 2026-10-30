//! A binary glTF (.glb): its JSON and its BIN chunk. Edits touch only what they name: meshes,
//! materials, vertex colours and extensions are written back as they were read.

use std::path::Path;

use anyhow::{Context, Result, bail, ensure};
use serde_json::{Value, json};

use crate::rules::Text;

const MAGIC: u32 = 0x4654_6C67;
const CHUNK_JSON: u32 = 0x4E4F_534A;
const CHUNK_BIN: u32 = 0x004E_4942;
const FLOAT: u64 = 5126;

#[derive(Default)]
pub struct Glb {
  pub json: Value,
  pub bin: Vec<u8>,
  /// the file as it was read: written back as it is while nothing changed
  read: Vec<u8>,
}

fn components(ty: &str) -> Option<usize> {
  Some(match ty {
    "SCALAR" => 1,
    "VEC2" => 2,
    "VEC3" => 3,
    "VEC4" | "MAT2" => 4,
    "MAT3" => 9,
    "MAT4" => 16,
    _ => return None,
  })
}

fn align4(bin: &mut Vec<u8>) {
  while !bin.len().is_multiple_of(4) {
    bin.push(0);
  }
}

fn renumber(keep: &[bool]) -> Vec<Option<usize>> {
  let mut next = 0;
  keep
    .iter()
    .map(|&k| {
      k.then(|| {
        next += 1;
        next - 1
      })
    })
    .collect()
}

fn mark(used: &mut [bool], index: &Value) {
  if let Some(i) = index.as_u64()
    && let Some(u) = used.get_mut(i as usize)
  {
    *u = true;
  }
}

fn remap(index: &mut Value, map: &[Option<usize>]) {
  if let Some(i) = index.as_u64()
    && let Some(Some(n)) = map.get(i as usize)
  {
    *index = json!(n);
  }
}

/// `object[a][b]...` when it exists (indexing a `Value` mutably would add the missing keys).
fn at<'a>(object: &'a mut Value, keys: &[&str]) -> Option<&'a mut Value> {
  keys.iter().try_fold(object, |v, k| v.get_mut(*k))
}

fn items<'a>(object: &'a mut Value, key: &str) -> impl Iterator<Item = &'a mut Value> {
  object
    .get_mut(key)
    .and_then(Value::as_array_mut)
    .into_iter()
    .flatten()
}

/// Every place of the core glTF 2.0 schema that names an accessor.
fn accessor_refs(json: &mut Value) -> Vec<&mut Value> {
  let mut refs = Vec::new();
  let Value::Object(root) = json else {
    return refs;
  };
  for (key, value) in root.iter_mut() {
    let Some(items) = value.as_array_mut() else {
      continue;
    };
    match key.as_str() {
      "meshes" => {
        for mesh in items {
          for prim in self::items(mesh, "primitives") {
            let Value::Object(prim) = prim else { continue };
            for (k, v) in prim.iter_mut() {
              match k.as_str() {
                "attributes" => {
                  refs.extend(v.as_object_mut().into_iter().flatten().map(|(_, a)| a))
                }
                "indices" => refs.push(v),
                "targets" => {
                  for t in v.as_array_mut().into_iter().flatten() {
                    refs.extend(t.as_object_mut().into_iter().flatten().map(|(_, a)| a));
                  }
                }
                _ => {}
              }
            }
          }
        }
      }
      "skins" => {
        for skin in items {
          if let Some(ibm) = skin.get_mut("inverseBindMatrices") {
            refs.push(ibm);
          }
        }
      }
      "animations" => {
        for anim in items {
          for s in self::items(anim, "samplers") {
            let Value::Object(s) = s else { continue };
            refs.extend(
              s.iter_mut()
                .filter(|(k, _)| *k == "input" || *k == "output")
                .map(|(_, v)| v),
            );
          }
        }
      }
      _ => {}
    }
  }
  refs
}

impl Glb {
  pub fn read(path: &Path) -> Result<Glb> {
    let bytes =
      std::fs::read(path).with_context(|| format!("Unable to read {}", path.display()))?;
    Glb::parse(&bytes).with_context(|| {
      Text::new(
        "unreadable",
        "{file} is not a usable .glb",
        &[("file", path.display().to_string())],
      )
    })
  }

  /// Only the JSON chunk (the data left empty): enough to know a model's skeleton and animation
  /// names without reading its data, for a folder of large models.
  pub fn read_json(path: &Path) -> Result<Glb> {
    use std::io::Read;
    let mut file = std::fs::File::open(path)?;
    let mut head = [0u8; 20];
    file.read_exact(&mut head)?;
    let word = |at: usize| u32::from_le_bytes([head[at], head[at + 1], head[at + 2], head[at + 3]]);
    ensure!(
      word(0) == MAGIC && word(16) == CHUNK_JSON,
      "not a binary glTF file"
    );
    let mut json = vec![0u8; word(12) as usize];
    file.read_exact(&mut json)?;
    Ok(Glb {
      json: serde_json::from_slice(&json)?,
      ..Glb::default()
    })
  }

  pub fn parse(bytes: &[u8]) -> Result<Glb> {
    let word = |at: usize| -> Result<u32> {
      let b = bytes.get(at..at + 4).context("the file is cut short")?;
      Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    };
    if word(0)? != MAGIC {
      bail!("not a binary glTF file");
    }
    let (mut json, mut bin) = (None, Vec::new());
    let mut at = 12;
    while at + 8 <= bytes.len() {
      let (len, kind) = (word(at)? as usize, word(at + 4)?);
      let data = bytes
        .get(at + 8..at + 8 + len)
        .context("the file is cut short")?;
      match kind {
        CHUNK_JSON => json = Some(serde_json::from_slice::<Value>(data)?),
        CHUNK_BIN => bin = data.to_vec(),
        _ => {}
      }
      at += 8 + len;
    }
    let json = json.context("no JSON chunk")?;
    let buffers = json["buffers"].as_array().map(Vec::as_slice).unwrap_or(&[]);
    if buffers.len() > 1 || buffers.iter().any(|b| b.get("uri").is_some()) {
      bail!("its data is in separate files: export it as a single .glb");
    }
    Ok(Glb {
      json,
      bin,
      read: bytes.to_vec(),
    })
  }

  pub fn to_bytes(&self) -> Vec<u8> {
    if Glb::parse(&self.read).is_ok_and(|g| g.json == self.json && g.bin == self.bin) {
      return self.read.clone();
    }
    let mut json = self.json.clone();
    if self.bin.is_empty() {
      if let Some(root) = json.as_object_mut() {
        root.remove("buffers");
      }
    } else if json["buffers"].as_array().is_some_and(|b| !b.is_empty()) {
      json["buffers"][0]["byteLength"] = json!(self.bin.len());
    } else {
      json["buffers"] = json!([{ "byteLength": self.bin.len() }]);
    }
    let mut text = serde_json::to_vec(&json).expect("a JSON value always serializes");
    while !text.len().is_multiple_of(4) {
      text.push(b' ');
    }
    let mut bin = self.bin.clone();
    align4(&mut bin);
    let total = 20 + text.len() + if bin.is_empty() { 0 } else { 8 + bin.len() };
    let mut out = Vec::with_capacity(total);
    for word in [MAGIC, 2, total as u32, text.len() as u32, CHUNK_JSON] {
      out.extend(word.to_le_bytes());
    }
    out.extend(text);
    if !bin.is_empty() {
      out.extend((bin.len() as u32).to_le_bytes());
      out.extend(CHUNK_BIN.to_le_bytes());
      out.extend(bin);
    }
    out
  }

  /// Written next to the file, then renamed over it: a failed write never leaves half a model.
  pub fn write(&self, path: &Path) -> Result<()> {
    let tmp = path.with_extension("glb.tmp");
    std::fs::write(&tmp, self.to_bytes())
      .with_context(|| format!("Unable to write {}", tmp.display()))?;
    std::fs::rename(&tmp, path).with_context(|| format!("Unable to write {}", path.display()))
  }

  pub fn array(&self, key: &str) -> &[Value] {
    self.json[key].as_array().map(Vec::as_slice).unwrap_or(&[])
  }

  pub fn array_mut(&mut self, key: &str) -> &mut Vec<Value> {
    if !self.json[key].is_array() {
      self.json[key] = json!([]);
    }
    self.json[key].as_array_mut().expect("made an array above")
  }

  pub fn is_float(&self, accessor: &Value) -> bool {
    accessor
      .as_u64()
      .and_then(|i| self.array("accessors").get(i as usize))
      .is_some_and(|a| a["componentType"].as_u64() == Some(FLOAT) && a.get("sparse").is_none())
  }

  /// The numbers of a float accessor, element after element: what build-actor reads keys from.
  pub fn floats(&self, accessor: &Value) -> Result<Vec<f32>> {
    if !self.is_float(accessor) {
      bail!("accessor {accessor} is not made of plain 32-bit floats");
    }
    self.numbers(accessor)
  }

  /// The numbers of an accessor of floats or unsigned integers (normalized ones brought to 0..1).
  pub fn numbers(&self, accessor: &Value) -> Result<Vec<f32>> {
    let index = accessor.as_u64().context("no accessor")?;
    let a = self
      .array("accessors")
      .get(index as usize)
      .with_context(|| format!("accessor {index} does not exist"))?;
    if a.get("sparse").is_some() {
      bail!("accessor {index} is sparse");
    }
    let n = a["type"]
      .as_str()
      .and_then(components)
      .with_context(|| format!("accessor {index} has an unknown type"))?;
    let (size, max) = match a["componentType"].as_u64() {
      Some(FLOAT) => (4, 0.0),
      Some(5121) => (1, 255.0),
      Some(5123) => (2, 65535.0),
      Some(5125) => (4, 4294967295.0),
      _ => bail!("accessor {index} holds signed numbers"),
    };
    let scale = if a["normalized"].as_bool() == Some(true) {
      max
    } else {
      1.0
    };
    let count = a["count"].as_u64().unwrap_or(0) as usize;
    let Some(view) = a["bufferView"].as_u64() else {
      return Ok(vec![0.0; n * count]);
    };
    let v = self
      .array("bufferViews")
      .get(view as usize)
      .with_context(|| format!("buffer view {view} does not exist"))?;
    let start = v["byteOffset"].as_u64().unwrap_or(0) as usize
      + a["byteOffset"].as_u64().unwrap_or(0) as usize;
    let stride = v["byteStride"].as_u64().map_or(n * size, |s| s as usize);
    let mut out = Vec::with_capacity(n * count);
    for i in 0..count {
      for c in 0..n {
        let at = start + i * stride + c * size;
        let b = self
          .bin
          .get(at..at + size)
          .with_context(|| format!("accessor {index} reads past the end of the data"))?;
        out.push(
          match size {
            1 => b[0] as f32,
            2 => u16::from_le_bytes([b[0], b[1]]) as f32,
            _ if max == 0.0 => f32::from_le_bytes([b[0], b[1], b[2], b[3]]),
            _ => u32::from_le_bytes([b[0], b[1], b[2], b[3]]) as f32,
          } / scale,
        );
      }
    }
    Ok(out)
  }

  /// Appends `values` as a new float accessor of `ty` (SCALAR, VEC3...) and returns its index.
  pub fn push_floats(&mut self, values: &[f32], ty: &str) -> usize {
    let n = components(ty).expect("a glTF accessor type");
    align4(&mut self.bin);
    let offset = self.bin.len();
    for v in values {
      self.bin.extend(v.to_le_bytes());
    }
    let views = self.array_mut("bufferViews");
    views.push(json!({ "buffer": 0, "byteOffset": offset, "byteLength": values.len() * 4 }));
    let view = views.len() - 1;
    let mut accessor = json!({
      "bufferView": view, "componentType": FLOAT, "count": values.len() / n, "type": ty
    });
    if ty == "SCALAR" {
      // glTF requires the bounds of keyframe times
      let (lo, hi) = values
        .iter()
        .fold((f32::MAX, f32::MIN), |(lo, hi), v| (lo.min(*v), hi.max(*v)));
      accessor["min"] = json!([lo]);
      accessor["max"] = json!([hi]);
    }
    let accessors = self.array_mut("accessors");
    accessors.push(accessor);
    accessors.len() - 1
  }

  /// A new accessor holding the elements `keep` of accessor `index`, in that order: same type,
  /// component type and normalization; the bounds of a float accessor made again (glTF requires
  /// those of POSITION). Each element starts on 4 bytes, as glTF wants a vertex attribute's.
  pub fn pick(&mut self, index: usize, keep: &[usize]) -> Result<usize> {
    let a = self
      .array("accessors")
      .get(index)
      .with_context(|| format!("accessor {index} does not exist"))?
      .clone();
    if a.get("sparse").is_some() {
      bail!("accessor {index} is sparse");
    }
    let n = a["type"]
      .as_str()
      .and_then(components)
      .with_context(|| format!("accessor {index} has an unknown type"))?;
    let size = match a["componentType"].as_u64() {
      Some(5120 | 5121) => 1,
      Some(5122 | 5123) => 2,
      Some(5125 | FLOAT) => 4,
      _ => bail!("accessor {index} has an unknown component type"),
    };
    let view = a["bufferView"].as_u64().context("no buffer view")? as usize;
    let v = self
      .array("bufferViews")
      .get(view)
      .with_context(|| format!("buffer view {view} does not exist"))?;
    let start = v["byteOffset"].as_u64().unwrap_or(0) as usize
      + a["byteOffset"].as_u64().unwrap_or(0) as usize;
    let element = n * size;
    let stride = v["byteStride"].as_u64().map_or(element, |s| s as usize);
    let step = element.next_multiple_of(4);
    let mut data = Vec::with_capacity(keep.len() * step);
    for &k in keep {
      let at = start + k * stride;
      let bytes = self
        .bin
        .get(at..at + element)
        .with_context(|| format!("accessor {index} reads past the end of the data"))?;
      data.extend_from_slice(bytes);
      data.resize(data.len() + step - element, 0);
    }
    align4(&mut self.bin);
    let offset = self.bin.len();
    self.bin.extend(&data);
    let mut view = json!({ "buffer": 0, "byteOffset": offset, "byteLength": data.len() });
    if step != element {
      view["byteStride"] = json!(step);
    }
    let views = self.array_mut("bufferViews");
    views.push(view);
    let mut accessor = json!({
      "bufferView": views.len() - 1, "componentType": a["componentType"], "count": keep.len(),
      "type": a["type"]
    });
    if a["normalized"].as_bool() == Some(true) {
      accessor["normalized"] = json!(true);
    }
    let accessors = self.array_mut("accessors");
    accessors.push(accessor);
    let new = accessors.len() - 1;
    if a.get("min").is_some() && a["componentType"].as_u64() == Some(FLOAT) && !keep.is_empty() {
      let values = self.numbers(&json!(new))?;
      let bound = |pick: fn(f32, f32) -> f32| -> Vec<f32> {
        (0..n)
          .map(|c| {
            values
              .iter()
              .skip(c)
              .step_by(n)
              .copied()
              .reduce(pick)
              .unwrap_or(0.0)
          })
          .collect()
      };
      let (lo, hi) = (bound(f32::min), bound(f32::max));
      self.array_mut("accessors")[new]["min"] = json!(lo);
      self.array_mut("accessors")[new]["max"] = json!(hi);
    }
    Ok(new)
  }

  /// Appends `values` as a new accessor of indices of `component_type` (5121, 5123 or 5125).
  pub fn push_indices(&mut self, values: &[u32], component_type: u64) -> usize {
    align4(&mut self.bin);
    let offset = self.bin.len();
    for &v in values {
      match component_type {
        5121 => self.bin.push(v as u8),
        5123 => self.bin.extend((v as u16).to_le_bytes()),
        _ => self.bin.extend(v.to_le_bytes()),
      }
    }
    let length = self.bin.len() - offset;
    let views = self.array_mut("bufferViews");
    views.push(json!({ "buffer": 0, "byteOffset": offset, "byteLength": length }));
    let view = views.len() - 1;
    let accessors = self.array_mut("accessors");
    accessors.push(json!({
      "bufferView": view, "componentType": component_type, "count": values.len(), "type": "SCALAR"
    }));
    accessors.len() - 1
  }

  /// Drops the accessors and buffer views nothing uses any more (those of a replaced or deleted
  /// animation) and packs the data again. Left as is when an extension could point at data itself.
  pub fn compact(&mut self) {
    let known = |e: &str| {
      e.starts_with("KHR_materials_")
        || matches!(
          e,
          "KHR_texture_transform" | "KHR_lights_punctual" | "KHR_mesh_quantization"
        )
    };
    if self
      .array("extensionsUsed")
      .iter()
      .any(|e| !e.as_str().is_some_and(known))
    {
      return;
    }

    let mut json = self.json.clone();
    let mut used = vec![false; self.array("accessors").len()];
    for r in accessor_refs(&mut json) {
      mark(&mut used, r);
    }
    let accessor_map = renumber(&used);
    let mut accessors: Vec<Value> = self
      .array("accessors")
      .iter()
      .zip(&used)
      .filter(|(_, u)| **u)
      .map(|(a, _)| a.clone())
      .collect();

    let mut view_used = vec![false; self.array("bufferViews").len()];
    for a in &accessors {
      mark(&mut view_used, &a["bufferView"]);
      mark(&mut view_used, &a["sparse"]["indices"]["bufferView"]);
      mark(&mut view_used, &a["sparse"]["values"]["bufferView"]);
    }
    for image in self.array("images") {
      mark(&mut view_used, &image["bufferView"]);
    }
    let view_map = renumber(&view_used);

    let mut bin = Vec::new();
    let mut views = Vec::new();
    for (view, _) in self
      .array("bufferViews")
      .iter()
      .zip(&view_used)
      .filter(|(_, u)| **u)
    {
      let start = view["byteOffset"].as_u64().unwrap_or(0) as usize;
      let len = view["byteLength"].as_u64().unwrap_or(0) as usize;
      let Some(data) = self.bin.get(start..start + len) else {
        return;
      };
      align4(&mut bin);
      let mut view = view.clone();
      view["byteOffset"] = json!(bin.len());
      bin.extend_from_slice(data);
      views.push(view);
    }

    for r in accessor_refs(&mut json) {
      remap(r, &accessor_map);
    }
    for a in &mut accessors {
      for keys in [
        &["bufferView"][..],
        &["sparse", "indices", "bufferView"],
        &["sparse", "values", "bufferView"],
      ] {
        if let Some(view) = at(a, keys) {
          remap(view, &view_map);
        }
      }
    }
    for image in items(&mut json, "images") {
      if let Some(view) = image.get_mut("bufferView") {
        remap(view, &view_map);
      }
    }
    let root = json.as_object_mut().expect("a glTF document is an object");
    for (key, items) in [("accessors", accessors), ("bufferViews", views)] {
      if items.is_empty() {
        root.remove(key);
      } else {
        root.insert(key.into(), Value::Array(items));
      }
    }
    self.json = json;
    self.bin = bin;
  }
}
