//! Codec boundary for preference payloads (serialization/deserialization).
//!
//! Default: serde_json.
//! Optional feature `mvcf`: call into `multivariate-convex-function` crate.
//!
//! The intention is to support P2P transmission of preference profiles / aggregated Cobb–Douglas
//! parameters, so peers can evaluate dyadic trades.

use serde::{Serialize, de::DeserializeOwned};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CodecError {
    #[error("serde_json error: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[cfg(feature = "mvcf")]
    #[error("mvcf codec error: {0}")]
    Mvcf(String),
}

pub fn encode<T: Serialize>(v: &T) -> Result<Vec<u8>, CodecError> {
    #[cfg(feature = "mvcf")]
    {
        // NOTE: Wire this to the real API of the crate once confirmed.
        // Placeholder keeps compilation stable by falling back to JSON even with the feature on.
        // Replace with crate-provided encoding when available.
        let bytes = serde_json::to_vec(v)?;
        return Ok(bytes);
    }

    #[cfg(not(feature = "mvcf"))]
    {
        Ok(serde_json::to_vec(v)?)
    }
}

pub fn decode<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, CodecError> {
    #[cfg(feature = "mvcf")]
    {
        // NOTE: Wire this to the real API of the crate once confirmed.
        let v = serde_json::from_slice(bytes)?;
        return Ok(v);
    }

    #[cfg(not(feature = "mvcf"))]
    {
        Ok(serde_json::from_slice(bytes)?)
    }
}
