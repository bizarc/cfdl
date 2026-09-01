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

/// A quantile of a CONTINUOUS sample, by linear interpolation between the two
/// order statistics that bracket it — the definition R calls type 7 and the
/// one Excel's PERCENTILE uses, so a published p95 agrees with the figure an
/// analyst reaches for it by hand.
///
/// Interpolating is right here and wrong for `period_distribution` above, and
/// the difference is the quantity, not the code: a period is an observation
/// that exists or does not, so month 9.5 is not an answer; an NPV is a
/// continuous amount, and the sample is a draw from a distribution rather than
/// the population itself.
///
/// At q = 0.5 this reduces exactly to `stats_median` — the midpoint of the two
/// central values for an even sample, the central value for an odd one — which
/// is what keeps every p50 already published identical to what it was.
pub(crate) fn stats_quantile(sorted: &[f64], q: f64) -> f64 {
    debug_assert!(
        sorted.windows(2).all(|w| w[0] <= w[1] || w[0].is_nan()),
        "must be sorted"
    );
    if sorted.is_empty() {
        return 0.0;
    }
    if sorted.len() == 1 {
        return sorted[0];
    }
    let position = q.clamp(0.0, 1.0) * (sorted.len() - 1) as f64;
    let lower = position.floor() as usize;
    let upper = position.ceil() as usize;
    if lower == upper {
        return sorted[lower];
    }
    let weight = position - lower as f64;
    sorted[lower] + (sorted[upper] - sorted[lower]) * weight
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

/// A period distribution over a SORTED, non-empty list of first occurrences.
///
/// Periods are integers and a quantile of them should be a period, not a
/// fraction of one: "the covenant first broke around month 9" is the answer,
/// and 9.5 would be a month that does not exist. So this takes the
/// nearest-rank order statistic rather than interpolating, which is also what
/// makes `min` and `max` come out as themselves. The mean is left fractional,
/// because a mean is explicitly an average rather than an observation.
pub(crate) fn period_distribution(sorted: &[usize]) -> PeriodDistribution {
    debug_assert!(sorted.windows(2).all(|w| w[0] <= w[1]), "must be sorted");
    let n = sorted.len();
    // Nearest-rank: the smallest value at or above the quantile's position.
    let at = |q: f64| -> usize {
        if n == 0 {
            return 0;
        }
        let idx = ((q * n as f64).ceil() as usize).saturating_sub(1);
        sorted[idx.min(n - 1)]
    };
    PeriodDistribution {
        min: sorted.first().copied().unwrap_or(0),
        p10: at(0.10),
        median: at(0.50),
        p90: at(0.90),
        max: sorted.last().copied().unwrap_or(0),
        mean: if n == 0 {
            0.0
        } else {
            round_share(sorted.iter().sum::<usize>() as f64 / n as f64)
        },
    }
}

/// A share or a mean period, to six places — enough to read and stable across
/// platforms, which a bare f64 division is not guaranteed to be in its tail.
pub(crate) fn round_share(value: f64) -> f64 {
    (value * 1_000_000.0).round() / 1_000_000.0
}

/// What one metric NAME drew across the trials of a Monte Carlo run.
///
/// A trial is a complete deterministic run, so every name the base run
/// publishes — `model.irr` and `model.moic`, each `stream.*.total` and
/// `entity.*.total`, every `domain.*` KPI and every metric the model declared
/// — is computed in each trial and can carry a distribution. `docs/13` §7.87
/// records what happened before: the loop built a fresh one-entry map holding
/// `model.npv` and dropped the rest, and since per-trial SERIES are
/// (reasonably) not retained, whatever the loop did not carry out was
/// unrecoverable after it.
///
/// Two things a sample can be that a distribution cannot be taken over, and
/// both are recorded rather than guessed at: a metric published as a STRING,
/// and a metric whose kind CHANGED between trials. Either makes the name
/// unsummarisable, and the per-trial values still publish — the trial rows are
/// the record, this is only the summary over them.
#[derive(Default)]
pub(crate) struct MetricSamples {
    values: Vec<f64>,
    /// Set from the first Money sample and required to match after it. `None`
    /// means every sample so far was a bare number.
    currency: Option<String>,
    unsummarisable: bool,
}

impl MetricSamples {
    pub(crate) fn observe(&mut self, value: &Scalar) {
        match value {
            Scalar::Number(number) => {
                if self.currency.is_some() {
                    self.unsummarisable = true;
                    return;
                }
                self.values.push(*number);
            }
            Scalar::Money(money) => {
                match &self.currency {
                    Some(currency) if *currency != money.currency => {
                        self.unsummarisable = true;
                        return;
                    }
                    Some(_) => {}
                    None => {
                        // A number seen first, then money: the same mixed-kind
                        // objection, from the other side.
                        if !self.values.is_empty() {
                            self.unsummarisable = true;
                            return;
                        }
                        self.currency = Some(money.currency.clone());
                    }
                }
                self.values.push(money.amount);
            }
            // A metric that is a string has no mean and no p95, and neither
            // does one that is null — a trial where a selection matched
            // nothing has no observation to contribute, and averaging it in as
            // zero would be the same mistake `series_max` refuses to make. It
            // is not an error: the metric still reaches every trial row.
            Scalar::String(_) | Scalar::Null => self.unsummarisable = true,
        }
    }

    /// The distribution over the trials that published this name, or `None`
    /// where a distribution is not a thing this metric has.
    ///
    /// Every percentile the schema defines is filled. The engine used to
    /// publish mean, stdev, min, max and p50 and hard-code p01 through p99
    /// `None` — a section whose whole subject is dispersion, declining to
    /// state its tails.
    pub(crate) fn summarise(&self) -> Option<MetricSummary> {
        if self.unsummarisable || self.values.is_empty() {
            return None;
        }
        let mut sorted = self.values.clone();
        sorted.sort_by(|a, b| a.total_cmp(b));
        let scalar = |amount: f64| match &self.currency {
            Some(currency) => Scalar::Money(Money {
                amount: round_amount(amount),
                currency: currency.clone(),
            }),
            None => Scalar::Number(round_amount(amount)),
        };
        let at = |q: f64| Some(scalar(stats_quantile(&sorted, q)));
        Some(MetricSummary {
            r#type: match self.currency {
                Some(_) => "money".to_string(),
                None => "number".to_string(),
            },
            trials: Some(self.values.len() as u32),
            mean: scalar(stats_mean(&self.values)),
            stdev: Some(scalar(stats_stddev_population(&self.values))),
            min: Some(scalar(sorted[0])),
            max: Some(scalar(sorted[sorted.len() - 1])),
            p01: at(0.01),
            p05: at(0.05),
            p10: at(0.10),
            p25: at(0.25),
            // Interpolated at q = 0.5, which is `stats_median` exactly — the
            // NPV p50 every blessed golden already carries does not move.
            p50: scalar(stats_quantile(&sorted, 0.50)),
            p75: at(0.75),
            p90: at(0.90),
            p95: at(0.95),
            p99: at(0.99),
        })
    }
}
