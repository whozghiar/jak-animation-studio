//! What the games say of their entities, by art group:
//! - names.json: the official names in English and French (game -> art group -> name, each with its
//!   source: the game's own text or the Jak and Daxter wikis). Props without one are not in it.
//! - living.json: which models are living entities (characters, creatures, enemies and bosses) and
//!   which are objects (props, platforms, vehicles, turrets, collectables, pieces and debris, robots
//!   and mechanical bosses). Made from jak-project's code (the model's own type derives from
//!   nav-enemy, enemy or process-taskable), then checked by hand: creatures written as plain
//!   process-drawable, cutscene characters and bosses added; turrets, explosions, parts and robots
//!   (the wiki's KG Death Bots, mechs, devices and Dark Maker units) removed.

use std::{
  collections::{HashMap, HashSet},
  sync::OnceLock,
};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Title {
  pub en: String,
  /// none when no French source names it
  pub fr: Option<String>,
}

type Names = HashMap<String, HashMap<String, Title>>;

#[derive(Deserialize)]
struct Kinds {
  objects: HashSet<String>,
}

fn names() -> &'static Names {
  static NAMES: OnceLock<Names> = OnceLock::new();
  NAMES.get_or_init(|| serde_json::from_str(include_str!("names.json")).expect("names.json"))
}

fn kinds() -> &'static HashMap<String, Kinds> {
  static KINDS: OnceLock<HashMap<String, Kinds>> = OnceLock::new();
  KINDS.get_or_init(|| serde_json::from_str(include_str!("living.json")).expect("living.json"))
}

/// The art group of a model: "babak" for babak-lod0.glb.
pub(crate) fn art_group(model: &str) -> &str {
  model
    .rsplit_once("-lod")
    .filter(|(_, n)| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()))
    .map_or(model, |(base, _)| base)
}

/// The official name of a model of `game`, when it has one.
pub fn title(game: &str, model: &str) -> Option<Title> {
  names().get(game)?.get(art_group(model)).cloned()
}

/// Whether a model is one of `game`'s objects rather than a living entity. A model the game does
/// not have (a model of your own) is not.
pub fn is_object(game: &str, model: &str) -> bool {
  kinds()
    .get(game)
    .is_some_and(|k| k.objects.contains(art_group(model)))
}
