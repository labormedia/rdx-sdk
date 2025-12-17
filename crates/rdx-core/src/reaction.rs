//! Reaction (endogenous transformation) module.
//!
//! This module hosts the *reaction term* of the Reaction–Diffusion economy:
//! local, agent-controlled transformations of an endowment vector via
//! linear "reaction rules" of the form:
//!
//!   inputs (a) + intensity x  -> outputs (b)
//!
//! For now this is intentionally **standalone**: it is *not* wired into the
//! simulation loop. The goal is to provide a stable API surface so the paper
//! repository can encode/deserialize reaction rules and later call them from
//! `sim.rs` as a Phase-1 step.
extern crate alloc;
use alloc::collections::btree_map::BTreeMap;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct ReactionRuleSpec {
    pub id: String,
    pub size_class: String,   // or enum SizeClass with serde(rename_all="UPPERCASE")
    pub name: String,
    pub lead: usize,          // index of the “lead” good

    pub inputs: BTreeMap<usize, f64>,   // { "0": 1.0, "35": 1.0, ... }
    pub outputs: BTreeMap<usize, f64>,  // { "1": 1.15, "35": 1.0, ... }
}