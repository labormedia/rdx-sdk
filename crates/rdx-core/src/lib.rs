//! Core library for the Reaction–Diffusion Extended Exchange Economy scaffold.
//!
//! Key modules:
//! - goods: service taxonomy as goods
//! - preferences: aggregated Cobb–Douglas profile + alpha-to-base
//! - pareto_oracle: dyadic Pareto-optimal exchange oracle for (A,B)
//! - trade: P2P evaluation across all goods vs base
//! - sim: simulation loop and metrics
//! - codec: (optional) encoding/decoding boundary for preference payloads

pub mod codec;
pub mod math;
pub mod model;
pub mod pareto_oracle;
pub mod preferences;
pub mod trade;
pub mod sim;
pub mod reaction;