use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackLoadError {
    pub message: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PackRegistry {
    packs: BTreeMap<String, LoadedPack>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LoadedPack {
    pub manifest: PackManifest,
    pub aliases: BTreeMap<String, String>,
    pub templates: Vec<PackTemplate>,
    pub lowering_rules: Vec<LoweringRule>,
    pub metric_specs: Vec<MetricSpec>,
    pub validations: Vec<PackValidation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct PackManifest {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub entrypoints: PackEntrypoints,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
pub struct PackEntrypoints {
    #[serde(default)]
    pub aliases: Option<String>,
    #[serde(default)]
    pub templates: Option<String>,
    #[serde(default)]
    pub lowering: Option<String>,
    #[serde(default)]
    pub metrics: Option<String>,
    #[serde(default)]
    pub validations: Option<String>,
}

/// A single declarative domain check supplied by a pack.
///
/// Packs own *what* to check (which term, which bound, which stable code);
/// the compiler owns spans, timeline access, and diagnostic emission. The
/// check kinds are a closed set with no expressions, recursion, or
/// interpolation, so a pack can never crash, hang, or allocate unboundedly
/// in the compiler.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PackValidation {
    /// Exactly one of `contract` / `contracts` must be set.
    #[serde(default)]
    pub contract: Option<String>,
    #[serde(default)]
    pub contracts: Vec<String>,
    /// How a contract name is matched: exact, or `base.instance` suffixes.
    #[serde(default, rename = "match")]
    pub match_kind: ContractMatch,
    pub code: String,
    pub message: String,
    #[serde(default)]
    pub severity: ValidationSeverity,
    pub check: ValidationCheck,
    /// Term under test (`term_present`, `term_number`, `term_enum`).
    #[serde(default)]
    pub term: Option<String>,
    /// Terms for `any_term_present`.
    #[serde(default)]
    pub terms: Vec<String>,
    #[serde(default)]
    pub number: NumberKind,
    #[serde(default)]
    pub when: WhenPresence,
    #[serde(default)]
    pub on_invalid: OnInvalid,
    #[serde(default)]
    pub min: Option<f64>,
    #[serde(default)]
    pub max: Option<f64>,
    #[serde(default)]
    pub exclusive_min: Option<f64>,
    #[serde(default)]
    pub exclusive_max: Option<f64>,
    /// Allowed values for `term_enum`.
    #[serde(default)]
    pub values: Vec<ValidationValue>,
    /// `term_compare` operands.
    #[serde(default)]
    pub left: Option<String>,
    #[serde(default)]
    pub right: Option<String>,
    #[serde(default)]
    pub op: Option<CompareOp>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContractMatch {
    #[default]
    Exact,
    /// Matches `<contract>` and `<contract>.<instance>` suffixed forms.
    Instance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationSeverity {
    #[default]
    Error,
    Warning,
    Info,
}

impl ValidationSeverity {
    pub fn as_str(self) -> &'static str {
        match self {
            ValidationSeverity::Error => "error",
            ValidationSeverity::Warning => "warning",
            ValidationSeverity::Info => "info",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationCheck {
    /// The term must be present.
    TermPresent,
    /// At least one of `terms` must be present.
    AnyTermPresent,
    /// The term must parse as a number and satisfy any declared bounds.
    TermNumber,
    /// The contract term range must be valid and inside the model timeline.
    TermRangeWithinTimeline,
    /// The term must equal one of `values`.
    TermEnum,
    /// Two numeric terms must satisfy `left <op> right`.
    TermCompare,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NumberKind {
    #[default]
    Decimal,
    Integer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WhenPresence {
    /// Run even when the term is absent (absence is itself a failure).
    #[default]
    Always,
    /// Only run when the term is present.
    Present,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OnInvalid {
    /// An unparseable value fails this check.
    #[default]
    Report,
    /// An unparseable value is another check's responsibility.
    Skip,
}

/// A literal an enum check compares against. Accepts TOML strings, integers,
/// and floats without exposing the TOML value type to consumers.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum ValidationValue {
    Integer(i64),
    Float(f64),
    Text(String),
}

impl ValidationValue {
    /// Whether a raw term value equals this literal.
    pub fn matches(&self, raw: &str) -> bool {
        match self {
            ValidationValue::Text(text) => text == raw,
            ValidationValue::Integer(number) => raw
                .parse::<i64>()
                .map(|parsed| parsed == *number)
                .unwrap_or(false),
            ValidationValue::Float(number) => raw
                .parse::<f64>()
                .map(|parsed| (parsed - *number).abs() < f64::EPSILON)
                .unwrap_or(false),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompareOp {
    Le,
    Lt,
    Ge,
    Gt,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
struct ValidationsFile {
    #[serde(default)]
    schema_version: Option<u32>,
    #[serde(default)]
    code_prefix: Option<String>,
    #[serde(default)]
    validations: Vec<PackValidation>,
}

impl PackValidation {
    /// Contract names this validation applies to.
    pub fn contract_names(&self) -> Vec<&str> {
        match &self.contract {
            Some(name) => vec![name.as_str()],
            None => self.contracts.iter().map(String::as_str).collect(),
        }
    }

    /// Whether this validation applies to a contract declared in a model.
    pub fn applies_to(&self, contract_name: &str) -> bool {
        self.contract_names()
            .into_iter()
            .any(|declared| match self.match_kind {
                ContractMatch::Exact => declared == contract_name,
                ContractMatch::Instance => {
                    contract_name == declared
                        || contract_name
                            .strip_prefix(declared)
                            .is_some_and(|rest| rest.starts_with('.'))
                }
            })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct LoweringFile {
    #[serde(default)]
    pub rules: Vec<LoweringRule>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct LoweringRule {
    pub id: String,
    pub contract_name: String,
    pub stream_name: String,
    pub owner_entity: String,
    pub direction: String,
    /// Currency the stream is denominated in.
    ///
    /// Omit it unless the instrument is genuinely fixed to one currency: an
    /// empty value inherits the model's declared currency, which is what makes
    /// a pack usable outside the United States. A PPA in Rajasthan is not a
    /// USD contract.
    #[serde(default)]
    pub currency: String,
    /// May contain `{{contract.<key>}}` placeholders (see expand_rule_template).
    pub amount_expr: String,
    pub schedule_kind: String,
    /// Annuity due: the stream pays at the start of each interval.
    ///
    /// Streams that behave like an expense — opex, rent paid, fees, capex —
    /// fall due in the period they belong to. Streams that behave like an
    /// annuity — coupons, debt service, pool collections — pay at the end of
    /// the interval that earned them, which is the default.
    #[serde(default)]
    pub schedule_due: bool,
    /// May contain `{{contract.term_start}}` / `{{contract.<key>}}` placeholders.
    pub schedule_from: String,
    /// May contain `{{contract.term_end}}` / `{{contract.<key>}}` placeholders.
    pub schedule_to: String,
    /// Default values for template placeholders when the contract does not
    /// declare the term. Keys are the bare placeholder names (no `contract.`
    /// prefix), e.g. `"lease_up.months" = "18"`.
    #[serde(default)]
    pub defaults: BTreeMap<String, String>,
}

/// Declarative domain-metric definition (metrics.toml). Metrics are
/// evaluated in file order, so ratio metrics may reference earlier ones.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct MetricSpec {
    /// Output key, e.g. "domain.cre.noi".
    pub id: String,
    /// "money" | "number"
    pub kind: String,
    /// "sum" (numerator + denominator stream totals, signed),
    /// "negated_sum" (-sum of numerator stream totals),
    /// "ratio" (numerator_metric / denominator_metric).
    pub op: String,
    #[serde(default)]
    pub numerator_streams: Vec<String>,
    #[serde(default)]
    pub denominator_streams: Vec<String>,
    #[serde(default)]
    pub numerator_metric: Option<String>,
    #[serde(default)]
    pub denominator_metric: Option<String>,
    /// Human-readable lineage formula, emitted verbatim.
    pub formula: String,
    /// Omit the metric unless its value is strictly positive.
    #[serde(default)]
    pub require_positive: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct MetricsFile {
    #[serde(default)]
    metrics: Vec<MetricSpec>,
}

/// Standard packs compiled into the library for hosts without filesystem
/// access (WASM playground, API server). Enabled by the `embedded-packs`
/// feature. NOTE: `include_str!` paths assume the repo layout (packs/ at the
/// workspace root); for crates.io publishing the pack data moves into the
/// crate — tracked for the 1.0 packaging pass.
#[cfg(feature = "embedded-packs")]
mod embedded {
    pub type EmbeddedFile = (&'static str, &'static str);

    pub const CRE: &[EmbeddedFile] = &[
        ("pack.toml", include_str!("../../../packs/cre/pack.toml")),
        (
            "aliases.toml",
            include_str!("../../../packs/cre/aliases.toml"),
        ),
        (
            "templates.toml",
            include_str!("../../../packs/cre/templates.toml"),
        ),
        (
            "lowering/rules.toml",
            include_str!("../../../packs/cre/lowering/rules.toml"),
        ),
        (
            "metrics.toml",
            include_str!("../../../packs/cre/metrics.toml"),
        ),
        (
            "validations.toml",
            include_str!("../../../packs/cre/validations.toml"),
        ),
    ];

    pub const OPCO: &[EmbeddedFile] = &[
        ("pack.toml", include_str!("../../../packs/opco/pack.toml")),
        (
            "aliases.toml",
            include_str!("../../../packs/opco/aliases.toml"),
        ),
        (
            "lowering/rules.toml",
            include_str!("../../../packs/opco/lowering/rules.toml"),
        ),
        (
            "metrics.toml",
            include_str!("../../../packs/opco/metrics.toml"),
        ),
        (
            "validations.toml",
            include_str!("../../../packs/opco/validations.toml"),
        ),
    ];

    pub const CREDIT: &[EmbeddedFile] = &[
        ("pack.toml", include_str!("../../../packs/credit/pack.toml")),
        (
            "aliases.toml",
            include_str!("../../../packs/credit/aliases.toml"),
        ),
        (
            "lowering/rules.toml",
            include_str!("../../../packs/credit/lowering/rules.toml"),
        ),
        (
            "metrics.toml",
            include_str!("../../../packs/credit/metrics.toml"),
        ),
        (
            "validations.toml",
            include_str!("../../../packs/credit/validations.toml"),
        ),
    ];

    pub const ENERGY: &[EmbeddedFile] = &[
        ("pack.toml", include_str!("../../../packs/energy/pack.toml")),
        (
            "aliases.toml",
            include_str!("../../../packs/energy/aliases.toml"),
        ),
        (
            "lowering/rules.toml",
            include_str!("../../../packs/energy/lowering/rules.toml"),
        ),
        (
            "metrics.toml",
            include_str!("../../../packs/energy/metrics.toml"),
        ),
        (
            "validations.toml",
            include_str!("../../../packs/energy/validations.toml"),
        ),
    ];

    pub const ALL: &[&[EmbeddedFile]] = &[CRE, CREDIT, ENERGY, OPCO];
}

impl PackRegistry {
    /// Load the standard packs compiled into the library (no filesystem).
    #[cfg(feature = "embedded-packs")]
    pub fn load_embedded() -> Result<Self, PackLoadError> {
        let mut packs = BTreeMap::new();
        for files in embedded::ALL {
            let lookup = |relative: Option<&str>| -> Option<&'static str> {
                let relative = relative?;
                files
                    .iter()
                    .find(|(name, _)| *name == relative)
                    .map(|(_, content)| *content)
            };
            let manifest_raw = lookup(Some("pack.toml")).ok_or_else(|| PackLoadError {
                message: "Embedded pack is missing pack.toml".to_string(),
            })?;
            let manifest: PackManifest =
                toml::from_str(manifest_raw).map_err(|err| PackLoadError {
                    message: format!("Failed to parse embedded pack manifest: {err}"),
                })?;
            let source = format!("embedded:{}", manifest.name);
            let aliases = match lookup(manifest.entrypoints.aliases.as_deref()) {
                Some(raw) => parse_aliases(raw, &source)?,
                None => BTreeMap::new(),
            };
            let templates = match lookup(manifest.entrypoints.templates.as_deref()) {
                Some(raw) => parse_templates(raw, &source)?,
                None => Vec::new(),
            };
            let lowering_rules = match lookup(manifest.entrypoints.lowering.as_deref()) {
                Some(raw) => parse_lowering_rules(raw, &source)?,
                None => Vec::new(),
            };
            let metric_specs = match lookup(manifest.entrypoints.metrics.as_deref()) {
                Some(raw) => parse_metric_specs(raw, &source)?,
                None => Vec::new(),
            };
            let validations = match lookup(manifest.entrypoints.validations.as_deref()) {
                Some(raw) => parse_validations(raw, &source)?,
                None => Vec::new(),
            };
            packs.insert(
                manifest.name.clone(),
                LoadedPack {
                    manifest,
                    aliases,
                    templates,
                    lowering_rules,
                    metric_specs,
                    validations,
                },
            );
        }
        Ok(Self { packs })
    }
}

/// Expand `{{contract.<key>}}` placeholders in a lowering-rule template.
///
/// `resolve` maps a bare key (e.g. `base_rent`, `term_start`,
/// `lease_up.months`) to its value; unresolved keys are collected and
/// returned as `Err` so the caller can emit one diagnostic per missing term.
/// Substitution is textual: numeric contract terms yield valid expression
/// fragments, string terms must be quoted inside the template.
pub fn expand_rule_template(
    template: &str,
    resolve: &dyn Fn(&str) -> Option<String>,
) -> Result<String, Vec<String>> {
    let mut out = String::with_capacity(template.len());
    let mut missing: Vec<String> = Vec::new();
    let mut rest = template;
    while let Some(start) = rest.find("{{") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 2..];
        let Some(end) = after.find("}}") else {
            // Unterminated placeholder: treat the remainder as literal text.
            out.push_str(&rest[start..]);
            rest = "";
            break;
        };
        let raw_key = after[..end].trim();
        let key = raw_key.strip_prefix("contract.").unwrap_or(raw_key);
        match resolve(key) {
            Some(value) => out.push_str(&value),
            None => {
                if !missing.iter().any(|k| k == key) {
                    missing.push(key.to_string());
                }
            }
        }
        rest = &after[end + 2..];
    }
    out.push_str(rest);
    if missing.is_empty() {
        Ok(out)
    } else {
        Err(missing)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActivePack {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateExpansionRequest {
    pub template: String,
    pub params: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateExpansion {
    pub generated_nodes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct TemplateFile {
    #[serde(default)]
    pub templates: Vec<PackTemplate>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct PackTemplate {
    pub id: String,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub kind: Option<String>,
    pub body: String,
    #[serde(default)]
    pub defaults: BTreeMap<String, String>,
}

impl PackRegistry {
    pub fn load_from_dir(root: &Path) -> Result<Self, PackLoadError> {
        if !root.exists() {
            return Ok(Self {
                packs: BTreeMap::new(),
            });
        }
        if !root.is_dir() {
            return Err(PackLoadError {
                message: format!("Pack root '{}' is not a directory.", root.display()),
            });
        }

        let mut subdirs: Vec<PathBuf> = fs::read_dir(root)
            .map_err(io_err)?
            .filter_map(|entry| entry.ok().map(|e| e.path()))
            .filter(|path| path.is_dir())
            .collect();
        subdirs.sort();

        let mut packs = BTreeMap::new();
        for pack_dir in subdirs {
            let manifest_path = pack_dir.join("pack.toml");
            if !manifest_path.exists() {
                continue;
            }
            let manifest_raw = fs::read_to_string(&manifest_path).map_err(io_err)?;
            let manifest: PackManifest =
                toml::from_str(&manifest_raw).map_err(|err| PackLoadError {
                    message: format!(
                        "Failed to parse manifest '{}': {err}",
                        manifest_path.display()
                    ),
                })?;
            let aliases = load_aliases(&pack_dir, manifest.entrypoints.aliases.as_deref())?;
            let templates = load_templates(&pack_dir, manifest.entrypoints.templates.as_deref())?;
            let lowering_rules =
                load_lowering_rules(&pack_dir, manifest.entrypoints.lowering.as_deref())?;
            let metric_specs =
                load_metric_specs(&pack_dir, manifest.entrypoints.metrics.as_deref())?;
            let validations =
                load_validations(&pack_dir, manifest.entrypoints.validations.as_deref())?;

            packs.insert(
                manifest.name.clone(),
                LoadedPack {
                    manifest,
                    aliases,
                    templates,
                    lowering_rules,
                    metric_specs,
                    validations,
                },
            );
        }

        Ok(Self { packs })
    }

    pub fn list(&self) -> Vec<&LoadedPack> {
        self.packs.values().collect()
    }

    pub fn active_pack(&self, name: &str, version: &str) -> Option<ActivePack> {
        self.packs.get(name).and_then(|pack| {
            if pack.manifest.version == version {
                Some(ActivePack {
                    name: pack.manifest.name.clone(),
                    version: pack.manifest.version.clone(),
                })
            } else {
                None
            }
        })
    }

    pub fn lookup_alias(&self, pack_name: &str, alias: &str) -> Option<&str> {
        self.packs
            .get(pack_name)
            .and_then(|pack| pack.aliases.get(alias))
            .map(String::as_str)
    }

    pub fn metric_specs(&self, pack_name: &str) -> Vec<MetricSpec> {
        self.packs
            .get(pack_name)
            .map(|pack| pack.metric_specs.clone())
            .unwrap_or_default()
    }

    pub fn validations(&self, pack_name: &str) -> Vec<PackValidation> {
        self.packs
            .get(pack_name)
            .map(|pack| pack.validations.clone())
            .unwrap_or_default()
    }

    pub fn lowering_rules(&self, pack_name: &str) -> Vec<LoweringRule> {
        self.packs
            .get(pack_name)
            .map(|pack| pack.lowering_rules.clone())
            .unwrap_or_default()
    }

    pub fn templates(&self, pack_name: &str) -> Vec<PackTemplate> {
        self.packs
            .get(pack_name)
            .map(|pack| pack.templates.clone())
            .unwrap_or_default()
    }

    pub fn template(&self, pack_name: &str, template_id: &str) -> Option<PackTemplate> {
        self.packs.get(pack_name).and_then(|pack| {
            pack.templates
                .iter()
                .find(|template| template.id == template_id)
                .cloned()
        })
    }

    pub fn expand_template(
        &self,
        pack_name: &str,
        request: TemplateExpansionRequest,
    ) -> Result<TemplateExpansion, PackLoadError> {
        let pack = self.packs.get(pack_name).ok_or_else(|| PackLoadError {
            message: format!("Pack '{pack_name}' is not loaded."),
        })?;
        let template = pack
            .templates
            .iter()
            .find(|template| template.id == request.template)
            .ok_or_else(|| PackLoadError {
                message: format!(
                    "Template '{}' was not found in pack '{}'.",
                    request.template, pack_name
                ),
            })?;
        let text = expand_template_body(template, &request.params);
        Ok(TemplateExpansion {
            generated_nodes: vec![text],
        })
    }
}

fn load_aliases(
    pack_dir: &Path,
    aliases_path: Option<&str>,
) -> Result<BTreeMap<String, String>, PackLoadError> {
    let Some(relative) = aliases_path else {
        return Ok(BTreeMap::new());
    };
    let path = pack_dir.join(relative);
    let raw = fs::read_to_string(&path).map_err(io_err)?;
    parse_aliases(&raw, &path.display().to_string())
}

fn parse_aliases(raw: &str, source: &str) -> Result<BTreeMap<String, String>, PackLoadError> {
    let parsed: AliasFile = toml::from_str(raw).map_err(|err| PackLoadError {
        message: format!("Failed to parse aliases '{source}': {err}"),
    })?;
    Ok(parsed.aliases)
}

fn load_validations(
    pack_dir: &Path,
    validations_path: Option<&str>,
) -> Result<Vec<PackValidation>, PackLoadError> {
    let Some(relative) = validations_path else {
        return Ok(vec![]);
    };
    let path = pack_dir.join(relative);
    if !path.exists() {
        return Ok(vec![]);
    }
    let raw = fs::read_to_string(&path).map_err(io_err)?;
    parse_validations(&raw, &path.display().to_string())
}

/// Parses and semantically checks a pack's validations file.
///
/// Every problem here is a load error, surfaced by the compiler as
/// `E4004_MISSING_PACK` — a malformed pack produces a clean diagnostic rather
/// than silently skipping checks or panicking.
fn parse_validations(raw: &str, source: &str) -> Result<Vec<PackValidation>, PackLoadError> {
    let parsed: ValidationsFile = toml::from_str(raw).map_err(|err| PackLoadError {
        message: format!("Failed to parse validations '{source}': {err}"),
    })?;

    if let Some(version) = parsed.schema_version {
        if version != 1 {
            return Err(PackLoadError {
                message: format!(
                    "Validations '{source}': unsupported schema_version {version} (expected 1)."
                ),
            });
        }
    }

    let mut seen: Vec<(String, String)> = Vec::new();
    for validation in &parsed.validations {
        let fail = |message: String| PackLoadError {
            message: format!("Validations '{source}': {message}"),
        };

        if validation.contract.is_some() != validation.contracts.is_empty() {
            return Err(fail(format!(
                "validation '{}' must set exactly one of `contract` or `contracts`.",
                validation.code
            )));
        }
        if validation.code.is_empty() {
            return Err(fail("a validation is missing `code`.".to_string()));
        }
        if let Some(prefix) = &parsed.code_prefix {
            if !validation.code.starts_with(prefix.as_str()) {
                return Err(fail(format!(
                    "code '{}' does not start with the pack's reserved prefix '{prefix}'.",
                    validation.code
                )));
            }
        }

        match validation.check {
            ValidationCheck::TermPresent | ValidationCheck::TermNumber => {
                if validation.term.is_none() {
                    return Err(fail(format!(
                        "validation '{}' requires `term`.",
                        validation.code
                    )));
                }
            }
            ValidationCheck::TermEnum => {
                if validation.term.is_none() || validation.values.is_empty() {
                    return Err(fail(format!(
                        "validation '{}' requires `term` and a non-empty `values`.",
                        validation.code
                    )));
                }
            }
            ValidationCheck::AnyTermPresent => {
                if validation.terms.is_empty() {
                    return Err(fail(format!(
                        "validation '{}' requires a non-empty `terms`.",
                        validation.code
                    )));
                }
            }
            ValidationCheck::TermCompare => {
                if validation.left.is_none()
                    || validation.right.is_none()
                    || validation.op.is_none()
                {
                    return Err(fail(format!(
                        "validation '{}' requires `left`, `op`, and `right`.",
                        validation.code
                    )));
                }
            }
            ValidationCheck::TermRangeWithinTimeline => {}
        }

        let has_bounds = validation.min.is_some()
            || validation.max.is_some()
            || validation.exclusive_min.is_some()
            || validation.exclusive_max.is_some();
        if has_bounds && validation.check != ValidationCheck::TermNumber {
            return Err(fail(format!(
                "validation '{}' declares bounds, which only apply to check 'term_number'.",
                validation.code
            )));
        }
        if let (Some(min), Some(max)) = (validation.min, validation.max) {
            if min > max {
                return Err(fail(format!(
                    "validation '{}' has min {min} greater than max {max}.",
                    validation.code
                )));
            }
        }

        for contract in validation.contract_names() {
            let key = (contract.to_string(), validation.code.clone());
            if seen.contains(&key) {
                return Err(fail(format!(
                    "duplicate code '{}' for contract '{contract}'.",
                    validation.code
                )));
            }
            seen.push(key);
        }
    }

    let mut validations = parsed.validations;
    // Deterministic order: diagnostics are sorted downstream, but a stable
    // load order keeps behaviour reproducible.
    validations.sort_by(|a, b| {
        a.contract_names()
            .cmp(&b.contract_names())
            .then_with(|| a.code.cmp(&b.code))
    });
    Ok(validations)
}

fn load_metric_specs(
    pack_dir: &Path,
    metrics_path: Option<&str>,
) -> Result<Vec<MetricSpec>, PackLoadError> {
    let Some(relative) = metrics_path else {
        return Ok(vec![]);
    };
    let path = pack_dir.join(relative);
    let raw = fs::read_to_string(&path).map_err(io_err)?;
    parse_metric_specs(&raw, &path.display().to_string())
}

fn parse_metric_specs(raw: &str, source: &str) -> Result<Vec<MetricSpec>, PackLoadError> {
    let parsed: MetricsFile = toml::from_str(raw).map_err(|err| PackLoadError {
        message: format!("Failed to parse metrics '{source}': {err}"),
    })?;
    for spec in &parsed.metrics {
        match spec.op.as_str() {
            "sum" | "negated_sum" => {}
            "wal_years" => {
                if spec.numerator_streams.is_empty() {
                    return Err(PackLoadError {
                        message: format!(
                            "Metric '{}': op 'wal_years' requires numerator_streams.",
                            spec.id
                        ),
                    });
                }
                if spec.kind != "number" {
                    return Err(PackLoadError {
                        message: format!(
                            "Metric '{}': op 'wal_years' requires kind 'number'.",
                            spec.id
                        ),
                    });
                }
            }
            "ratio" => {
                if spec.numerator_metric.is_none() || spec.denominator_metric.is_none() {
                    return Err(PackLoadError {
                        message: format!(
                            "Metric '{}': op 'ratio' requires numerator_metric and denominator_metric.",
                            spec.id
                        ),
                    });
                }
            }
            other => {
                return Err(PackLoadError {
                    message: format!(
                        "Metric '{}': unknown op '{other}' (expected sum, negated_sum, ratio, wal_years).",
                        spec.id
                    ),
                });
            }
        }
        if !matches!(spec.kind.as_str(), "money" | "number") {
            return Err(PackLoadError {
                message: format!(
                    "Metric '{}': unknown kind '{}' (expected money or number).",
                    spec.id, spec.kind
                ),
            });
        }
    }
    Ok(parsed.metrics)
}

fn load_lowering_rules(
    pack_dir: &Path,
    lowering_path: Option<&str>,
) -> Result<Vec<LoweringRule>, PackLoadError> {
    let Some(relative) = lowering_path else {
        return Ok(vec![]);
    };
    let path = pack_dir.join(relative);
    let raw = fs::read_to_string(&path).map_err(io_err)?;
    parse_lowering_rules(&raw, &path.display().to_string())
}

fn parse_lowering_rules(raw: &str, source: &str) -> Result<Vec<LoweringRule>, PackLoadError> {
    let parsed: LoweringFile = toml::from_str(raw).map_err(|err| PackLoadError {
        message: format!("Failed to parse lowering rules '{source}': {err}"),
    })?;
    for rule in &parsed.rules {
        // Templated stream names ({{contract.*}}) are validated post-expansion
        // by the compiler.
        if !rule.stream_name.contains("{{") && !is_qualified_name(&rule.stream_name) {
            return Err(PackLoadError {
                message: format!(
                    "Lowering rule '{}' has invalid stream_name '{}'; expected dotted qualified name.",
                    rule.id, rule.stream_name
                ),
            });
        }
        if !(rule.owner_entity.is_empty()
            || rule.owner_entity == "${subject}"
            || is_qualified_name(&rule.owner_entity))
        {
            return Err(PackLoadError {
                message: format!(
                    "Lowering rule '{}' has invalid owner_entity '{}'; expected '${{subject}}' or dotted qualified entity symbol.",
                    rule.id, rule.owner_entity
                ),
            });
        }
    }
    Ok(parsed.rules)
}

fn load_templates(
    pack_dir: &Path,
    templates_path: Option<&str>,
) -> Result<Vec<PackTemplate>, PackLoadError> {
    let Some(relative) = templates_path else {
        return Ok(vec![]);
    };
    let path = pack_dir.join(relative);
    let raw = fs::read_to_string(&path).map_err(io_err)?;
    parse_templates(&raw, &path.display().to_string())
}

fn parse_templates(raw: &str, source: &str) -> Result<Vec<PackTemplate>, PackLoadError> {
    let mut parsed: TemplateFile = toml::from_str(raw).map_err(|err| PackLoadError {
        message: format!("Failed to parse templates '{source}': {err}"),
    })?;
    parsed.templates.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(parsed.templates)
}

fn expand_template_body(template: &PackTemplate, params: &BTreeMap<String, String>) -> String {
    let mut output = String::with_capacity(template.body.len());
    let chars = template.body.chars().collect::<Vec<_>>();
    let mut idx = 0usize;
    while idx < chars.len() {
        if chars[idx] == '$' && idx + 1 < chars.len() && chars[idx + 1] == '{' {
            idx += 2;
            let mut key = String::new();
            while idx < chars.len() && chars[idx] != '}' {
                key.push(chars[idx]);
                idx += 1;
            }
            if idx < chars.len() && chars[idx] == '}' {
                idx += 1;
            }
            let value = params
                .get(&key)
                .or_else(|| template.defaults.get(&key))
                .cloned()
                .unwrap_or_default();
            output.push_str(&value);
        } else {
            output.push(chars[idx]);
            idx += 1;
        }
    }
    output
}

pub fn render_template(template: &PackTemplate, params: &BTreeMap<String, String>) -> String {
    expand_template_body(template, params)
}

fn io_err(err: std::io::Error) -> PackLoadError {
    PackLoadError {
        message: format!("I/O error while loading packs: {err}"),
    }
}

fn is_qualified_name(value: &str) -> bool {
    let mut parts = value.split('.');
    let Some(first) = parts.next() else {
        return false;
    };
    if !is_ident_segment(first) {
        return false;
    }
    let mut count = 1usize;
    for part in parts {
        if !is_ident_segment(part) {
            return false;
        }
        count += 1;
    }
    count >= 2
}

fn is_ident_segment(segment: &str) -> bool {
    let mut chars = segment.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first.is_ascii_alphabetic() || first == '_') {
        return false;
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct AliasFile {
    #[serde(default)]
    aliases: BTreeMap<String, String>,
}

#[cfg(all(test, feature = "embedded-packs"))]
mod embedded_tests {
    use super::*;

    #[test]
    fn embedded_matches_filesystem_packs() {
        let embedded = PackRegistry::load_embedded().expect("embedded packs load");
        let fs_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../packs")
            .canonicalize()
            .expect("packs dir");
        let from_fs = PackRegistry::load_from_dir(&fs_root).expect("fs packs load");
        for name in ["cre", "opco"] {
            assert_eq!(
                embedded.lowering_rules(name),
                from_fs.lowering_rules(name),
                "{name} rules"
            );
            assert_eq!(
                embedded.metric_specs(name),
                from_fs.metric_specs(name),
                "{name} metrics"
            );
            assert!(
                !embedded.lowering_rules(name).is_empty(),
                "{name} non-empty"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn loads_pack_registry_from_filesystem() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("cfdl-pack-test-{unique}"));
        let pack_dir = root.join("testpack");
        let lowering_dir = pack_dir.join("lowering");
        fs::create_dir_all(&lowering_dir).expect("create test dirs");
        fs::write(
            pack_dir.join("pack.toml"),
            r#"name = "testpack"
version = "0.1.0"
[entrypoints]
aliases = "aliases.toml"
templates = "templates.toml"
lowering = "lowering/rules.toml"
"#,
        )
        .expect("write manifest");
        fs::write(
            pack_dir.join("aliases.toml"),
            r#"[aliases]
Lease = "core.Contract"
"#,
        )
        .expect("write aliases");
        fs::write(
            pack_dir.join("templates.toml"),
            r#"[[templates]]
id = "lease.basic"
label = "Lease Basic"
kind = "contract"
body = "contract core.lease ${name} term ${term_start}..${term_end}"

[templates.defaults]
name = "lease_main"
term_start = "2026-01"
term_end = "2026-12"
"#,
        )
        .expect("write templates");
        fs::write(
            lowering_dir.join("rules.toml"),
            r#"[[rules]]
id = "rule"
contract_name = "lease_contract"
stream_name = "pack.stream"
owner_entity = "legal.borrower"
direction = "inflow"
currency = "USD"
amount_expr = "1"
schedule_kind = "every"
schedule_from = "2026-01"
schedule_to = "2026-12"
"#,
        )
        .expect("write lowering");

        let registry = PackRegistry::load_from_dir(&root).expect("registry");
        assert!(registry.active_pack("testpack", "0.1.0").is_some());
        assert_eq!(
            registry.lookup_alias("testpack", "Lease"),
            Some("core.Contract")
        );
        assert_eq!(registry.templates("testpack").len(), 1);
        assert_eq!(registry.lowering_rules("testpack").len(), 1);
        let expansion = registry
            .expand_template(
                "testpack",
                TemplateExpansionRequest {
                    template: "lease.basic".to_string(),
                    params: BTreeMap::from([("name".to_string(), "lease_001".to_string())]),
                },
            )
            .expect("template expansion");
        assert_eq!(
            expansion.generated_nodes,
            vec!["contract core.lease lease_001 term 2026-01..2026-12".to_string()]
        );

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn rejects_non_qualified_stream_name_in_lowering_rule() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("cfdl-pack-test-invalid-stream-{unique}"));
        let pack_dir = root.join("testpack");
        let lowering_dir = pack_dir.join("lowering");
        fs::create_dir_all(&lowering_dir).expect("create dirs");
        fs::write(
            pack_dir.join("pack.toml"),
            r#"name = "testpack"
version = "0.1.0"
[entrypoints]
lowering = "lowering/rules.toml"
"#,
        )
        .expect("write manifest");
        fs::write(
            lowering_dir.join("rules.toml"),
            r#"[[rules]]
id = "rule_bad"
contract_name = "lease_contract"
stream_name = "flatname"
owner_entity = "legal.borrower"
direction = "inflow"
currency = "USD"
amount_expr = "1"
schedule_kind = "every"
schedule_from = "2026-01"
schedule_to = "2026-12"
"#,
        )
        .expect("write lowering");

        let err = PackRegistry::load_from_dir(&root).expect_err("invalid lowering");
        assert!(err.message.contains("invalid stream_name"));
        let _ = fs::remove_dir_all(&root);
    }
}

#[cfg(test)]
mod validation_tests {
    use super::*;

    const VALID: &str = r#"
schema_version = 1
code_prefix = "E6"

[[validations]]
contract = "cre.lease"
code = "E6001_CRE_LEASE_MISSING_BASE_RENT"
message = "missing base_rent"
check = "term_present"
term = "base_rent"

[[validations]]
contracts = ["cre.ops_revenue", "cre.ops_expense"]
code = "E6020_CRE_OPS_MISSING_AMOUNT"
message = "missing amount"
check = "term_number"
term = "amount"
when = "present"
exclusive_min = 0.0
"#;

    #[test]
    fn parses_a_valid_file() {
        let parsed = parse_validations(VALID, "test").expect("parses");
        assert_eq!(parsed.len(), 2);
        assert!(parsed
            .iter()
            .any(|v| v.contract_names() == vec!["cre.lease"]));
    }

    #[test]
    fn rejects_unknown_fields() {
        let raw = VALID.replace("exclusive_min", "exclusiv_min");
        let err = parse_validations(&raw, "test").expect_err("typo must not be silently ignored");
        assert!(err.message.contains("Failed to parse validations"));
    }

    #[test]
    fn rejects_code_outside_the_reserved_prefix() {
        let raw = VALID.replace("E6001_CRE_LEASE_MISSING_BASE_RENT", "E7001_WRONG_PACK");
        let err = parse_validations(&raw, "test").expect_err("prefix is enforced");
        assert!(err.message.contains("reserved prefix"));
    }

    #[test]
    fn rejects_inverted_bounds() {
        let raw = r#"
code_prefix = "E6"
[[validations]]
contract = "c.x"
code = "E6099_X"
message = "m"
check = "term_number"
term = "t"
min = 10.0
max = 1.0
"#;
        let err = parse_validations(raw, "test").expect_err("min > max is rejected");
        assert!(err.message.contains("greater than max"));
    }

    #[test]
    fn rejects_bounds_on_non_numeric_checks() {
        let raw = r#"
[[validations]]
contract = "c.x"
code = "X1"
message = "m"
check = "term_present"
term = "t"
min = 1.0
"#;
        let err = parse_validations(raw, "test").expect_err("bounds need term_number");
        assert!(err.message.contains("only apply to check 'term_number'"));
    }

    #[test]
    fn rejects_both_or_neither_contract_forms() {
        for body in [
            r#"contract = "a.b"
contracts = ["c.d"]"#,
            "",
        ] {
            let raw = format!(
                r#"
[[validations]]
{body}
code = "X1"
message = "m"
check = "term_present"
term = "t"
"#
            );
            let err = parse_validations(&raw, "test").expect_err("exactly one form required");
            assert!(err.message.contains("exactly one"));
        }
    }

    #[test]
    fn rejects_duplicate_code_for_a_contract() {
        let raw = format!(
            "{VALID}\n{}",
            VALID
                .replace("schema_version = 1", "")
                .replace("code_prefix = \"E6\"", "")
        );
        let err = parse_validations(&raw, "test").expect_err("duplicates are rejected");
        assert!(err.message.contains("duplicate code"));
    }

    #[test]
    fn rejects_unsupported_schema_version() {
        let raw = VALID.replace("schema_version = 1", "schema_version = 2");
        let err = parse_validations(&raw, "test").expect_err("version is checked");
        assert!(err.message.contains("unsupported schema_version"));
    }

    #[test]
    fn instance_matching_covers_suffixed_contracts() {
        let raw = r#"
[[validations]]
contract = "cre.lease_unit"
match = "instance"
code = "X1"
message = "m"
check = "term_present"
term = "rent_year"
"#;
        let parsed = parse_validations(raw, "test").expect("parses");
        let v = &parsed[0];
        assert!(v.applies_to("cre.lease_unit"));
        assert!(v.applies_to("cre.lease_unit.tenant_a"));
        assert!(!v.applies_to("cre.lease_unit_other"));
        assert!(!v.applies_to("cre.lease"));
    }

    #[test]
    fn exact_matching_is_the_default() {
        let parsed = parse_validations(VALID, "test").expect("parses");
        let lease = parsed
            .iter()
            .find(|v| v.code.starts_with("E6001"))
            .expect("lease rule");
        assert!(lease.applies_to("cre.lease"));
        assert!(!lease.applies_to("cre.lease.primary"));
    }
}
