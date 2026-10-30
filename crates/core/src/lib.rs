//! OpenGOAL Animation Studio's core, shared by the app and the MCP server: it reads the skeleton and
//! the animations of a skinned .glb, checks them against what OpenGOAL's build-actor needs, and
//! writes animations that build-actor turns into the game's art group.

pub mod edit;
pub mod game_test;
pub mod glb;
pub mod library;
pub mod looks;
pub mod math;
pub mod model;
pub mod names;
pub mod rules;

#[cfg(test)]
mod tests;
