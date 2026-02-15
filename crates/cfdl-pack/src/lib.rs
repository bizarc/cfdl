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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateExpansion {
    pub generated_nodes: Vec<String>,
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
            let lowering_rules =
                load_lowering_rules(&pack_dir, manifest.entrypoints.lowering.as_deref())?;

            packs.insert(
                manifest.name.clone(),
                LoadedPack {
                    manifest,
                    aliases,
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

    pub fn expand_template_stub(
        &self,
        _pack_name: &str,
        _request: TemplateExpansionRequest,
    ) -> TemplateExpansion {
        TemplateExpansion {
            generated_nodes: vec![],
        }
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
    Ok(parsed.rules)
}

fn io_err(err: std::io::Error) -> PackLoadError {
    PackLoadError {
        message: format!("I/O error while loading packs: {err}"),
    }
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
            lowering_dir.join("rules.toml"),
            r#"[[rules]]
id = "rule"
contract_name = "lease_contract"
stream_name = "s"
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
        assert_eq!(registry.lowering_rules("testpack").len(), 1);

        let _ = fs::remove_dir_all(&root);
    }
}
