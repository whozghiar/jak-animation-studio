use std::f32::consts::FRAC_1_SQRT_2;

use serde_json::json;

use crate::{
  edit::{self, Channel, JointPose, Pose},
  glb::Glb,
  library, math, model,
  rules::{self, Level},
};

/// A chain of `n` joints ("align", then "j1", "j2"... 10 cm above each other) skinning a triangle.
fn skinned(n: usize) -> Glb {
  let mut glb = Glb::default();
  glb.json = json!({ "asset": { "version": "2.0" } });
  let positions = glb.push_floats(&[0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0], "VEC3");
  let ibm: Vec<f32> = (0..n)
    .flat_map(|i| {
      let mut m = [0.0; 16];
      (m[0], m[5], m[10], m[15]) = (1.0, 1.0, 1.0, 1.0);
      m[13] = -0.1 * i as f32;
      m
    })
    .collect();
  let ibm = glb.push_floats(&ibm, "MAT4");
  let mut nodes: Vec<_> = (0..n)
    .map(|i| {
      let mut node = json!({
        "name": if i == 0 { "align".to_owned() } else { format!("j{i}") },
        "translation": [0.0, if i == 0 { 0.0 } else { 0.1 }, 0.0],
      });
      if i + 1 < n {
        node["children"] = json!([i + 1]);
      }
      node
    })
    .collect();
  nodes.push(json!({ "name": "body", "mesh": 0, "skin": 0 }));
  glb.json["nodes"] = json!(nodes);
  glb.json["meshes"] = json!([{ "primitives": [{ "attributes": { "POSITION": positions } }] }]);
  glb.json["skins"] = json!([{ "joints": (0..n).collect::<Vec<_>>(), "inverseBindMatrices": ibm }]);
  glb.json["scenes"] = json!([{ "nodes": [0, n] }]);
  glb
}

fn rotation(joint: usize, times: &[f32]) -> Channel {
  let values = times
    .iter()
    .flat_map(|t| math::from_euler_xyz_degrees([0.0, 0.0, 90.0 * t]))
    .collect();
  Channel {
    joint,
    path: "rotation".into(),
    times: times.to_vec(),
    values,
  }
}

fn errors(glb: &Glb, name: &str) -> Vec<&'static str> {
  rules::check(glb, name)
    .into_iter()
    .filter(|i| i.level == Level::Error)
    .map(|i| i.text.code)
    .collect()
}

#[test]
fn reads_back_what_it_writes() {
  let glb = Glb::parse(&skinned(4).to_bytes()).unwrap();
  let joints = model::skeleton(&glb).unwrap();
  let names: Vec<_> = joints.iter().map(|j| j.name.as_str()).collect();
  assert_eq!(names, ["align", "j1", "j2", "j3"]);
  assert_eq!(joints[3].parent, Some(2));
  assert_eq!(joints[0].parent, None);
  assert_eq!(glb.floats(&json!(0)).unwrap()[3], 1.0);
}

#[test]
fn every_joint_is_keyed_and_constant_keys_are_stored_once() {
  let mut glb = skinned(4);
  edit::set_animation(&mut glb, "wave", &[rotation(2, &[0.0, 0.5, 1.0])]).unwrap();
  let anim = &glb.array("animations")[0];
  let channels = model::channels(&glb, anim);
  assert_eq!(channels.len(), 4 * 3);
  for c in &channels {
    let (times, _) = c.keys.as_ref().unwrap();
    let moving = c.node == 2 && c.path == "rotation";
    assert_eq!(times.len(), if moving { 3 } else { 1 }, "{c:?}");
  }
  assert!(errors(&glb, "chain").is_empty());
  assert_eq!(model::animations(&glb)[0].duration, 1.0);
}

#[test]
fn replacing_keeps_the_slot_and_the_size() {
  let mut glb = skinned(4);
  edit::set_animation(&mut glb, "a", &[rotation(1, &[0.0, 1.0])]).unwrap();
  edit::set_animation(&mut glb, "b", &[rotation(2, &[0.0, 1.0])]).unwrap();
  let size = glb.bin.len();
  edit::set_animation(&mut glb, "a", &[rotation(3, &[0.0, 1.0])]).unwrap();
  let names: Vec<_> = model::animations(&glb)
    .into_iter()
    .map(|a| a.name)
    .collect();
  assert_eq!(names, ["a", "b"]);
  assert_eq!(glb.bin.len(), size);
  edit::delete_animation(&mut glb, "a").unwrap();
  edit::delete_animation(&mut glb, "b").unwrap();
  // what is left: the triangle and the inverse bind matrices
  assert_eq!(glb.bin.len(), 9 * 4 + 4 * 16 * 4);
  assert_eq!(glb.floats(&json!(0)).unwrap()[3], 1.0);
  assert!(glb.json.get("animations").is_none());
}

#[test]
fn a_foreign_animation_is_flagged_then_fixed_by_normalize() {
  let mut glb = skinned(4);
  let times = glb.push_floats(&[0.0, 1.0], "SCALAR");
  let rot = glb.push_floats(
    &[0.0, 0.0, 0.0, 1.0, 0.0, 0.0, FRAC_1_SQRT_2, FRAC_1_SQRT_2],
    "VEC4",
  );
  let weights = glb.push_floats(&[0.0, 1.0], "SCALAR");
  let moves = glb.push_floats(&[0.0, 0.0, 0.0, 1.0, 0.0, 0.0], "VEC3");
  glb.json["animations"] = json!([{
    "name": "Wave",
    "samplers": [
      { "input": times, "output": rot, "interpolation": "STEP" },
      { "input": times, "output": weights },
      { "input": times, "output": moves }
    ],
    "channels": [
      { "sampler": 0, "target": { "node": 2, "path": "rotation" } },
      { "sampler": 1, "target": { "node": 4, "path": "weights" } },
      { "sampler": 2, "target": { "node": 4, "path": "translation" } }
    ]
  }]);
  let found = errors(&glb, "chain");
  for code in ["missing-keys", "weights", "not-a-joint"] {
    assert!(found.contains(&code), "{code} in {found:?}");
  }
  let notes = edit::normalize(&mut glb, "chain").unwrap();
  for code in [
    "made-linear",
    "weights-removed",
    "node-dropped",
    "rest-keyed",
  ] {
    assert!(notes.iter().any(|n| n.code == code), "{code} in {notes:?}");
  }
  assert!(errors(&glb, "chain").is_empty());
  assert_eq!(model::animations(&glb)[0].name, "Wave", "names are kept");
}

#[test]
fn animation_names_lose_their_model_prefix() {
  assert_eq!(rules::short_name("babak-idle", "babak-lod0"), "idle");
  assert_eq!(rules::short_name("babak-lod0-walk", "babak-lod0"), "walk");
  assert_eq!(rules::short_name("wave", "babak-lod0"), "wave");
  assert_eq!(rules::short_name("crab-idle", "my-crab"), "crab-idle");
}

#[test]
fn the_game_limits_are_checked() {
  assert!(errors(&skinned(rules::MAX_JOINTS + 1), "big").contains(&"too-many-joints"));
  assert!(errors(&skinned(rules::MAX_JOINTS), "big").is_empty());

  // 110 joints moving in translation, rotation and scale: 2 x 64 + 108 x 20 bytes a frame
  let mut glb = skinned(110);
  let channels: Vec<Channel> = (0..110)
    .flat_map(|j| {
      let moving = |path: &str, a: f32, b: f32| Channel {
        joint: j,
        path: path.into(),
        times: vec![0.0, 1.0],
        values: vec![a, a, a, b, b, b],
      };
      [
        moving("translation", 0.0, 0.1),
        moving("scale", 1.0, 1.1),
        rotation(j, &[0.0, 1.0]),
      ]
    })
    .collect();
  edit::set_animation(&mut glb, "everything", &channels).unwrap();
  assert!(errors(&glb, "big").contains(&"frame-too-big"));

  // 98 joints turning for a minute: 3600 frames of 784 bytes, more than a level of any game loads
  let mut glb = skinned(100);
  let turning =
    |seconds: f32| -> Vec<Channel> { (2..100).map(|j| rotation(j, &[0.0, seconds])).collect() };
  edit::set_animation(&mut glb, "long", &turning(60.0)).unwrap();
  assert!(errors(&glb, "big").contains(&"animations-too-big"));
  // 40 s: 1.9 MB, too big for Jak 2 and Jak 3 only
  edit::set_animation(&mut glb, "long", &turning(40.0)).unwrap();
  assert!(errors(&glb, "big").is_empty());
  assert!(
    rules::check(&glb, "big")
      .iter()
      .any(|i| i.text.code == "animations-too-big-23")
  );
}

#[test]
fn poses_are_relative_to_the_rest_pose() {
  let mut glb = skinned(3);
  let joints = model::skeleton(&glb).unwrap();
  let poses: Vec<Pose> = serde_json::from_value(json!([
    { "time": 1.0, "joints": { "j1": { "rotate": [0, 0, 90], "translate": [0, 0.5, 0] } } },
    { "time": 0.0, "joints": { "j1": {} } }
  ]))
  .unwrap();
  let channels = edit::channels_from_poses(&joints, &poses).unwrap();
  let t = channels.iter().find(|c| c.path == "translation").unwrap();
  assert_eq!(t.times, [1.0]);
  assert!((t.values[1] - 0.6).abs() < 1e-6);
  edit::set_animation(&mut glb, "turn", &channels).unwrap();
  let bad = vec![Pose {
    time: 0.0,
    joints: [("nope".to_owned(), JointPose::default())].into(),
  }];
  assert!(edit::channels_from_poses(&joints, &bad).is_err());
}

#[test]
fn a_creation_is_a_copy_and_exports_for_the_game() {
  let root = std::env::temp_dir().join(format!("ogas-test-{}", std::process::id()));
  let _ = std::fs::remove_dir_all(&root);
  let (models, creations) = (root.join("models"), root.join("creations"));
  std::fs::create_dir_all(&models).unwrap();
  let source = models.join("My Crab.glb");
  skinned(4).write(&source).unwrap();
  let mut rock = Glb::default();
  rock.json = json!({ "asset": { "version": "2.0" } });
  rock.write(&models.join("rock.glb")).unwrap();

  let listed = library::list_models(&models, "jak2", true);
  assert_eq!(listed.len(), 1, "only the entity is listed");
  let creation = library::edit_creation(&source, &creations, "jak2", |glb| {
    edit::set_animation(glb, "idle", &[rotation(1, &[0.0, 1.0])])
  })
  .unwrap();
  assert_eq!(
    creation,
    creations.join("jak2").join("my-crab").join("my-crab.glb")
  );
  assert!(
    Glb::read(&source).unwrap().array("animations").is_empty(),
    "the original is untouched"
  );

  let export = library::export(&creation, "jak2", None, "en").unwrap();
  assert!(export.notes.is_empty(), "{:?}", export.notes);
  let names = |g: &Glb| {
    model::skeleton(g)
      .unwrap()
      .into_iter()
      .map(|j| j.name)
      .collect::<Vec<_>>()
  };
  assert_eq!(
    names(&Glb::read(&export.glb).unwrap()),
    names(&Glb::read(&creation).unwrap())[1..],
    "only the skin changes: align leaves it, for build-actor"
  );
  assert!(
    export
      .glb
      .ends_with("export/custom_assets/jak2/models/custom_levels/my-crab.glb")
  );
  assert!(
    export.issues.iter().all(|i| i.level != Level::Error),
    "{:?}",
    export.issues
  );
  let readme = std::fs::read_to_string(&export.readme).unwrap();
  assert!(readme.contains("(build-actor \"my-crab\")") && readme.contains(":art (idle-ja)"));
  assert_eq!(
    library::list_models(&creations, "jak2", true).len(),
    1,
    "exports are not listed"
  );

  let rock = library::import(&models.join("rock.glb"), &creations, "jak2", false).unwrap_err();
  assert_eq!(
    rock.downcast_ref::<rules::Text>().map(|t| t.code),
    Some("not-an-entity"),
    "the interface says it in its language"
  );
  let readme = library::export(&creation, "jak2", None, "fr")
    .unwrap()
    .readme;
  assert!(
    std::fs::read_to_string(readme)
      .unwrap()
      .contains("Construisez son art group")
  );
  assert!(library::import(&source, &creations, "jak2", false).is_err());
  library::import(&source, &creations, "jak2", true).unwrap();
  assert!(
    Glb::read(&creation).unwrap().array("animations").is_empty(),
    "replaced by the imported model"
  );
  assert!(library::import(&source, &creations, "jak4", false).is_err());

  let request = library::PlayRequest {
    model: creation,
    animation: Some("idle".into()),
    game: "jak2".into(),
    at: library::PlayRequest::now(),
  };
  request.send(&root).unwrap();
  assert_eq!(library::PlayRequest::take(&root), Some(request));
  assert_eq!(library::PlayRequest::take(&root), None, "played once");
  std::fs::remove_dir_all(&root).unwrap();
}

#[test]
fn a_turn_is_about_the_armature_axes() {
  // j1 lies turned 90 degrees about X at rest: its own axes are not the armature's
  let mut glb = skinned(3);
  glb.json["nodes"][1]["rotation"] = json!(math::from_euler_xyz_degrees([90.0, 0.0, 0.0]));
  let joints = model::skeleton(&glb).unwrap();
  let poses: Vec<Pose> =
    serde_json::from_value(json!([{ "time": 0.0, "joints": { "j2": { "turn": [0, 0, 90] } } }]))
      .unwrap();
  let channels = edit::channels_from_poses(&joints, &poses).unwrap();
  let local: [f32; 4] = channels[0].values[..4].try_into().unwrap();
  let world = model::rest_world(&joints);
  let turned = math::rotate(math::mul(world[1].0, local), [0.0, 1.0, 0.0]);
  let expected = math::rotate(
    math::from_euler_xyz_degrees([0.0, 0.0, 90.0]),
    math::rotate(world[2].0, [0.0, 1.0, 0.0]),
  );
  for i in 0..3 {
    assert!(
      (turned[i] - expected[i]).abs() < 1e-5,
      "{turned:?} {expected:?}"
    );
  }
}

#[test]
fn entities_have_their_official_names() {
  let flutflut = crate::names::title("jak1", "flutflut-lod0").unwrap();
  assert_eq!(
    (flutflut.en.as_str(), flutflut.fr.as_deref()),
    ("Flut Flut", Some("Flut-Flut"))
  );
  assert!(crate::names::title("jak1", "my-crab").is_none());
  assert!(
    crate::names::title("jak2", "flutflut-lod0").is_none(),
    "names are by game"
  );
}

#[test]
fn objects_are_told_from_living_entities() {
  use crate::names::is_object;
  for living in ["babak-lod0", "eichar-lod0", "yakow-lod0", "flutflut-lod0"] {
    assert!(!is_object("jak1", living), "{living}");
  }
  for object in [
    "barrel-lod0",
    "fuel-cell-lod0",
    "ecovalve-geo",
    "plant-boss-arms-lod0",
    "robotboss-basic-lod0",
  ] {
    assert!(is_object("jak1", object), "{object}");
  }
  assert!(
    !is_object("jak1", "my-crab"),
    "a model of your own is shown"
  );
  assert!(is_object("jak2", "atoll-hatch-lod0") && !is_object("jak2", "fodder-lod0"));
  assert!(
    is_object("jak2", "roboguard-lod0") && is_object("jak3", "kg-grunt-lod0"),
    "robots"
  );
}

/// A u32 index accessor.
fn push_indices(glb: &mut Glb, indices: &[u32]) -> usize {
  while !glb.bin.len().is_multiple_of(4) {
    glb.bin.push(0);
  }
  let offset = glb.bin.len();
  for i in indices {
    glb.bin.extend(i.to_le_bytes());
  }
  glb
    .array_mut("bufferViews")
    .push(json!({ "buffer": 0, "byteOffset": offset, "byteLength": indices.len() * 4 }));
  let view = glb.array("bufferViews").len() - 1;
  glb.array_mut("accessors").push(
    json!({ "bufferView": view, "componentType": 5125, "count": indices.len(), "type": "SCALAR" }),
  );
  glb.array("accessors").len() - 1
}

#[test]
fn a_look_hides_parts_and_bakes_blend_shapes() {
  let mut glb = skinned(3);
  // two pieces, each drawing its own three vertices; one blend shape lifting the first vertex
  let mut corners = Vec::new();
  for z in [0.0, 1.0] {
    corners.extend([0.0, 0.0, z, 1.0, 0.0, z, 0.0, 1.0, z]);
  }
  let positions = glb.push_floats(&corners, "VEC3");
  let mut lift = vec![0.0; 18];
  lift[1] = 1.0;
  let lift = glb.push_floats(&lift, "VEC3");
  let (a, b) = (
    push_indices(&mut glb, &[0, 1, 2]),
    push_indices(&mut glb, &[3, 4, 5]),
  );
  let piece = |indices| json!({ "attributes": { "POSITION": positions }, "indices": indices, "targets": [{ "POSITION": lift }] });
  glb.json["meshes"] = json!([{ "primitives": [piece(a), piece(b)], "weights": [0.0] }]);

  let looks = crate::looks::looks(&glb);
  assert_eq!(looks.targets, 1);
  assert_eq!(
    looks
      .parts
      .iter()
      .map(|p| p.primitives.clone())
      .collect::<Vec<_>>(),
    [vec![0], vec![1]]
  );

  crate::looks::apply(&mut glb, &[(0, 1)], &[1.0]).unwrap();
  let mesh = &glb.array("meshes")[0];
  assert_eq!(mesh["primitives"].as_array().unwrap().len(), 1);
  assert!(mesh["primitives"][0].get("targets").is_none() && mesh.get("weights").is_none());
  let baked = glb
    .floats(&mesh["primitives"][0]["attributes"]["POSITION"])
    .unwrap();
  assert_eq!(baked[1], 1.0, "the shape is in the vertices");
  assert!(
    crate::looks::apply(&mut glb, &[(0, 0)], &[]).is_err(),
    "a model keeps a part"
  );
}

#[test]
fn a_twin_skeleton_completes_the_animations() {
  let root = std::env::temp_dir().join(format!("ogas-twins-{}", std::process::id()));
  let _ = std::fs::remove_dir_all(&root);
  std::fs::create_dir_all(root.join("level")).unwrap();
  let mesh = root.join("jak-lod0.glb");
  skinned(4).write(&mesh).unwrap();
  let mut anims = skinned(4);
  edit::set_animation(&mut anims, "run", &[rotation(2, &[0.0, 1.0])]).unwrap();
  anims
    .write(&root.join("level").join("jak-anims-lod0.glb"))
    .unwrap();
  skinned(5).write(&root.join("other-lod0.glb")).unwrap();
  let mut stranger = skinned(4);
  edit::set_animation(&mut stranger, "idle", &[rotation(2, &[0.0, 1.0])]).unwrap();
  stranger.write(&root.join("stranger-lod0.glb")).unwrap();

  let twins = library::twins(&mesh, std::slice::from_ref(&root));
  assert_eq!(
    twins.len(),
    1,
    "neither the other skeleton nor another entity's same one is a twin"
  );
  assert_eq!(twins[0].animations, ["run"]);

  let mut glb = Glb::read(&mesh).unwrap();
  let added = edit::copy_animations(&mut glb, &Glb::read(&twins[0].path).unwrap()).unwrap();
  assert_eq!(added, 1);
  assert_eq!(model::animations(&glb)[0].name, "run");
  assert_eq!(
    edit::copy_animations(&mut glb, &Glb::read(&twins[0].path).unwrap()).unwrap(),
    0,
    "an animation it has is not added again"
  );
  std::fs::remove_dir_all(&root).unwrap();
}

/// A .glb of `json`, written as is, and `bin`.
fn glb_bytes(json: &str, bin: &[u8]) -> Vec<u8> {
  let mut json = json.as_bytes().to_vec();
  while !json.len().is_multiple_of(4) {
    json.push(b' ');
  }
  let total = 12 + 8 + json.len() + 8 + bin.len();
  let mut out = Vec::new();
  for word in [0x4654_6C67, 2, total as u32, json.len() as u32, 0x4E4F_534A] {
    out.extend(u32::to_le_bytes(word));
  }
  out.extend(json);
  out.extend((bin.len() as u32).to_le_bytes());
  out.extend(0x004E_4942u32.to_le_bytes());
  out.extend(bin);
  out
}

/// The triangle's points following `numbers` (4 joint numbers per point, as bytes).
fn with_joint_numbers(mut glb: Glb, numbers: &[u8]) -> Glb {
  let offset = glb.bin.len();
  glb.bin.extend(numbers);
  let views = glb.array_mut("bufferViews");
  views.push(json!({ "buffer": 0, "byteOffset": offset, "byteLength": numbers.len() }));
  let view = views.len() - 1;
  let accessors = glb.array_mut("accessors");
  accessors.push(json!({
    "bufferView": view, "componentType": 5121, "count": numbers.len() / 4, "type": "VEC4"
  }));
  glb.json["meshes"][0]["primitives"][0]["attributes"]["JOINTS_0"] = json!(accessors.len() - 1);
  glb
}

fn joint_numbers(glb: &Glb) -> Vec<f32> {
  glb
    .numbers(&glb.json["meshes"][0]["primitives"][0]["attributes"]["JOINTS_0"])
    .unwrap()
}

/// Where each joint of the skin is at time `t` of the first animation, by name.
fn posed(glb: &Glb, t: f32) -> std::collections::HashMap<String, [f32; 3]> {
  let joints = model::skeleton(glb).unwrap();
  let channels = model::channels(glb, &glb.array("animations")[0]);
  let mut world: Vec<(math::Quat, [f32; 3])> = Vec::new();
  for j in &joints {
    let key = |path: &str| {
      channels
        .iter()
        .find(|c| c.node == j.node && c.path == path)
        .map_or_else(
          || j.rest(path).to_vec(),
          |c| {
            let (times, values) = c.keys.as_ref().unwrap();
            edit::sample(times, values, t)
          },
        )
    };
    let (tr, ro) = (key("translation"), key("rotation"));
    let local = ([ro[0], ro[1], ro[2], ro[3]], [tr[0], tr[1], tr[2]]);
    world.push(match j.parent {
      Some(p) => {
        let (pr, pt) = world[p];
        let moved = math::rotate(pr, local.1);
        (math::mul(pr, local.0), [0, 1, 2].map(|k| pt[k] + moved[k]))
      }
      None => local,
    });
  }
  joints
    .into_iter()
    .zip(world)
    .map(|(j, (_, at))| (j.name, at))
    .collect()
}

/// build-actor puts its own align in front of the skeleton, and the mesh counts the points' joints
/// one off: the skin is set so that every build-actor builds the model right.
#[test]
fn the_skin_is_set_for_any_build_actor() {
  // align travels 1 m forward while j1 turns: once align has left the skin, j1 carries its travel
  let numbers = [1, 0, 0, 0, 2, 1, 0, 0, 2, 0, 0, 0];
  let mut glb = with_joint_numbers(skinned(3), &numbers);
  let travel = Channel {
    joint: 0,
    path: "translation".into(),
    times: vec![0.0, 1.0],
    values: vec![0.0, 0.0, 0.0, 0.0, 0.0, 1.0],
  };
  edit::set_animation(&mut glb, "walk", &[travel, rotation(1, &[0.0, 1.0])]).unwrap();
  let before = Glb::parse(&glb.to_bytes()).unwrap();
  edit::for_build_actor(&mut glb).unwrap();
  let glb = Glb::parse(&glb.to_bytes()).unwrap();

  let names: Vec<_> = model::skeleton(&glb)
    .unwrap()
    .into_iter()
    .map(|j| j.name)
    .collect();
  assert_eq!(names, ["j1", "j2"], "every build-actor adds its own align");
  let skin = &glb.json["skins"][0];
  assert_eq!(
    glb.floats(&skin["inverseBindMatrices"]).unwrap().len(),
    2 * 16
  );
  assert_eq!(
    joint_numbers(&glb),
    numbers.map(f32::from),
    "the points keep their numbers: in the game, joint n is the skin's joint n - 1"
  );
  assert!(
    model::channels(&glb, &glb.array("animations")[0])
      .iter()
      .all(|c| c.node != 0),
    "build-actor stops on keys of a node outside the skin"
  );
  for t in [0.0, 0.25, 0.5, 1.0] {
    let (was, is) = (posed(&before, t), posed(&glb, t));
    for joint in ["j1", "j2"] {
      let d = (0..3)
        .map(|k| (was[joint][k] - is[joint][k]).abs())
        .fold(0.0, f32::max);
      assert!(
        d < 1e-5,
        "{joint} at {t} s: {:?} became {:?}",
        was[joint],
        is[joint]
      );
    }
  }

  // a skin without align: build-actor adds one, the points' numbers go up by one
  let mut glb = with_joint_numbers(skinned(3), &numbers);
  glb.json["nodes"][0]["name"] = json!("root");
  edit::for_build_actor(&mut glb).unwrap();
  assert_eq!(model::skeleton(&glb).unwrap().len(), 3);
  assert_eq!(joint_numbers(&glb), numbers.map(|n| f32::from(n + 1)));
}

#[test]
fn a_model_left_as_it_was_is_written_back_byte_for_byte() {
  let model = skinned(3);
  // laid out as serde_json never would: keys out of order, spaces, numbers as another tool wrote them
  let text = serde_json::to_string_pretty(&model.json).unwrap().replacen(
    '{',
    r#"{ "extras": { "tiny": 1E-5, "one": 1.0 },"#,
    1,
  );
  let bytes = glb_bytes(&text, &model.bin);
  let mut glb = Glb::parse(&bytes).unwrap();
  assert_eq!(glb.to_bytes(), bytes);
  assert!(edit::normalize(&mut glb, "chain").unwrap().is_empty());
  assert_eq!(glb.to_bytes(), bytes, "nothing to fix: nothing touched");
}

#[test]
fn an_added_animation_leaves_the_rest_as_it_was() {
  let mut glb = skinned(4);
  edit::set_animation(&mut glb, "a", &[rotation(1, &[0.0, 1.0])]).unwrap();
  let before = Glb::parse(&glb.to_bytes()).unwrap();
  let mut twin = skinned(4);
  edit::set_animation(&mut twin, "c", &[rotation(2, &[0.0, 0.5, 1.0])]).unwrap();

  edit::set_animation(&mut glb, "b", &[rotation(3, &[0.0, 1.0])]).unwrap();
  assert_eq!(edit::copy_animations(&mut glb, &twin).unwrap(), 1);
  let names: Vec<_> = model::animations(&glb)
    .into_iter()
    .map(|a| a.name)
    .collect();
  assert_eq!(
    names,
    ["a", "b", "c"],
    "added after the others: their slots stay"
  );
  for key in ["asset", "nodes", "meshes", "skins", "scene", "scenes"] {
    assert_eq!(glb.json[key], before.json[key], "{key}");
  }
  let keys = |g: &Glb| {
    let anim = &g.array("animations")[0];
    model::channels(g, anim)
      .into_iter()
      .map(|c| (c.node, c.path, c.keys.unwrap()))
      .collect::<Vec<_>>()
  };
  assert_eq!(keys(&glb), keys(&before));
  let mesh = |g: &Glb| {
    g.numbers(&g.json["meshes"][0]["primitives"][0]["attributes"]["POSITION"])
      .unwrap()
  };
  assert_eq!(mesh(&glb), mesh(&before));
}

/// The extraction's own models, as the studio meets them: `OGAS_MODELS=<folder> cargo test -p
/// animation-studio-core -- --ignored --nocapture` (decompiler_out/<game>/levels, say).
#[test]
#[ignore = "needs the models of a game: OGAS_MODELS=<folder>"]
fn the_games_models_survive_the_studio() {
  let dir = std::path::PathBuf::from(std::env::var("OGAS_MODELS").expect("OGAS_MODELS"));
  let mut stack = vec![dir];
  let (mut models, mut fixed) = (0, std::collections::BTreeMap::<String, usize>::new());
  while let Some(d) = stack.pop() {
    for e in std::fs::read_dir(&d).unwrap().flatten() {
      let path = e.path();
      if path.is_dir() {
        stack.push(path);
        continue;
      }
      if path.extension().is_none_or(|x| x != "glb") {
        continue;
      }
      let bytes = std::fs::read(&path).unwrap();
      let Ok(mut glb) = Glb::parse(&bytes) else {
        continue;
      };
      let Ok(joints) = model::skeleton(&glb) else {
        continue;
      };
      models += 1;
      assert_eq!(glb.to_bytes(), bytes, "{}", path.display());
      let actor = rules::goal_name(&library::model_name(&path));
      let notes =
        edit::normalize(&mut glb, &actor).unwrap_or_else(|e| panic!("{}: {e:#}", path.display()));
      for n in &notes {
        *fixed.entry(n.code.to_owned()).or_default() += 1;
      }
      if notes.is_empty() {
        assert_eq!(glb.to_bytes(), bytes, "{}", path.display());
      }
      // what the export writes: the skin set for build-actor, a joint fewer
      edit::for_build_actor(&mut glb).unwrap_or_else(|e| panic!("{}: {e:#}", path.display()));
      assert_eq!(
        model::skeleton(&glb).unwrap().len(),
        joints.len() - usize::from(joints[0].name == "align"),
        "{}",
        path.display()
      );

      let mut glb = Glb::parse(&bytes).unwrap();
      let before = Glb::parse(&bytes).unwrap();
      let wave: Vec<Channel> = vec![Channel {
        joint: joints.len() - 1,
        path: "rotation".into(),
        times: vec![0.0, 1.0],
        values: vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0, FRAC_1_SQRT_2, FRAC_1_SQRT_2],
      }];
      edit::set_animation(&mut glb, "studio-test", &wave).unwrap();
      let old = before.array("animations");
      assert_eq!(&glb.array("animations")[..old.len()].len(), &old.len());
      for (i, anim) in old.iter().enumerate() {
        let keys = |g: &Glb, a: &serde_json::Value| {
          model::channels(g, a)
            .into_iter()
            .map(|c| (c.node, c.path, c.keys.ok()))
            .collect::<Vec<_>>()
        };
        assert_eq!(
          keys(&glb, &glb.array("animations")[i]),
          keys(&before, anim),
          "{} {i}",
          path.display()
        );
      }
      for key in [
        "nodes",
        "meshes",
        "skins",
        "materials",
        "textures",
        "images",
        "samplers",
      ] {
        assert_eq!(glb.json[key], before.json[key], "{} {key}", path.display());
      }
      for (i, image) in before.array("images").iter().enumerate() {
        // an image in a data: uri is part of the JSON compared above
        let data = |g: &Glb, image: &serde_json::Value| {
          let v = &g.array("bufferViews")[image["bufferView"].as_u64()? as usize];
          let at = v["byteOffset"].as_u64().unwrap_or(0) as usize;
          Some(g.bin[at..at + v["byteLength"].as_u64()? as usize].to_vec())
        };
        assert!(
          data(&glb, &glb.array("images")[i]) == data(&before, image),
          "{} image {i}",
          path.display()
        );
      }
    }
  }
  println!("{models} models read and written back byte for byte; fixed by the export: {fixed:?}");
}
