//! `skeleton`: a pack + contract types -> a minimal valid model to grow.
//!
//! Assembled from the pack's own `templates.toml` (the same snippets the LSP
//! offers), wrapped in a model header, and compiled before it is returned —
//! the response says whether the skeleton is valid, so an agent starts from a
//! model the verifier has already accepted.

use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::Defaults;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SkeletonParams {
    /// The domain pack (e.g. `cre`, `credit`, `energy`, `opco`).
    pub pack: String,
    /// Contract types or template ids to include (e.g. `lease_unit`,
    /// `cre.opex_line.property_tax`). Default: the pack's first template.
    #[serde(default)]
    pub contract_types: Option<Vec<String>>,
    /// Model calendar (default `monthly`).
    #[serde(default)]
    pub calendar: Option<String>,
    /// Grid length in periods (default 12).
    #[serde(default)]
    pub periods: Option<u32>,
    /// First period, `YYYY-MM` (default `2026-01`).
    #[serde(default)]
    pub start: Option<String>,
    /// `${placeholder}` values for the templates, overriding their defaults.
    #[serde(default)]
    pub template_params: Option<BTreeMap<String, String>>,
    /// Pack directory override (as in `compile`).
    #[serde(default)]
    pub packs_dir: Option<String>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct SkeletonResult {
    /// The model source. Grow it; do not start from prose.
    pub model: String,
    /// Whether the skeleton compiles as returned.
    pub ok: bool,
    /// Compile diagnostics when it does not (unfilled `${placeholders}` land here).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub diagnostics: Vec<serde_json::Value>,
    pub templates_used: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
}

pub fn skeleton(params: &SkeletonParams, defaults: &Defaults) -> Result<SkeletonResult, String> {
    let registry = super::load_registry(
        super::resolve_packs_dir(params.packs_dir.as_deref(), defaults).as_deref(),
    )?;
    let Some(pack) = registry.pack(&params.pack) else {
        let available: Vec<String> = registry
            .list()
            .iter()
            .map(|p| p.manifest.name.clone())
            .collect();
        return Err(format!(
            "no pack '{}'; available: {}",
            params.pack,
            available.join(", ")
        ));
    };
    if pack.templates.is_empty() {
        return Err(format!(
            "pack '{}' ships no templates to build a skeleton from",
            params.pack
        ));
    }

    let calendar = params.calendar.as_deref().unwrap_or("monthly");
    let periods = params.periods.unwrap_or(12);
    let start = params.start.as_deref().unwrap_or("2026-01");
    let mut notes = Vec::new();

    // Pick templates: by id, by suffix after the pack prefix, or by leading
    // type segment; default to the pack's first template.
    let wanted: Vec<String> = match &params.contract_types {
        Some(types) if !types.is_empty() => types.clone(),
        _ => {
            let first = pack.templates[0].id.clone();
            notes.push(format!("no contract_types given; using template '{first}'"));
            vec![first]
        }
    };
    let mut chosen = Vec::new();
    for want in &wanted {
        let found = pack.templates.iter().find(|t| {
            let after_pack =
                t.id.strip_prefix(&format!("{}.", params.pack))
                    .unwrap_or(&t.id);
            t.id == *want || after_pack == want || after_pack.starts_with(&format!("{want}."))
        });
        match found {
            Some(template) => chosen.push(template.clone()),
            None => {
                let available: Vec<&str> = pack.templates.iter().map(|t| t.id.as_str()).collect();
                return Err(format!(
                    "no template for '{want}' in pack '{}'; available: {}",
                    params.pack,
                    available.join(", ")
                ));
            }
        }
    }

    // The subject entity: the pack's first asset type, else the language base.
    let entity_type = pack
        .ontology
        .entities
        .iter()
        .find(|e| e.family == "asset")
        .map(|e| e.type_id.clone())
        .unwrap_or_else(|| "Asset.Real".to_string());

    // Term defaults spanning the grid, unless the caller or the template says.
    let term_end = advance_ym(start, periods.saturating_sub(1), calendar);
    let mut fills: BTreeMap<String, String> = BTreeMap::new();
    fills.insert("term_start".to_string(), start.to_string());
    fills.insert("term_end".to_string(), term_end.clone());
    if let Some(overrides) = &params.template_params {
        fills.extend(overrides.clone());
    }

    let mut model = format!(
        "version 0.1\nmodel \"{pack}-skeleton\"\nuse pack \"{pack}\" version \"{version}\"\ntime calendar {calendar} from {start} for {periods}\n\nentity asset subject : {entity_type}\n",
        pack = params.pack,
        version = pack.manifest.version,
    );
    let mut templates_used = Vec::new();
    for template in &chosen {
        // Template defaults win only where the caller did not fill a value.
        let mut template_fills = fills.clone();
        for (key, value) in &template.defaults {
            template_fills
                .entry(key.clone())
                .or_insert_with(|| value.clone());
        }
        let mut body = cfdl_pack::render_template(template, &template_fills);
        // The snippet bodies omit the subject binding the LSP inserts in
        // context; a standalone model needs it on the contract header.
        if let Some(first_line_end) = body.find('\n') {
            let header = &body[..first_line_end];
            if header.trim_start().starts_with("contract ") && !header.contains(" on ") {
                let bound = header.replacen(" {", " on entity asset.subject {", 1);
                body = format!("{bound}{}", &body[first_line_end..]);
            }
        }
        model.push('\n');
        model.push_str(body.trim_end());
        model.push('\n');
        templates_used.push(template.id.clone());
    }

    // Compile the skeleton before handing it over.
    let mut files = BTreeMap::new();
    files.insert("model.cfdl".to_string(), model.clone());
    let compiled = super::compile::compile_ir(
        None,
        Some(&files),
        None,
        params.packs_dir.as_deref(),
        defaults,
    )?;
    let (ok, diagnostics) = match compiled {
        Ok(_) => (true, Vec::new()),
        Err(diags) => (
            false,
            diags
                .iter()
                .map(|d| serde_json::to_value(d).unwrap_or(serde_json::Value::Null))
                .collect(),
        ),
    };
    if !ok {
        notes.push(
            "the skeleton does not compile as-is; fill the remaining ${placeholders} \
             (see `template_params`) and re-check with `compile`"
                .to_string(),
        );
    }
    Ok(SkeletonResult {
        model,
        ok,
        diagnostics,
        templates_used,
        notes,
    })
}

/// `YYYY-MM` plus N periods on the given calendar (monthly/quarterly/annual).
fn advance_ym(start: &str, offset: u32, calendar: &str) -> String {
    let mut parts = start.splitn(2, '-');
    let year: i32 = parts.next().and_then(|y| y.parse().ok()).unwrap_or(2026);
    let month: u32 = parts.next().and_then(|m| m.parse().ok()).unwrap_or(1);
    let step = match calendar {
        "annual" => 12,
        "quarterly" => 3,
        _ => 1,
    };
    let total = (month - 1) + offset * step;
    format!("{}-{:02}", year + (total / 12) as i32, (total % 12) + 1)
}
