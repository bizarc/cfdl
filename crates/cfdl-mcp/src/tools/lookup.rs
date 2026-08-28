//! `lookup`: a term -> its terminology/glossary entry; a pack -> its contract
//! roster, with benchmark coverage re-derived from `contract <pack>.<type>`
//! declarations (the measurement docs/13 §7.3 asks for, not the prose table).

use std::path::Path;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// The terminology register ships inside the binary, so `lookup` answers
/// anywhere — the same source `docs/glossary.md` is generated from.
const TERMINOLOGY: &str = include_str!("../../../../docs/terminology.toml");

#[derive(Debug, Default, Deserialize, JsonSchema)]
pub struct LookupParams {
    /// A term to look up in the terminology register (case-insensitive;
    /// exact match first, then substring matches).
    #[serde(default)]
    pub term: Option<String>,
    /// A pack name (e.g. `cre`, `credit`, `energy`, `opco`) for its roster.
    #[serde(default)]
    pub pack: Option<String>,
    /// Pack directory override (as in `compile`).
    #[serde(default)]
    pub packs_dir: Option<String>,
    /// Benchmarks directory for contract-coverage scanning. Default: the
    /// server's benchmarks directory; absent, coverage is omitted.
    #[serde(default)]
    pub benchmarks_dir: Option<String>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct TermEntry {
    /// `technical_name` | `technical_verb` | `preferred`
    pub kind: String,
    pub term: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub definition: Option<String>,
    /// For `preferred` entries: the spellings to avoid.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub instead_of: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct ContractInfo {
    pub type_id: String,
    /// The lowering rule this type binds to. Absent means an election — an
    /// option the engine resolves rather than a rule a pack lowers.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contract_name: Option<String>,
    pub election: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub parties: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Benchmark cases declaring `contract <pack>.<type>` — the §7.3
    /// coverage measurement. Absent (not empty) when no benchmarks directory
    /// was available to scan.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exercised_by: Option<Vec<String>>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct TemplateInfo {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct PackInfo {
    pub name: String,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Model calendars this pack's rules lower correctly on (empty = all).
    pub cadences: Vec<String>,
    pub contracts: Vec<ContractInfo>,
    pub templates: Vec<TemplateInfo>,
    /// Domain metric output keys (e.g. `domain.cre.noi`).
    pub metrics: Vec<String>,
    pub validations: usize,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct LookupResult {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub terms: Vec<TermEntry>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pack: Option<PackInfo>,
    /// When neither `term` nor `pack` was given: the packs available.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub packs: Vec<String>,
}

pub fn lookup(params: &LookupParams, defaults: &super::Defaults) -> Result<LookupResult, String> {
    let mut result = LookupResult {
        terms: Vec::new(),
        pack: None,
        packs: Vec::new(),
    };
    if let Some(term) = &params.term {
        result.terms = find_terms(term)?;
    }
    if params.pack.is_some() || params.term.is_none() {
        let registry = super::load_registry(
            super::resolve_packs_dir(params.packs_dir.as_deref(), defaults).as_deref(),
        )?;
        match &params.pack {
            Some(pack_name) => {
                result.pack = Some(pack_info(&registry, pack_name, params, defaults)?);
            }
            None if params.term.is_none() => {
                result.packs = registry
                    .list()
                    .iter()
                    .map(|p| p.manifest.name.clone())
                    .collect();
            }
            None => {}
        }
    }
    Ok(result)
}

fn find_terms(term: &str) -> Result<Vec<TermEntry>, String> {
    let register: toml::Value = TERMINOLOGY
        .parse()
        .map_err(|err| format!("terminology register: {err}"))?;
    let needle = term.to_lowercase();
    let mut exact = Vec::new();
    let mut partial = Vec::new();

    let mut consider = |entry: TermEntry, key: &str| {
        let hay = key.to_lowercase();
        if hay == needle {
            exact.push(entry);
        } else if hay.contains(&needle) {
            partial.push(entry);
        }
    };

    for kind in ["technical_name", "technical_verb"] {
        for item in register
            .get(kind)
            .and_then(|v| v.as_array())
            .into_iter()
            .flatten()
        {
            let Some(term) = item.get("term").and_then(|t| t.as_str()) else {
                continue;
            };
            consider(
                TermEntry {
                    kind: kind.to_string(),
                    term: term.to_string(),
                    category: item
                        .get("category")
                        .and_then(|c| c.as_str())
                        .map(str::to_string),
                    definition: item
                        .get("definition")
                        .and_then(|d| d.as_str())
                        .map(str::to_string),
                    instead_of: Vec::new(),
                    note: item
                        .get("note")
                        .and_then(|n| n.as_str())
                        .map(str::to_string),
                },
                term,
            );
        }
    }
    for item in register
        .get("preferred")
        .and_then(|v| v.as_array())
        .into_iter()
        .flatten()
    {
        let Some(preferred) = item.get("use").and_then(|u| u.as_str()) else {
            continue;
        };
        let instead_of: Vec<String> = item
            .get("instead_of")
            .and_then(|i| i.as_array())
            .into_iter()
            .flatten()
            .filter_map(|s| s.as_str().map(str::to_string))
            .collect();
        let entry = TermEntry {
            kind: "preferred".to_string(),
            term: preferred.to_string(),
            category: None,
            definition: None,
            instead_of: instead_of.clone(),
            note: item
                .get("note")
                .and_then(|n| n.as_str())
                .map(str::to_string),
        };
        // A discouraged spelling should find its preferred form.
        let hay = format!("{} {}", preferred, instead_of.join(" ")).to_lowercase();
        if preferred.to_lowercase() == needle
            || instead_of.iter().any(|i| i.to_lowercase() == needle)
        {
            exact.push(entry);
        } else if hay.contains(&needle) {
            partial.push(entry);
        }
    }
    exact.extend(partial);
    exact.truncate(20);
    Ok(exact)
}

fn pack_info(
    registry: &cfdl_pack::PackRegistry,
    pack_name: &str,
    params: &LookupParams,
    defaults: &super::Defaults,
) -> Result<PackInfo, String> {
    let Some(pack) = registry.pack(pack_name) else {
        let available: Vec<String> = registry
            .list()
            .iter()
            .map(|p| p.manifest.name.clone())
            .collect();
        return Err(format!(
            "no pack '{pack_name}'; available: {}",
            available.join(", ")
        ));
    };
    let coverage = params
        .benchmarks_dir
        .as_ref()
        .map(std::path::PathBuf::from)
        .or_else(|| defaults.benchmarks_dir.clone())
        .map(|dir| scan_contract_coverage(&dir, pack_name))
        .transpose()?;

    let contracts = pack
        .ontology
        .contracts
        .iter()
        .map(|contract| ContractInfo {
            type_id: contract.type_id.clone(),
            contract_name: contract.contract_name.clone(),
            election: contract.contract_name.is_none(),
            parties: contract.parties.clone(),
            description: contract.description.clone(),
            // A declaration `contract cre.lease_unit.tenant_a` exercises the
            // type whose contract_name is `cre.lease_unit`: instance names
            // extend the contract name with further segments.
            exercised_by: coverage.as_ref().map(|cases| {
                let Some(name) = &contract.contract_name else {
                    return Vec::new();
                };
                cases
                    .iter()
                    .filter(|(declared, _)| {
                        declared == name || declared.starts_with(&format!("{name}."))
                    })
                    .map(|(_, case)| case.clone())
                    .collect::<Vec<_>>()
                    .into_iter()
                    .fold(Vec::new(), |mut acc, case| {
                        if !acc.contains(&case) {
                            acc.push(case);
                        }
                        acc
                    })
            }),
        })
        .collect();

    Ok(PackInfo {
        name: pack.manifest.name.clone(),
        version: pack.manifest.version.clone(),
        description: pack.manifest.description.clone(),
        cadences: pack.manifest.cadences.clone(),
        contracts,
        templates: pack
            .templates
            .iter()
            .map(|t| TemplateInfo {
                id: t.id.clone(),
                label: t.label.clone(),
            })
            .collect(),
        metrics: pack.metric_specs.iter().map(|m| m.id.clone()).collect(),
        validations: pack.validations.len(),
    })
}

/// Scan `benchmarks/*/*/model.cfdl` for `contract <pack>.<rest>` declarations.
/// Returns (full declared name, case id) pairs.
fn scan_contract_coverage(
    benchmarks_dir: &Path,
    pack_name: &str,
) -> Result<Vec<(String, String)>, String> {
    let mut declarations = Vec::new();
    let prefix = format!("{pack_name}.");
    let groups = std::fs::read_dir(benchmarks_dir)
        .map_err(|err| format!("cannot read '{}': {err}", benchmarks_dir.display()))?;
    for group in groups.flatten() {
        if !group.path().is_dir() {
            continue;
        }
        let Ok(cases) = std::fs::read_dir(group.path()) else {
            continue;
        };
        for case in cases.flatten() {
            let model = case.path().join("model.cfdl");
            let Ok(source) = std::fs::read_to_string(&model) else {
                continue;
            };
            let case_id = format!(
                "{}/{}",
                group.file_name().to_string_lossy(),
                case.file_name().to_string_lossy()
            );
            for line in source.lines() {
                let line = line.trim_start();
                let Some(rest) = line.strip_prefix("contract ") else {
                    continue;
                };
                let declared = rest.split_whitespace().next().unwrap_or("");
                if declared.starts_with(&prefix) {
                    declarations.push((declared.to_string(), case_id.clone()));
                }
            }
        }
    }
    Ok(declarations)
}
