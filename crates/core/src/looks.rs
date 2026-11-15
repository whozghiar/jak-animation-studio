//! A model's looks: the parts of its skinned meshes that can be shown or hidden, and its blend shapes
//! (morph targets). The games' extracted models hold every variant of a character at once: Jak's
//! armor pieces, his scarves, the shapes that make him Dark Jak. In game, `setup-masks` turns merc
//! effects on and off and `skel override` sets the blend weights; the extraction keeps the primitives
//! and the targets but not which effect a primitive belonged to, so a part here is a run of
//! primitives drawing the same vertices (an effect's piece, as the extraction lays them out), or,
//! when the art group can be read (`game_looks`), an effect, with the game's segments.

use std::collections::{BTreeSet, HashMap};
use std::path::Path;

use serde::Serialize;
use serde_json::Value;

use crate::glb::Glb;

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
  /// the game's segments, bit by bit of `setup-masks`: the parts each one draws (none in Jak 1, or
  /// when the art group cannot be read)
  pub segments: Vec<Vec<usize>>,
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
  Looks {
    parts,
    targets,
    segments: Vec::new(),
  }
}

/// The looks with the game's effects and segments when the model's art group can be read (Jak II
/// and 3). `setup-masks draw on off` (process-drawable.gc) keeps a mask of hidden segments; each
/// segment stands for the effects of its entry in the merc-ctrl's seg-table, and an effect a hidden
/// segment holds is not drawn (foreground.gc, disable-draw). The glb writes each effect's draws in
/// order (fr3_to_gltf.cpp, add_merc), so its primitives split into the effects by their triangle
/// counts (merc-effect tri-count): the crimson guard's 38 primitives into 8 effects, of 196, 28,
/// 242... triangles.
pub fn game_looks(glb: &Glb, model: &Path, game: &str, project: Option<&Path>) -> Looks {
  let mut found = looks(glb);
  let name = crate::library::model_name(model);
  let read = roots(model, project)
    .iter()
    .find_map(|root| merc_segments(&root.join("decompiler_out").join(game), game, &name));
  if let Some((tris, table)) = read
    && let Some(parts) = effects(glb, &tris)
  {
    found.segments = table
      .iter()
      .map(|&mask| (0..parts.len()).filter(|&e| mask >> e & 1 == 1).collect())
      .collect();
    found.parts = parts;
  }
  found
}

/// The single skinned mesh's primitives split into effects of `tris` triangles each.
fn effects(glb: &Glb, tris: &[u16]) -> Option<Vec<Part>> {
  let meshes: Vec<usize> = skinned_meshes(glb).into_iter().collect();
  let [mesh] = meshes[..] else {
    return None;
  };
  let primitives = glb.array("meshes")[mesh]["primitives"].as_array()?;
  let mut parts = Vec::new();
  let mut next = 0;
  for &want in tris {
    let mut part = Part {
      mesh,
      primitives: Vec::new(),
      materials: Vec::new(),
    };
    let mut have = 0;
    while have < usize::from(want) {
      let primitive = primitives.get(next)?;
      let indices = primitive["indices"].as_u64()? as usize;
      have += glb.array("accessors")[indices]["count"].as_u64()? as usize / 3;
      let material = primitive["material"]
        .as_u64()
        .and_then(|m| glb.array("materials").get(m as usize))
        .and_then(|m| m["name"].as_str())
        .unwrap_or("")
        .to_owned();
      if !part.materials.contains(&material) {
        part.materials.push(material);
      }
      part.primitives.push(next);
      next += 1;
    }
    (have == usize::from(want)).then_some(())?;
    parts.push(part);
  }
  (next == primitives.len()).then_some(parts)
}

/// The jak-project folders a model can be read from: those holding it, then the settings' one.
fn roots<'a>(model: &'a Path, project: Option<&'a Path>) -> Vec<&'a Path> {
  model
    .ancestors()
    .filter(|d| d.join("decompiler_out").is_dir())
    .chain(project)
    .collect()
}

/// The merc-ctrl named `name` in the raw art groups of `out` (`decompiler_out/<game>`): its
/// effects' triangle counts and its seg-table. Its art group is the model's (crimson-guard), else
/// whichever holds it (Jak 3's jakb-c-lod0 is in jak-ext-geo-c).
fn merc_segments(out: &Path, game: &str, name: &str) -> Option<(Vec<u16>, Vec<u64>)> {
  let raw = out.join("raw_obj");
  let mut group = crate::names::art_group(name);
  loop {
    if let Ok(data) = std::fs::read(raw.join(format!("{group}-ag.go")))
      && let Some(found) = merc_ctrl(&data, game, name)
    {
      return Some(found);
    }
    let Some((shorter, _)) = group.rsplit_once('-') else {
      break;
    };
    group = shorter;
  }
  let mut needle = (name.len() as u32).to_le_bytes().to_vec();
  needle.extend(name.as_bytes());
  needle.push(0);
  std::fs::read_dir(&raw)
    .ok()?
    .flatten()
    .filter(|e| e.file_name().to_string_lossy().ends_with("-ag.go"))
    .filter_map(|e| std::fs::read(e.path()).ok())
    .filter(|data| data.windows(needle.len()).any(|w| w == needle))
    .find_map(|data| merc_ctrl(&data, game, name))
}

/// A merc-ctrl of a raw art group (merc-h.gc), found by its header: the effect count at byte 24,
/// the fragment and triangle counts at 32 and 34, which its effects (128 bytes on, 32 bytes each,
/// frag-count at 18, tri-count at 22) add up to. Its name and seg-table are fields of the
/// art-element before it (Jak II: 24 and 8 bytes before the header, Jak 3: 40 and 20), pointers
/// from the start of the data (byte 16 of Jak II's files, 128 of Jak 3's); the seg-table is an
/// array of 64-bit effect masks, its length at the pointer, its data 12 bytes on.
fn merc_ctrl(data: &[u8], game: &str, name: &str) -> Option<(Vec<u16>, Vec<u64>)> {
  let (base, name_at, table_at) = match game {
    "jak2" => (0x10, 24, 8),
    "jak3" => (0x80, 40, 20),
    _ => return None,
  };
  let bytes = |at: usize, n: usize| data.get(at..at + n);
  let u16_at = |at| bytes(at, 2).map(|b| u16::from_le_bytes(b.try_into().expect("2 bytes")));
  let u32_at = |at| bytes(at, 4).map(|b| u32::from_le_bytes(b.try_into().expect("4 bytes")));
  let u64_at = |at| bytes(at, 8).map(|b| u64::from_le_bytes(b.try_into().expect("8 bytes")));
  (name_at..data.len().saturating_sub(160))
    .step_by(4)
    .find_map(|h| {
      let n = u32_at(h + 24)? as usize;
      if !(1..=64).contains(&n) {
        return None;
      }
      let effect = |i: usize| h + 128 + 32 * i;
      let tris: Vec<u16> = (0..n)
        .map(|i| u16_at(effect(i) + 22))
        .collect::<Option<_>>()?;
      let frags: u32 = (0..n)
        .map(|i| u16_at(effect(i) + 18).map(u32::from))
        .sum::<Option<_>>()?;
      let all: u32 = tris.iter().map(|&t| u32::from(t)).sum();
      if frags == 0 || frags != u32::from(u16_at(h + 32)?) || all != u32::from(u16_at(h + 34)?) {
        return None;
      }
      let string = base + u32_at(h - name_at)? as usize;
      if bytes(string + 4, u32_at(string)? as usize)? != name.as_bytes() {
        return None;
      }
      let table = match u32_at(h - table_at)? as usize {
        0 => Vec::new(),
        at => {
          let at = base + at;
          let count = (u32_at(at)? as usize).min(64);
          (0..count)
            .map(|i| u64_at(at + 12 + 8 * i))
            .collect::<Option<_>>()?
        }
      };
      Some((tris, table))
    })
}

/// The textures the game draws a model's eyes with, left then right: iris, pupil, lid.
#[derive(Serialize, Debug, PartialEq)]
pub struct Eyes {
  pub left: [String; 3],
  pub right: [String; 3],
}

/// The textures the game draws a model's eyes with. The extracted model only keeps stand-ins
/// (`programmer_eye_*`): the game draws the eyes apart (eye.gc), from its merc-ctrl's merc-eye-ctrl,
/// whose adgif shaders (3 in Jak 1: iris, pupil, lid; 6 in Jak II and 3: left, then right) each
/// hold a texture id at byte 24 (texture-h.gc). They are read from the decompiler's raw art group
/// (`decompiler_out/<game>/raw_obj/<art group>-ag.go`) and named with its texture table
/// (`decompiler/config/<game>/ntsc_v1/tex-info.min.json`). Ids of the character's own textures
/// are the art group's, which each level that loads it renames (its BSP's texture-remap-table,
/// pairs of 32-bit ids: original, then the level's): Kleever's iris (#x56a007) is `klever-eye`
/// once Jak 3's desliz renames it, the Mayor's `mayor-white-eye` through Jak 1's beach.
pub fn eye_textures(model: &Path, game: &str, project: Option<&Path>) -> Option<Eyes> {
  let name = crate::library::model_name(model);
  for root in roots(model, project) {
    let out = root.join("decompiler_out").join(game);
    let Ok(table) = std::fs::read(root.join(format!(
      "decompiler/config/{game}/ntsc_v1/tex-info.min.json"
    ))) else {
      continue;
    };
    let Ok(table) = serde_json::from_slice::<Vec<(u64, Value)>>(&table) else {
      continue;
    };
    let textures: HashMap<u64, String> = table
      .into_iter()
      .filter_map(|(id, t)| Some((id, t["name"].as_str()?.to_owned())))
      .collect();
    let named = |id: u32| {
      let key = (u64::from(id >> 20) << 16) | u64::from((id >> 8) & 0xfff);
      textures.get(&key).cloned()
    };
    // the art group: the model's (crimson-guard), else the one it is a skin of (jakb for jakb-c)
    let shaders = if game == "jak1" { 3 } else { 6 };
    let mut group = crate::names::art_group(&name);
    let ids = loop {
      if let Ok(data) = std::fs::read(out.join(format!("raw_obj/{group}-ag.go")))
        && let Some(ids) = eye_control(&data, shaders)
      {
        break ids;
      }
      group = group.rsplit_once('-')?.0;
    };
    // the levels' BSPs (Jak 1's <level>-vis.go, Jak II and 3's <DGO>.go), the model's own first
    let own = model
      .parent()
      .and_then(|d| d.file_name())
      .map(|d| d.to_string_lossy().into_owned());
    let mut levels: Vec<String> = own.into_iter().collect();
    for e in std::fs::read_dir(out.join("levels"))
      .into_iter()
      .flatten()
      .flatten()
    {
      let level = e.file_name().to_string_lossy().into_owned();
      if !levels.contains(&level) {
        levels.push(level);
      }
    }
    let mut remaps: Vec<Vec<u8>> = Vec::new();
    let mut level = levels.iter();
    let mut texture = |id: u32| -> Option<String> {
      if let Some(n) = named(id) {
        return Some(n);
      }
      let original = id & 0xffff_ff00;
      loop {
        if let Some(n) = remaps.iter().find_map(|bsp| renamed(bsp, original, &named)) {
          return Some(n);
        }
        let next = level.next()?;
        for file in [format!("{next}-vis.go"), format!("{next}.go")] {
          if let Ok(bsp) = std::fs::read(out.join("raw_obj").join(file)) {
            remaps.push(bsp);
          }
        }
      }
    };
    let names: Vec<String> = ids.iter().map(|&id| texture(id)).collect::<Option<_>>()?;
    let side = |k: usize| [names[k].clone(), names[k + 1].clone(), names[k + 2].clone()];
    let right = if shaders == 6 { 3 } else { 0 };
    return Some(Eyes {
      left: side(0),
      right: side(right),
    });
  }
  None
}

/// One eye at one frame, as eye.gc draws it: `x`, `y` (the iris and pupil moved by that much of
/// the eye), `lid` (below 0: open, the lid then at `1 + lid`), the sizes of the iris, the pupil and
/// the lid (times the eye's).
pub type EyeFrame = [f32; 6];

/// The eye animation of each animation of a model's art group, by name: per frame, the left eye then
/// the right. The game draws the eyes from it (eye.gc, merc-eye-anim), not from the model: Jak's
/// iris fills his eye (iris-scale 1.0), where eye.gc's defaults, without data, are 0.55. Each
/// art-joint-anim (art-h.gc) points to its merc-eye-anim-block (Jak 1 and II: 4 bytes into the
/// object, Jak 3: 28; its name 8 in), a max-frame (int16), then 8 bytes 8 on per eye and frame:
/// x, y, lid as signed bytes / 128, then iris, pupil and lid sizes as bytes / 64 at 4 to 6
/// (convert-eye-data).
pub fn eye_frames(
  model: &Path,
  game: &str,
  project: Option<&Path>,
) -> HashMap<String, Vec<[EyeFrame; 2]>> {
  let (base, eye_at) = match game {
    "jak3" => (0x80, 28),
    _ => (0x10, 4),
  };
  let name = crate::library::model_name(model);
  for root in roots(model, project) {
    let raw = root.join("decompiler_out").join(game).join("raw_obj");
    let mut group = crate::names::art_group(&name);
    loop {
      if let Ok(data) = std::fs::read(raw.join(format!("{group}-ag.go"))) {
        let found = eye_blocks(&data, base, eye_at);
        if !found.is_empty() {
          return found;
        }
      }
      let Some((shorter, _)) = group.rsplit_once('-') else {
        break;
      };
      group = shorter;
    }
  }
  HashMap::new()
}

fn eye_blocks(data: &[u8], base: usize, eye_at: usize) -> HashMap<String, Vec<[EyeFrame; 2]>> {
  let bytes = |at: usize, n: usize| data.get(at..at + n);
  let u32_at = |at| bytes(at, 4).map(|b| u32::from_le_bytes(b.try_into().expect("4 bytes")));
  let string = |at: usize| -> Option<String> {
    let text = bytes(at + 4, u32_at(at)? as usize)?;
    (!text.is_empty() && text.len() < 80 && text.iter().all(|c| c.is_ascii_graphic()))
      .then(|| String::from_utf8_lossy(text).into_owned())
  };
  let eye = |b: &[u8]| -> EyeFrame {
    let s = |i: usize| f32::from(b[i] as i8) / 128.0;
    let u = |i: usize| f32::from(b[i]) / 64.0;
    [s(0), s(1), s(2), u(4), u(5), u(6)]
  };
  let mut found = HashMap::new();
  for tag in (0..data.len().saturating_sub(32)).step_by(4) {
    if u32_at(tag) != Some(u32::MAX) {
      continue;
    }
    let (Some(name), Some(block)) = (
      u32_at(tag + 8).and_then(|p| string(base + p as usize)),
      u32_at(tag + eye_at).filter(|&p| p != 0),
    ) else {
      continue;
    };
    let block = base + block as usize;
    let Some(max) = bytes(block, 2)
      .map(|b| i16::from_le_bytes([b[0], b[1]]))
      .filter(|m| (0..2000).contains(m))
    else {
      continue;
    };
    let frames = (0..=max as usize)
      .map(|f| {
        let at = block + 8 + 16 * f;
        Some([eye(bytes(at, 8)?), eye(bytes(at + 8, 8)?)])
      })
      .collect::<Option<Vec<_>>>();
    if let Some(frames) = frames {
      found.entry(name).or_insert(frames);
    }
  }
  found
}

/// The first merc-eye-ctrl of an art group's data: its shader count (byte 2), then its adgif
/// shaders 16 bytes on, 80 bytes each, told by their GS registers (miptbp1 #x34 at byte 40, clamp
/// #x08 at byte 56); their texture ids.
fn eye_control(data: &[u8], shaders: usize) -> Option<Vec<u32>> {
  let word = |at: usize| u32::from_le_bytes(data[at..at + 4].try_into().expect("4 bytes"));
  let last = data.len().checked_sub(16 + shaders * 80)?;
  (0..=last).step_by(4).find_map(|at| {
    let shader = |k: usize| at + 16 + k * 80;
    (usize::from(data[at + 2]) == shaders
      && (0..shaders).all(|k| data[shader(k) + 40] == 0x34 && data[shader(k) + 56] == 0x08))
    .then(|| (0..shaders).map(|k| word(shader(k) + 24)).collect())
  })
}

/// The name a level's texture-remap-table gives the texture `original` (its id, low byte 0): the
/// entry is that id, then the level's, which the texture table names.
fn renamed(bsp: &[u8], original: u32, named: &impl Fn(u32) -> Option<String>) -> Option<String> {
  let wanted = original.to_le_bytes();
  (0..=bsp.len().checked_sub(8)?)
    .step_by(4)
    .filter(|&at| bsp[at..at + 4] == wanted)
    .find_map(|at| {
      named(u32::from_le_bytes(
        bsp[at + 4..at + 8].try_into().expect("4 bytes"),
      ))
    })
}
