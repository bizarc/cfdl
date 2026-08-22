// Extracted from lib.rs — one stage of the engine. See docs/13 §7.44.
use super::*;

#[derive(Debug, Clone)]
pub struct RunConfig {
    pub discount_rate: f64,
    pub as_of: Option<Date>,
    pub parameter_overrides: BTreeMap<String, f64>,
    pub scenarios: BTreeMap<String, ScenarioRunConfig>,
    pub monte_carlo: Option<MonteCarloRunConfig>,
    /// The grain to value at — `"annual"`, or `None` for the model grid.
    ///
    /// Named for the `Grain` it produces, because that is what it is choosing.
    /// Two alternatives were considered and rejected: "discounting period"
    /// collides with `period`, which means a model grid period everywhere else
    /// in CFDL, and "cadence" is already taken — `pack.cadences` means which
    /// model calendars a pack's rules lower correctly on, which is a
    /// compatibility list rather than an aggregation frequency.
    ///
    /// `None` means the model grid, which is what every run did before this
    /// existed and is what keeps published NPVs unmoved.
    ///
    /// Naming a coarser grain — currently `"annual"` — sums cash into those
    /// buckets and discounts each once, which is the convention published
    /// sources use ("sum NOI by year, then discount the year"). Until this,
    /// `ppy` came from `ir.time.calendar`, so a model's CALENDAR silently
    /// decided its valuation convention: the same deal built monthly and
    /// annually returned different present values, and `mit_rentleg_plaza`
    /// records that as a 1.3% gap it attributed to the rebuild rather than to
    /// the coupling.
    pub valuation_grain: Option<String>,
    /// Which arithmetic expressions evaluate under.
    ///
    /// Decimal is the default and is what every published number uses.
    /// `"excel_compat"` reproduces a spreadsheet's float64 artifacts, for
    /// reconciling against a workbook rather than for producing an answer. It
    /// is a property of the RUN because it describes the comparison being made,
    /// not the deal: the same model reconciles against a spreadsheet under one
    /// mode and states its own numbers under the other.
    pub arithmetic: cfdl_expr::Mode,
}

#[derive(Debug, Clone)]
pub struct ScenarioRunConfig {
    pub discount_rate: Option<f64>,
    pub as_of: Option<Date>,
    pub parameter_overrides: BTreeMap<String, f64>,
}

#[derive(Debug, Clone)]
pub struct MonteCarloRunConfig {
    pub trial_count: u32,
    pub seed: u64,
    pub distributions: BTreeMap<String, ConfiguredDistribution>,
}

/// A run-config distribution and its optional clip bounds.
///
/// Mirrors what `assume x ~ Dist(..., clip=[lo, hi])` expresses in the
/// language, so a driver declared in either place behaves identically.
#[derive(Debug, Clone)]
pub struct ConfiguredDistribution {
    pub spec: DistributionSpec,
    pub clip: Option<[f64; 2]>,
}

#[derive(Debug, Clone)]
pub enum DistributionSpec {
    Fixed { value: f64 },
    Normal { mean: f64, stddev: f64 },
    Uniform { min: f64, max: f64 },
    LogNormal { mu: f64, sigma: f64 },
    Triangular { min: f64, mode: f64, max: f64 },
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RunConfigFile {
    #[serde(default)]
    pub(crate) deterministic: DeterministicConfigFile,
    #[serde(default)]
    pub(crate) scenarios: BTreeMap<String, ScenarioConfigFile>,
    pub(crate) monte_carlo: Option<MonteCarloConfigFile>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DeterministicConfigFile {
    #[serde(rename = "annual_discount_rate")]
    pub(crate) discount_rate: Option<f64>,
    pub(crate) as_of: Option<String>,
    /// How NPV groups cash before discounting. Omitted, the model's own grain:
    /// each cash flow discounts at the annual rate raised to its fractional
    /// year, which for an annual model IS annual discounting. `"annual"`
    /// buckets a finer model's cash by calendar year first and discounts the
    /// buckets at the annual rate — the convention a hand-built annual
    /// spreadsheet uses on monthly data.
    pub(crate) valuation_grain: Option<String>,
    #[serde(default)]
    pub(crate) arithmetic: Option<String>,
    #[serde(default)]
    pub(crate) parameters: BTreeMap<String, f64>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ScenarioConfigFile {
    #[serde(rename = "annual_discount_rate")]
    pub(crate) discount_rate: Option<f64>,
    pub(crate) as_of: Option<String>,
    #[serde(default)]
    pub(crate) parameters: BTreeMap<String, f64>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct MonteCarloConfigFile {
    pub(crate) trial_count: u32,
    pub(crate) seed: u64,
    #[serde(default)]
    pub(crate) distributions: BTreeMap<String, DistributionConfigFile>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum DistributionConfigFile {
    Fixed {
        value: f64,
        #[serde(default)]
        clip: Option<[f64; 2]>,
    },
    Normal {
        mean: f64,
        // The language spells this `stdev`; `stddev` predates it and is kept
        // so existing run configs keep working.
        #[serde(alias = "stddev")]
        stdev: f64,
        #[serde(default)]
        clip: Option<[f64; 2]>,
    },
    Uniform {
        min: f64,
        max: f64,
        #[serde(default)]
        clip: Option<[f64; 2]>,
    },
    LogNormal {
        mu: f64,
        sigma: f64,
        #[serde(default)]
        clip: Option<[f64; 2]>,
    },
    Triangular {
        min: f64,
        mode: f64,
        max: f64,
        #[serde(default)]
        clip: Option<[f64; 2]>,
    },
}

pub fn run_config_from_json_file(
    path: &Path,
    fallback_rate: f64,
    fallback_as_of: Option<Date>,
) -> Result<RunConfig, EngineError> {
    let raw = std::fs::read_to_string(path)?;
    run_config_from_json_str(&raw, fallback_rate, fallback_as_of)
}

pub fn run_config_from_json_str(
    raw: &str,
    fallback_rate: f64,
    fallback_as_of: Option<Date>,
) -> Result<RunConfig, EngineError> {
    let config_file: RunConfigFile = serde_json::from_str(raw)?;
    run_config_from_value(config_file, fallback_rate, fallback_as_of)
}

pub(crate) fn run_config_from_value(
    config_file: RunConfigFile,
    fallback_rate: f64,
    fallback_as_of: Option<Date>,
) -> Result<RunConfig, EngineError> {
    let mut config = RunConfig {
        discount_rate: config_file
            .deterministic
            .discount_rate
            .unwrap_or(fallback_rate),
        as_of: fallback_as_of,
        parameter_overrides: config_file.deterministic.parameters,
        scenarios: BTreeMap::new(),
        monte_carlo: None,
        valuation_grain: match config_file.deterministic.valuation_grain.as_deref() {
            None | Some("period") => None,
            Some("annual") => Some("annual".to_string()),
            Some(other) => {
                return Err(EngineError::InvalidRunConfig(format!(
                    "valuation_grain '{other}' is not a grain this engine knows; \
                     use \"annual\", or omit it for the model's own grain"
                )))
            }
        },
        arithmetic: match config_file.deterministic.arithmetic.as_deref() {
            None | Some("decimal") => cfdl_expr::Mode::Decimal,
            Some("excel_compat") => cfdl_expr::Mode::ExcelCompat,
            Some(other) => {
                return Err(EngineError::InvalidRunConfig(format!(
                    "arithmetic '{other}' is not an arithmetic this engine knows; \
                     use \"excel_compat\", or omit it for decimal"
                )))
            }
        },
    };

    if let Some(as_of) = config_file.deterministic.as_of {
        config.as_of = Some(Date::parse(&as_of)?);
    }

    for (name, scenario) in config_file.scenarios {
        let scenario_as_of = match scenario.as_of {
            Some(raw) => Some(Date::parse(&raw)?),
            None => None,
        };
        config.scenarios.insert(
            name,
            ScenarioRunConfig {
                discount_rate: scenario.discount_rate,
                as_of: scenario_as_of,
                parameter_overrides: scenario.parameters,
            },
        );
    }

    if let Some(monte_carlo) = config_file.monte_carlo {
        if monte_carlo.trial_count == 0 {
            return Err(EngineError::InvalidRunConfig(
                "monte_carlo.trial_count must be >= 1".to_string(),
            ));
        }
        let mut distributions = BTreeMap::new();
        for (name, dist) in monte_carlo.distributions {
            let (spec, clip) = match dist {
                DistributionConfigFile::Fixed { value, clip } => {
                    (DistributionSpec::Fixed { value }, clip)
                }
                DistributionConfigFile::Normal { mean, stdev, clip } => {
                    if stdev < 0.0 {
                        return Err(EngineError::InvalidRunConfig(format!(
                            "distribution '{name}' has negative stdev"
                        )));
                    }
                    (
                        DistributionSpec::Normal {
                            mean,
                            stddev: stdev,
                        },
                        clip,
                    )
                }
                DistributionConfigFile::Uniform { min, max, clip } => {
                    if min > max {
                        return Err(EngineError::InvalidRunConfig(format!(
                            "distribution '{name}' has min > max"
                        )));
                    }
                    (DistributionSpec::Uniform { min, max }, clip)
                }
                DistributionConfigFile::LogNormal { mu, sigma, clip } => {
                    if sigma < 0.0 {
                        return Err(EngineError::InvalidRunConfig(format!(
                            "distribution '{name}' has negative sigma"
                        )));
                    }
                    (DistributionSpec::LogNormal { mu, sigma }, clip)
                }
                DistributionConfigFile::Triangular {
                    min,
                    mode,
                    max,
                    clip,
                } => {
                    if !(min <= mode && mode <= max) {
                        return Err(EngineError::InvalidRunConfig(format!(
                            "distribution '{name}' requires min <= mode <= max"
                        )));
                    }
                    (DistributionSpec::Triangular { min, mode, max }, clip)
                }
            };
            if let Some([lo, hi]) = clip {
                if lo > hi {
                    return Err(EngineError::InvalidRunConfig(format!(
                        "distribution '{name}' has clip lower bound above upper bound"
                    )));
                }
            }
            distributions.insert(name, ConfiguredDistribution { spec, clip });
        }
        config.monte_carlo = Some(MonteCarloRunConfig {
            trial_count: monte_carlo.trial_count,
            seed: monte_carlo.seed,
            distributions,
        });
    }

    Ok(config)
}
