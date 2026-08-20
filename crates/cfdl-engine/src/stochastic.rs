// Extracted from lib.rs — one stage of the engine. See docs/13 §7.44.
use super::*;

pub(crate) fn splitmix64(state: u64) -> u64 {
    let mut z = state.wrapping_add(0x9e3779b97f4a7c15);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
    z ^ (z >> 31)
}

pub(crate) fn next_uniform_open_closed(seed: &mut u64) -> f64 {
    let bits = splitmix64(*seed);
    *seed = bits;
    ((bits as f64) + 1.0) / ((u64::MAX as f64) + 1.0)
}

pub(crate) fn sample_distribution(distribution: &DistributionSpec, seed: &mut u64) -> f64 {
    match distribution {
        DistributionSpec::Fixed { value } => *value,
        DistributionSpec::Uniform { min, max } => {
            if (*max - *min).abs() < f64::EPSILON {
                *min
            } else {
                min + ((*max - *min) * next_uniform_open_closed(seed))
            }
        }
        DistributionSpec::Normal { mean, stddev } => {
            if *stddev == 0.0 {
                *mean
            } else {
                let u1 = next_uniform_open_closed(seed);
                let u2 = next_uniform_open_closed(seed);
                let z0 = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
                mean + stddev * z0
            }
        }
        DistributionSpec::LogNormal { mu, sigma } => {
            if *sigma == 0.0 {
                mu.exp()
            } else {
                let u1 = next_uniform_open_closed(seed);
                let u2 = next_uniform_open_closed(seed);
                let z0 = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
                (mu + sigma * z0).exp()
            }
        }
        DistributionSpec::Triangular { min, mode, max } => {
            if (*max - *min).abs() < f64::EPSILON {
                *min
            } else {
                // Inverse-CDF sampling.
                let u = next_uniform_open_closed(seed);
                let fc = (mode - min) / (max - min);
                if u < fc {
                    min + (u * (max - min) * (mode - min)).sqrt()
                } else {
                    max - ((1.0 - u) * (max - min) * (max - mode)).sqrt()
                }
            }
        }
    }
}

/// FNV-1a hash for per-assumption seed derivation: adding or removing one
/// assumption never reshuffles another assumption's draws.
pub(crate) fn fnv1a(name: &str) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in name.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

pub(crate) fn ir_distribution_spec(dist: &IrDistribution) -> Result<DistributionSpec, EngineError> {
    let get =
        |names: &[&str]| -> Option<f64> { names.iter().find_map(|n| dist.params.get(*n).copied()) };
    let missing = |what: &str| {
        EngineError::InvalidRunConfig(format!(
            "assumption distribution {} missing parameter '{}'",
            dist.kind, what
        ))
    };
    match dist.kind.as_str() {
        "Normal" => Ok(DistributionSpec::Normal {
            mean: get(&["mean"]).ok_or_else(|| missing("mean"))?,
            stddev: get(&["stdev", "stddev"]).ok_or_else(|| missing("stdev"))?,
        }),
        "LogNormal" => Ok(DistributionSpec::LogNormal {
            mu: get(&["mu"]).ok_or_else(|| missing("mu"))?,
            sigma: get(&["sigma"]).ok_or_else(|| missing("sigma"))?,
        }),
        "Uniform" => Ok(DistributionSpec::Uniform {
            min: get(&["min"]).ok_or_else(|| missing("min"))?,
            max: get(&["max"]).ok_or_else(|| missing("max"))?,
        }),
        "Triangular" => Ok(DistributionSpec::Triangular {
            min: get(&["min"]).ok_or_else(|| missing("min"))?,
            mode: get(&["mode"]).ok_or_else(|| missing("mode"))?,
            max: get(&["max"]).ok_or_else(|| missing("max"))?,
        }),
        other => Err(EngineError::InvalidRunConfig(format!(
            "unknown assumption distribution kind: {other}"
        ))),
    }
}

pub(crate) fn apply_clip(value: f64, clip: Option<[f64; 2]>) -> f64 {
    match clip {
        Some([lo, hi]) => value.clamp(lo, hi),
        None => value,
    }
}

/// Deterministic central value of a distribution, used outside Monte Carlo:
/// Normal -> mean; LogNormal -> exp(mu + sigma^2/2) (the distribution mean);
/// Uniform -> midpoint; Triangular -> (min + mode + max) / 3.
pub(crate) fn central_value(spec: &DistributionSpec) -> f64 {
    match spec {
        DistributionSpec::Fixed { value } => *value,
        DistributionSpec::Normal { mean, .. } => *mean,
        DistributionSpec::LogNormal { mu, sigma } => (mu + sigma * sigma / 2.0).exp(),
        DistributionSpec::Uniform { min, max } => (min + max) / 2.0,
        DistributionSpec::Triangular { min, mode, max } => (min + mode + max) / 3.0,
    }
}

pub(crate) fn stats_mean(values: &[f64]) -> f64 {
    values.iter().sum::<f64>() / values.len() as f64
}

pub(crate) fn stats_median(values: &[f64]) -> f64 {
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.total_cmp(b));
    let mid = sorted.len() / 2;
    if sorted.len().is_multiple_of(2) {
        (sorted[mid - 1] + sorted[mid]) / 2.0
    } else {
        sorted[mid]
    }
}

pub(crate) fn stats_stddev_population(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let mean = stats_mean(values);
    let variance = values
        .iter()
        .map(|value| {
            let delta = value - mean;
            delta * delta
        })
        .sum::<f64>()
        / values.len() as f64;
    variance.sqrt()
}

pub(crate) fn probability_negative(values: &[f64]) -> f64 {
    let negatives = values.iter().filter(|value| **value < 0.0).count();
    negatives as f64 / values.len() as f64
}
