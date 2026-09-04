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
    /// A master contract type (e.g. `Contract.Debt`, `Contract.Security`)
    /// for its roster — roles, fields, lines, side — read from the language
    /// base, or from the named pack's view when `pack` is given too. A
    /// master a pack does not refine is still readable this way.
    #[serde(default)]
    pub master: Option<String>,
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

/// A role of a contract type, resolved through its master chain (docs/40 §5).
#[derive(Debug, Serialize, JsonSchema)]
pub struct RoleInfo {
    /// The word a model binds — the pack's (`landlord`).
    pub name: String,
    /// The master's word for it (`lessor`); the same as `name` where the
    /// pack inherits the master's word.
    pub master: String,
    /// The agreement has no such party in this form; a model may not bind it.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub unbound: bool,
}

/// A term of a contract type — its effective roster, the masters' fields
/// included (docs/40 §3). What `terms { }` may state.
#[derive(Debug, Serialize, JsonSchema)]
pub struct FieldInfo {
    pub name: String,
    pub field_type: String,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    /// Fields sharing a group are alternatives; a contract states at least one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub one_of: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct ContractInfo {
    pub type_id: String,
    /// The lowering rule this type binds to. Absent means an election — an
    /// option the engine resolves rather than a rule a pack lowers — or a
    /// master, which is never declared.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contract_name: Option<String>,
    pub election: bool,
    /// A master: exists to be refined, never declared (docs/40 §2).
    #[serde(rename = "abstract", skip_serializing_if = "std::ops::Not::not")]
    pub is_abstract: bool,
    /// The type this one refines; absent on a master.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refines: Option<String>,
    /// The master at the root of the chain — itself for a master.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub master: Option<String>,
    /// The type's effective roles, the master's word beside the pack's.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub roles: Vec<RoleInfo>,
    /// The type's effective terms — what `terms { }` may state, and must.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub fields: Vec<FieldInfo>,
    /// The lines of cash the type produces, by role (`interest`, `rent`).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub lines: Vec<String>,
    /// Which way cash runs for the subject — `pays` or `receives` — where the
    /// type fixes it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub side: Option<String>,
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
    /// The pack's own contract types, each resolved against its master chain.
    pub contracts: Vec<ContractInfo>,
    /// The language-base masters this pack's types refine (docs/40 §4), with
    /// the fields, roles and lines every refinement inherits. What "a debt"
    /// means before any pack's word for one.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub masters: Vec<ContractInfo>,
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
    /// The master asked for, against its chain.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub master: Option<ContractInfo>,
    /// When neither `term` nor `pack` was given: the packs available.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub packs: Vec<String>,
}

pub fn lookup(params: &LookupParams, defaults: &super::Defaults) -> Result<LookupResult, String> {
    let mut result = LookupResult {
        terms: Vec::new(),
        pack: None,
        master: None,
        packs: Vec::new(),
    };
    if let Some(term) = &params.term {
        result.terms = find_terms(term)?;
    }
    if let Some(master) = &params.master {
        // The base's view, or the named pack's — which adds nothing to a
        // master but lets a pack type id be asked for by the same door.
        let merged = match &params.pack {
            Some(pack_name) => {
                let registry = super::load_registry(
                    super::resolve_packs_dir(params.packs_dir.as_deref(), defaults).as_deref(),
                )?;
                registry
                    .pack(pack_name)
                    .map(|p| p.ontology.merged_with_base())
                    .ok_or_else(|| format!("no pack '{pack_name}'"))?
            }
            None => cfdl_pack::PackOntology::language_base(),
        };
        let Some(contract) = merged.contract(master) else {
            let mut known: Vec<&str> = merged
                .contracts
                .iter()
                .filter(|c| c.is_abstract)
                .map(|c| c.type_id.as_str())
                .collect();
            known.sort_unstable();
            return Err(format!(
                "no contract type '{master}'; masters: {}",
                known.join(", ")
            ));
        };
        result.master = Some(describe_contract(&merged, contract));
    }
    if params.pack.is_some() || (params.term.is_none() && params.master.is_none()) {
        let registry = super::load_registry(
            super::resolve_packs_dir(params.packs_dir.as_deref(), defaults).as_deref(),
        )?;
        match &params.pack {
            Some(pack_name) => {
                result.pack = Some(pack_info(&registry, pack_name, params, defaults)?);
            }
            None if params.term.is_none() && params.master.is_none() => {
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

    // The pack's types are read against the merged view: a refinement's
    // roster is its master's plus its own, and the master lives in the base.
    let merged = pack.ontology.merged_with_base();
    let describe = |contract: &cfdl_pack::OntologyContract| describe_contract(&merged, contract);
    let mut master_ids: Vec<String> = pack
        .ontology
        .contracts
        .iter()
        .filter_map(|c| merged.master_of(&c.type_id))
        .filter(|m| pack.ontology.contract(m).is_none())
        .collect();
    master_ids.sort_unstable();
    master_ids.dedup();
    let masters: Vec<ContractInfo> = master_ids
        .iter()
        .filter_map(|id| merged.contract(id))
        .map(describe)
        .collect();

    let contracts = pack
        .ontology
        .contracts
        .iter()
        .map(|contract| ContractInfo {
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
            ..describe(contract)
        })
        .collect();

    Ok(PackInfo {
        name: pack.manifest.name.clone(),
        version: pack.manifest.version.clone(),
        description: pack.manifest.description.clone(),
        cadences: pack.manifest.cadences.clone(),
        contracts,
        masters,
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

/// A contract type read against its master chain (docs/40): the master,
/// the effective roster with the masters' fields inherited, each role
/// beside the master's word, the lines and the side.
fn describe_contract(
    merged: &cfdl_pack::PackOntology,
    contract: &cfdl_pack::OntologyContract,
) -> ContractInfo {
    ContractInfo {
        type_id: contract.type_id.clone(),
        contract_name: contract.contract_name.clone(),
        election: !contract.is_abstract && contract.contract_name.is_none(),
        is_abstract: contract.is_abstract,
        refines: contract.refines.clone(),
        master: merged.master_of(&contract.type_id),
        roles: merged
            .effective_roles(&contract.type_id)
            .into_iter()
            .map(|r| RoleInfo {
                name: r.name,
                master: r.master,
                unbound: r.unbound,
            })
            .collect(),
        fields: merged
            .effective_fields(&contract.type_id)
            .into_iter()
            .map(|f| FieldInfo {
                name: f.name,
                field_type: f.field_type,
                required: f.required,
                unit: f.unit,
                one_of: f.one_of,
                description: f.description,
            })
            .collect(),
        lines: merged
            .effective_lines(&contract.type_id)
            .into_iter()
            .map(|l| {
                if l.allocated {
                    format!("{} (allocated)", l.name)
                } else if l.optional {
                    format!("{} (optional)", l.name)
                } else {
                    l.name
                }
            })
            .collect(),
        side: merged.effective_side(&contract.type_id),
        description: contract.description.clone(),
        exercised_by: None,
    }
}
