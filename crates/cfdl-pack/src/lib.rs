use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackLoadError {
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackRegistry {
    packs: BTreeMap<String, LoadedPack>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedPack {
    pub manifest: PackManifest,
    pub aliases: BTreeMap<String, String>,
    pub templates: Vec<PackTemplate>,
    pub lowering_rules: Vec<LoweringRule>,
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
    pub currency: String,
    pub amount_cel: String,
    pub schedule_kind: String,
    pub schedule_from: String,
    pub schedule_to: String,
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

            packs.insert(
                manifest.name.clone(),
                LoadedPack {
                    manifest,
                    aliases,
                    templates,
                    lowering_rules,
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
    let parsed: AliasFile = toml::from_str(&raw).map_err(|err| PackLoadError {
        message: format!("Failed to parse aliases '{}': {err}", path.display()),
    })?;
    Ok(parsed.aliases)
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
    let parsed: LoweringFile = toml::from_str(&raw).map_err(|err| PackLoadError {
        message: format!("Failed to parse lowering rules '{}': {err}", path.display()),
    })?;
    for rule in &parsed.rules {
        if !is_qualified_name(&rule.stream_name) {
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
    let mut parsed: TemplateFile = toml::from_str(&raw).map_err(|err| PackLoadError {
        message: format!("Failed to parse templates '{}': {err}", path.display()),
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
amount_cel = "1"
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
amount_cel = "1"
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
