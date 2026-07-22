//! Resource limits for the public API. Documented in the OpenAPI description.

use std::time::Duration;

/// Maximum request body size (1 MiB).
pub const MAX_BODY_BYTES: usize = 1 << 20;

/// Wall-clock timeout for a single request.
pub const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

/// Maximum Monte Carlo trials a run may request; larger counts are rejected
/// (rather than silently clamped) so results are never quietly truncated.
pub const MAX_MC_TRIALS: u32 = 1000;
