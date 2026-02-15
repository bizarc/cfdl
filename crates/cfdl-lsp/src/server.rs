use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use cfdl_compile::{CompileOptions, Diagnostic as CfdlDiagnostic, Span as CfdlSpan};
use lsp_types::{
    Diagnostic, DiagnosticSeverity, DidChangeTextDocumentParams, DidCloseTextDocumentParams,
    DidOpenTextDocumentParams, InitializeParams, InitializeResult, MessageType, NumberOrString,
    Position, Range, ServerCapabilities, TextDocumentSyncCapability, TextDocumentSyncKind, Url,
};
use tokio::sync::RwLock;
use tower_lsp::{jsonrpc::Result, Client, LanguageServer};

#[derive(Debug, Default)]
pub struct DocumentStore {
    docs: HashMap<Url, String>,
}

impl DocumentStore {
    pub fn open(&mut self, uri: Url, text: String) {
        self.docs.insert(uri, text);
    }

    pub fn change_full(&mut self, uri: &Url, text: String) {
        if let Some(existing) = self.docs.get_mut(uri) {
            *existing = text;
        }
    }

    pub fn close(&mut self, uri: &Url) {
        self.docs.remove(uri);
    }

    #[cfg(test)]
    pub fn get(&self, uri: &Url) -> Option<&str> {
        self.docs.get(uri).map(String::as_str)
    }
}

pub struct Backend {
    client: Client,
    docs: Arc<RwLock<DocumentStore>>,
    published_by_root: Arc<RwLock<HashMap<PathBuf, HashSet<Url>>>>,
}

impl Backend {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            docs: Arc::new(RwLock::new(DocumentStore::default())),
            published_by_root: Arc::new(RwLock::new(HashMap::default())),
        }
    }

    async fn refresh_diagnostics_for_uri(&self, source_uri: &Url) {
        let Some(model_root) = detect_model_root(source_uri) else {
            self.client
                .publish_diagnostics(source_uri.clone(), vec![], None)
                .await;
            return;
        };

        let compile_root = model_root.clone();
        let compile_result = tokio::task::spawn_blocking(move || {
            cfdl_compile::compile_to_json_with_options(&compile_root, &CompileOptions::default())
        })
        .await;

        let cfdl_diags = match compile_result {
            Ok(Ok(_)) => vec![],
            Ok(Err(diags)) => diags,
            Err(err) => {
                self.client
                    .log_message(
                        MessageType::ERROR,
                        format!("CFDL diagnostics compile task failed: {err}"),
                    )
                    .await;
                vec![]
            }
        };

        let grouped = group_diagnostics_by_uri(&model_root, source_uri, cfdl_diags);
        let next_published: HashSet<Url> = grouped
            .values()
            .map(|(uri, _)| uri.clone())
            .collect::<HashSet<_>>();

        for (_, (uri, diagnostics)) in grouped {
            self.client
                .publish_diagnostics(uri, diagnostics, None)
                .await;
        }

        let stale_uris = {
            let mut tracked = self.published_by_root.write().await;
            let previous = tracked.get(&model_root).cloned().unwrap_or_default();
            tracked.insert(model_root.clone(), next_published.clone());
            previous
                .difference(&next_published)
                .cloned()
                .collect::<Vec<_>>()
        };

        for uri in stale_uris {
            self.client.publish_diagnostics(uri, vec![], None).await;
        }

        if next_published.is_empty() {
            self.client
                .publish_diagnostics(source_uri.clone(), vec![], None)
                .await;
        }
    }
}

pub fn server_capabilities() -> ServerCapabilities {
    ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
        ..ServerCapabilities::default()
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: server_capabilities(),
            ..InitializeResult::default()
        })
    }

    async fn initialized(&self, _: lsp_types::InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "cfdl-lsp initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        let mut docs = self.docs.write().await;
        docs.open(params.text_document.uri, params.text_document.text);
        drop(docs);
        self.refresh_diagnostics_for_uri(&uri).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let Some(change) = params.content_changes.into_iter().next() else {
            return;
        };
        let uri = params.text_document.uri.clone();
        let mut docs = self.docs.write().await;
        docs.change_full(&uri, change.text);
        drop(docs);
        self.refresh_diagnostics_for_uri(&uri).await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;
        let mut docs = self.docs.write().await;
        docs.close(&uri);
        drop(docs);

        self.client
            .publish_diagnostics(uri.clone(), vec![], None)
            .await;

        let mut tracked = self.published_by_root.write().await;
        for uris in tracked.values_mut() {
            uris.remove(&uri);
        }
        tracked.retain(|_, uris| !uris.is_empty());
    }
}

fn detect_model_root(uri: &Url) -> Option<PathBuf> {
    let file_path = uri.to_file_path().ok()?;
    let mut current = if file_path.is_dir() {
        file_path
    } else {
        file_path.parent()?.to_path_buf()
    };

    loop {
        if current.join("model.cfdl").is_file() {
            return Some(current);
        }
        if !current.pop() {
            return None;
        }
    }
}

fn group_diagnostics_by_uri(
    model_root: &Path,
    source_uri: &Url,
    diagnostics: Vec<CfdlDiagnostic>,
) -> BTreeMap<String, (Url, Vec<Diagnostic>)> {
    let mut grouped: BTreeMap<String, (Url, Vec<Diagnostic>)> = BTreeMap::new();
    for diag in diagnostics {
        let target_uri = resolve_diagnostic_uri(model_root, source_uri, diag.file.as_deref());
        let key = target_uri.as_str().to_string();
        let entry = grouped
            .entry(key)
            .or_insert_with(|| (target_uri.clone(), Vec::new()));
        entry.1.push(cfdl_diagnostic_to_lsp(&diag));
    }
    grouped
}

fn resolve_diagnostic_uri(model_root: &Path, source_uri: &Url, file: Option<&str>) -> Url {
    if let Some(file) = file {
        let candidate = model_root.join(file);
        if let Ok(uri) = Url::from_file_path(candidate) {
            return uri;
        }
    }
    source_uri.clone()
}

fn cfdl_diagnostic_to_lsp(diag: &CfdlDiagnostic) -> Diagnostic {
    Diagnostic {
        range: span_to_range(diag.span.as_ref()),
        severity: Some(match diag.severity.as_str() {
            "error" => DiagnosticSeverity::ERROR,
            "warning" => DiagnosticSeverity::WARNING,
            "info" => DiagnosticSeverity::INFORMATION,
            _ => DiagnosticSeverity::HINT,
        }),
        code: Some(NumberOrString::String(diag.code.clone())),
        code_description: None,
        source: Some("cfdl".to_string()),
        message: diag.message.clone(),
        related_information: None,
        tags: None,
        data: None,
    }
}

fn span_to_range(span: Option<&CfdlSpan>) -> Range {
    let Some(span) = span else {
        return Range {
            start: Position::new(0, 0),
            end: Position::new(0, 0),
        };
    };

    Range {
        start: Position::new(
            span.start_line.saturating_sub(1),
            span.start_col.saturating_sub(1),
        ),
        end: Position::new(
            span.end_line.saturating_sub(1),
            span.end_col.saturating_sub(1),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        cfdl_diagnostic_to_lsp, detect_model_root, group_diagnostics_by_uri, server_capabilities,
        DocumentStore,
    };
    use cfdl_compile::{Diagnostic as CfdlDiagnostic, Span as CfdlSpan};
    use lsp_types::{TextDocumentSyncCapability, TextDocumentSyncKind, Url};
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn document_store_open_change_close() {
        let mut store = DocumentStore::default();
        let uri = Url::parse("file:///tmp/model.cfdl").expect("valid uri");

        store.open(uri.clone(), "one".to_string());
        assert_eq!(store.get(&uri), Some("one"));

        store.change_full(&uri, "two".to_string());
        assert_eq!(store.get(&uri), Some("two"));

        store.close(&uri);
        assert_eq!(store.get(&uri), None);
    }

    #[test]
    fn change_ignores_unknown_document() {
        let mut store = DocumentStore::default();
        let missing = Url::parse("file:///tmp/missing.cfdl").expect("valid uri");

        store.change_full(&missing, "ignored".to_string());
        assert_eq!(store.get(&missing), None);
    }

    #[test]
    fn capabilities_use_full_document_sync() {
        let capabilities = server_capabilities();
        assert_eq!(
            capabilities.text_document_sync,
            Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL))
        );
    }

    #[test]
    fn detect_model_root_from_nested_file() {
        let root = make_temp_dir("detect-root");
        let nested = root.join("nested").join("deeper");
        fs::create_dir_all(&nested).expect("create nested dirs");
        fs::write(root.join("model.cfdl"), "model {}".as_bytes()).expect("write model root");
        let file_path = nested.join("module.cfdl");
        fs::write(&file_path, "entity x {}".as_bytes()).expect("write module");

        let uri = Url::from_file_path(&file_path).expect("valid file uri");
        let detected = detect_model_root(&uri).expect("model root found");
        assert_eq!(detected, root);

        fs::remove_dir_all(detected).expect("cleanup temp dir");
    }

    #[test]
    fn diagnostic_mapping_converts_severity_code_and_span() {
        let diag = CfdlDiagnostic {
            code: "E0001_TEST".to_string(),
            severity: "warning".to_string(),
            message: "warn".to_string(),
            file: Some("model.cfdl".to_string()),
            span: Some(CfdlSpan {
                start_line: 2,
                start_col: 3,
                end_line: 4,
                end_col: 5,
            }),
            path: None,
            hint: None,
            notes: vec![],
        };

        let mapped = cfdl_diagnostic_to_lsp(&diag);
        assert_eq!(
            mapped.severity,
            Some(lsp_types::DiagnosticSeverity::WARNING)
        );
        assert_eq!(
            mapped.code,
            Some(lsp_types::NumberOrString::String("E0001_TEST".to_string()))
        );
        assert_eq!(mapped.range.start.line, 1);
        assert_eq!(mapped.range.start.character, 2);
        assert_eq!(mapped.range.end.line, 3);
        assert_eq!(mapped.range.end.character, 4);
    }

    #[test]
    fn diagnostic_mapping_uses_safe_fallback_without_span() {
        let diag = CfdlDiagnostic {
            code: "E0002_TEST".to_string(),
            severity: "error".to_string(),
            message: "err".to_string(),
            file: Some("model.cfdl".to_string()),
            span: None,
            path: None,
            hint: None,
            notes: vec![],
        };

        let mapped = cfdl_diagnostic_to_lsp(&diag);
        assert_eq!(mapped.range.start.line, 0);
        assert_eq!(mapped.range.start.character, 0);
        assert_eq!(mapped.range.end.line, 0);
        assert_eq!(mapped.range.end.character, 0);
    }

    #[test]
    fn grouped_diagnostics_are_bucketed_by_target_file() {
        let root = make_temp_dir("group-diags");
        fs::create_dir_all(root.join("imports")).expect("create imports dir");
        fs::write(root.join("model.cfdl"), "model {}".as_bytes()).expect("write model");
        fs::write(root.join("imports").join("child.cfdl"), "".as_bytes()).expect("write child");

        let source_uri = Url::from_file_path(root.join("model.cfdl")).expect("valid source uri");
        let diagnostics = vec![
            CfdlDiagnostic {
                code: "E0003_A".to_string(),
                severity: "error".to_string(),
                message: "a".to_string(),
                file: Some("imports/child.cfdl".to_string()),
                span: None,
                path: None,
                hint: None,
                notes: vec![],
            },
            CfdlDiagnostic {
                code: "E0003_B".to_string(),
                severity: "error".to_string(),
                message: "b".to_string(),
                file: Some("model.cfdl".to_string()),
                span: None,
                path: None,
                hint: None,
                notes: vec![],
            },
        ];

        let grouped = group_diagnostics_by_uri(&root, &source_uri, diagnostics);
        assert_eq!(grouped.len(), 2);
        let uris = grouped
            .values()
            .map(|(uri, _)| uri.as_str().to_string())
            .collect::<Vec<_>>();
        let mut sorted = uris.clone();
        sorted.sort();
        assert_eq!(uris, sorted);

        fs::remove_dir_all(root).expect("cleanup temp dir");
    }

    fn make_temp_dir(prefix: &str) -> PathBuf {
        let mut dir = std::env::temp_dir();
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock moved backwards")
            .as_nanos();
        dir.push(format!("cfdl-lsp-{prefix}-{}-{nanos}", std::process::id()));
        fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }
}
