use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use cfdl_compile::{CompileOptions, Diagnostic as CfdlDiagnostic, Span as CfdlSpan};
use cfdl_lexer::{Keyword, Token, TokenKind};
use cfdl_pack::{render_template, PackRegistry, PackTemplate};
use cfdl_parser::{ScheduleKind, Stmt};
use cfdl_resolver::{ResolveOutput, RootModule, SymbolTables};
use lsp_types::{
    CompletionItem, CompletionItemKind, CompletionOptions, CompletionParams, CompletionResponse,
    Diagnostic, DiagnosticSeverity, DidChangeConfigurationParams, DidChangeTextDocumentParams,
    DidCloseTextDocumentParams, DidOpenTextDocumentParams, ExecuteCommandOptions,
    ExecuteCommandParams, GotoDefinitionParams, GotoDefinitionResponse, Hover, HoverContents,
    HoverParams, HoverProviderCapability, InitializeParams, InitializeResult, Location,
    MarkupContent, MarkupKind, MessageType, NumberOrString, OneOf, Position, Range, SemanticToken,
    SemanticTokenModifier, SemanticTokenType, SemanticTokens, SemanticTokensDelta,
    SemanticTokensDeltaParams, SemanticTokensEdit, SemanticTokensFullDeltaResult,
    SemanticTokensFullOptions, SemanticTokensLegend, SemanticTokensOptions, SemanticTokensParams,
    SemanticTokensResult, ServerCapabilities, TextDocumentSyncCapability, TextDocumentSyncKind,
    Url,
};
use serde_json::Value;
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

    pub fn get(&self, uri: &Url) -> Option<&str> {
        self.docs.get(uri).map(String::as_str)
    }
}

#[derive(Debug, Clone)]
struct DefinitionBinding {
    source_range: Range,
    target: Location,
}

#[derive(Debug, Clone, Default)]
struct SymbolIndex {
    bindings_by_uri: HashMap<Url, Vec<DefinitionBinding>>,
}

impl SymbolIndex {
    fn add_binding(
        &mut self,
        source_uri: Url,
        source_range: Range,
        target_uri: Url,
        target_range: Range,
    ) {
        self.bindings_by_uri
            .entry(source_uri)
            .or_default()
            .push(DefinitionBinding {
                source_range,
                target: Location {
                    uri: target_uri,
                    range: target_range,
                },
            });
    }

    fn lookup(&self, uri: &Url, position: Position) -> Option<Location> {
        let bindings = self.bindings_by_uri.get(uri)?;
        bindings
            .iter()
            .find(|binding| range_contains_position(&binding.source_range, position))
            .map(|binding| binding.target.clone())
    }

    fn sort_bindings(&mut self) {
        for bindings in self.bindings_by_uri.values_mut() {
            bindings.sort_by(|a, b| {
                a.source_range
                    .start
                    .line
                    .cmp(&b.source_range.start.line)
                    .then(
                        a.source_range
                            .start
                            .character
                            .cmp(&b.source_range.start.character),
                    )
                    .then(a.target.uri.as_str().cmp(b.target.uri.as_str()))
                    .then(a.target.range.start.line.cmp(&b.target.range.start.line))
                    .then(
                        a.target
                            .range
                            .start
                            .character
                            .cmp(&b.target.range.start.character),
                    )
            });
        }
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct AnalysisContext {
    resolve_output: ResolveOutput,
    symbols: SymbolTables,
    file_tokens: HashMap<String, Vec<Token>>,
    symbol_index: SymbolIndex,
    pack_context: PackContext,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct SemanticTokenCacheEntry {
    result_id: String,
    data: Vec<SemanticToken>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum SemanticKind {
    Keyword = 0,
    String = 1,
    Number = 2,
    Type = 3,
    Property = 4,
    Variable = 5,
    Function = 6,
    EnumMember = 7,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum SemanticModifierKind {
    Declaration = 0,
    Readonly = 1,
}

#[derive(Debug, Clone)]
struct SemanticAtom {
    line: u32,
    start: u32,
    length: u32,
    kind: SemanticKind,
    modifiers: u32,
}

#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
struct PackContext {
    loaded_packs: Vec<PackSummary>,
    active_pack: Option<ActivePack>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct PackSummary {
    name: String,
    version: String,
}

#[derive(Debug, Clone)]
struct ActivePack {
    name: String,
    version: String,
    aliases: BTreeMap<String, String>,
    manifest_uri: Option<Url>,
    templates: Vec<TemplateInfo>,
}

#[derive(Debug, Clone)]
struct TemplateInfo {
    id: String,
    label: String,
    kind: String,
    body: String,
    defaults: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TraceServer {
    Off,
    Messages,
    Verbose,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct LspSettings {
    packs_path: Option<String>,
    entry_file: String,
    enable_lowering_validation: bool,
    trace_server: TraceServer,
}

impl Default for LspSettings {
    fn default() -> Self {
        Self {
            packs_path: None,
            entry_file: "model.cfdl".to_string(),
            enable_lowering_validation: true,
            trace_server: TraceServer::Off,
        }
    }
}

pub struct Backend {
    client: Client,
    docs: Arc<RwLock<DocumentStore>>,
    published_by_root: Arc<RwLock<HashMap<PathBuf, HashSet<Url>>>>,
    analysis_by_root: Arc<RwLock<HashMap<PathBuf, AnalysisContext>>>,
    semantic_tokens_by_uri: Arc<RwLock<HashMap<Url, SemanticTokenCacheEntry>>>,
    refresh_generation_by_root: Arc<RwLock<HashMap<PathBuf, u64>>>,
    settings: Arc<RwLock<LspSettings>>,
}

const CMD_LIST_TEMPLATES: &str = "cfdl.listTemplates";
const CMD_APPLY_TEMPLATE: &str = "cfdl.applyTemplate";

impl Backend {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            docs: Arc::new(RwLock::new(DocumentStore::default())),
            published_by_root: Arc::new(RwLock::new(HashMap::default())),
            analysis_by_root: Arc::new(RwLock::new(HashMap::default())),
            semantic_tokens_by_uri: Arc::new(RwLock::new(HashMap::default())),
            refresh_generation_by_root: Arc::new(RwLock::new(HashMap::default())),
            settings: Arc::new(RwLock::new(LspSettings::default())),
        }
    }

    async fn refresh_diagnostics_for_uri(&self, source_uri: &Url) {
        let settings = self.settings.read().await.clone();
        let Some(model_root) = detect_model_root_with_entry(source_uri, &settings.entry_file)
        else {
            self.client
                .publish_diagnostics(source_uri.clone(), vec![], None)
                .await;
            return;
        };

        let compile_root = model_root.clone();
        let compile_options = compile_options_for_root(&model_root, &settings);
        let compile_result = tokio::task::spawn_blocking(move || {
            cfdl_compile::compile_to_json_with_options(&compile_root, &compile_options)
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

    async fn refresh_analysis_for_uri(&self, source_uri: &Url) {
        let settings = self.settings.read().await.clone();
        let Some(model_root) = detect_model_root_with_entry(source_uri, &settings.entry_file)
        else {
            return;
        };

        let root_for_task = model_root.clone();
        let settings_for_task = settings.clone();
        let build_result = tokio::task::spawn_blocking(move || {
            build_analysis_context(&root_for_task, &settings_for_task)
        })
        .await;
        match build_result {
            Ok(Some(context)) => {
                let mut contexts = self.analysis_by_root.write().await;
                contexts.insert(model_root, context);
            }
            Ok(None) => {
                self.clear_analysis_for_root(&model_root).await;
            }
            Err(err) => {
                self.client
                    .log_message(
                        MessageType::ERROR,
                        format!("CFDL symbol index task failed: {err}"),
                    )
                    .await;
                self.clear_analysis_for_root(&model_root).await;
            }
        }
    }

    async fn definition_for_position(
        &self,
        uri: &Url,
        position: Position,
    ) -> Option<GotoDefinitionResponse> {
        let settings = self.settings.read().await.clone();
        let model_root = detect_model_root_with_entry(uri, &settings.entry_file)?;
        let contexts = self.analysis_by_root.read().await;
        let context = contexts.get(&model_root)?;
        context
            .symbol_index
            .lookup(uri, position)
            .map(GotoDefinitionResponse::Scalar)
    }

    async fn completion_for_uri(&self, uri: &Url) -> Option<CompletionResponse> {
        let settings = self.settings.read().await.clone();
        let model_root = detect_model_root_with_entry(uri, &settings.entry_file)?;
        let contexts = self.analysis_by_root.read().await;
        let context = contexts.get(&model_root)?;
        Some(CompletionResponse::Array(completion_items(context)))
    }

    async fn semantic_tokens_full_for_uri(&self, uri: &Url) -> Option<SemanticTokensResult> {
        let data = self.semantic_tokens_for_uri(uri).await?;
        let result_id = next_semantic_result_id(&data);
        let tokens = SemanticTokens {
            result_id: Some(result_id.clone()),
            data: data.clone(),
        };
        let mut cache = self.semantic_tokens_by_uri.write().await;
        cache.insert(uri.clone(), SemanticTokenCacheEntry { result_id, data });
        Some(SemanticTokensResult::Tokens(tokens))
    }

    async fn semantic_tokens_delta_for_uri(
        &self,
        uri: &Url,
        previous_result_id: &str,
    ) -> Option<SemanticTokensFullDeltaResult> {
        let next_data = self.semantic_tokens_for_uri(uri).await?;
        let next_result_id = next_semantic_result_id(&next_data);
        let mut cache = self.semantic_tokens_by_uri.write().await;
        let previous = cache.get(uri).cloned();
        let result = compute_semantic_delta_result(
            previous.as_ref(),
            previous_result_id,
            next_data.clone(),
            next_result_id.clone(),
        );
        cache.insert(
            uri.clone(),
            SemanticTokenCacheEntry {
                result_id: next_result_id,
                data: next_data,
            },
        );
        Some(result)
    }

    async fn semantic_tokens_for_uri(&self, uri: &Url) -> Option<Vec<SemanticToken>> {
        let docs = self.docs.read().await;
        if !docs.docs.contains_key(uri) {
            return Some(vec![]);
        }
        drop(docs);

        let settings = self.settings.read().await.clone();
        let model_root = detect_model_root_with_entry(uri, &settings.entry_file)?;
        let contexts = self.analysis_by_root.read().await;
        let context = contexts.get(&model_root)?;
        let relative_file = path_relative_to_root(uri, &model_root)?;
        let tokens = context.file_tokens.get(&relative_file)?;
        Some(encode_semantic_tokens(tokens))
    }

    async fn execute_command_for_params(&self, params: ExecuteCommandParams) -> Option<Value> {
        match params.command.as_str() {
            CMD_LIST_TEMPLATES => {
                let arg = params.arguments.first()?;
                let uri = command_uri(arg)?;
                self.list_templates_for_uri(&uri).await
            }
            CMD_APPLY_TEMPLATE => {
                let arg = params.arguments.first()?;
                let payload = parse_apply_template_request(arg)?;
                self.apply_template_for_request(payload).await
            }
            _ => None,
        }
    }

    async fn list_templates_for_uri(&self, uri: &Url) -> Option<Value> {
        let settings = self.settings.read().await.clone();
        let model_root = detect_model_root_with_entry(uri, &settings.entry_file)?;
        let contexts = self.analysis_by_root.read().await;
        let context = contexts.get(&model_root)?;
        let active = context.pack_context.active_pack.as_ref()?;
        let list = active
            .templates
            .iter()
            .map(|template| {
                serde_json::json!({
                    "id": template.id,
                    "label": template.label,
                    "kind": template.kind,
                    "pack": format!("{}@{}", active.name, active.version),
                })
            })
            .collect::<Vec<_>>();
        Some(Value::Array(list))
    }

    async fn apply_template_for_request(&self, request: ApplyTemplateRequest) -> Option<Value> {
        let settings = self.settings.read().await.clone();
        let model_root = detect_model_root_with_entry(&request.uri, &settings.entry_file)?;
        let contexts = self.analysis_by_root.read().await;
        let context = contexts.get(&model_root)?;
        let active = context.pack_context.active_pack.as_ref()?;
        let template = active
            .templates
            .iter()
            .find(|template| template.id == request.template_id)?;

        let expanded = render_template(
            &PackTemplate {
                id: template.id.clone(),
                label: Some(template.label.clone()),
                kind: Some(template.kind.clone()),
                body: template.body.clone(),
                defaults: template.defaults.clone(),
            },
            &request.params,
        );
        Some(serde_json::json!({ "text": expanded }))
    }

    async fn queue_refresh_for_uri(&self, source_uri: &Url) {
        let settings = self.settings.read().await.clone();
        let Some(model_root) = detect_model_root_with_entry(source_uri, &settings.entry_file)
        else {
            return;
        };
        let generation = {
            let mut generations = self.refresh_generation_by_root.write().await;
            let next = generations
                .get(&model_root)
                .copied()
                .unwrap_or(0)
                .saturating_add(1);
            generations.insert(model_root.clone(), next);
            next
        };

        tokio::time::sleep(Duration::from_millis(300)).await;

        let is_latest = {
            let generations = self.refresh_generation_by_root.read().await;
            generations
                .get(&model_root)
                .copied()
                .map(|value| value == generation)
                .unwrap_or(false)
        };
        if !is_latest {
            return;
        }

        if !self.uri_is_parseable(source_uri).await {
            self.clear_published_diagnostics_for_root(&model_root, source_uri)
                .await;
            self.clear_analysis_for_root(&model_root).await;
            return;
        }

        self.refresh_diagnostics_for_uri(source_uri).await;
        self.refresh_analysis_for_uri(source_uri).await;
    }

    async fn uri_is_parseable(&self, source_uri: &Url) -> bool {
        let docs = self.docs.read().await;
        let Some(text) = docs.docs.get(source_uri) else {
            return true;
        };
        source_parseable(source_uri.as_str(), text)
    }

    async fn clear_analysis_for_root(&self, model_root: &Path) {
        let mut contexts = self.analysis_by_root.write().await;
        contexts.remove(model_root);
        let mut semantic = self.semantic_tokens_by_uri.write().await;
        semantic.retain(|uri, _| {
            uri.to_file_path()
                .ok()
                .map(|path| !path.starts_with(model_root))
                .unwrap_or(true)
        });
    }

    async fn clear_published_diagnostics_for_root(&self, model_root: &Path, fallback_uri: &Url) {
        let stale_uris = {
            let mut tracked = self.published_by_root.write().await;
            tracked
                .remove(model_root)
                .unwrap_or_else(|| {
                    let mut uris = HashSet::new();
                    uris.insert(fallback_uri.clone());
                    uris
                })
                .into_iter()
                .collect::<Vec<_>>()
        };
        for uri in stale_uris {
            self.client.publish_diagnostics(uri, vec![], None).await;
        }
    }

    async fn update_settings(&self, incoming: &Value) {
        let mut settings = self.settings.write().await;
        apply_settings_value(&mut settings, incoming);
    }
}

#[derive(Debug, Clone)]
struct ApplyTemplateRequest {
    uri: Url,
    template_id: String,
    params: BTreeMap<String, String>,
}

pub fn server_capabilities() -> ServerCapabilities {
    ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
        definition_provider: Some(OneOf::Left(true)),
        hover_provider: Some(HoverProviderCapability::Simple(true)),
        completion_provider: Some(CompletionOptions::default()),
        execute_command_provider: Some(ExecuteCommandOptions {
            commands: vec![
                CMD_LIST_TEMPLATES.to_string(),
                CMD_APPLY_TEMPLATE.to_string(),
            ],
            work_done_progress_options: Default::default(),
        }),
        semantic_tokens_provider: Some(
            SemanticTokensOptions {
                work_done_progress_options: Default::default(),
                legend: semantic_tokens_legend(),
                range: None,
                full: Some(SemanticTokensFullOptions::Delta { delta: Some(true) }),
            }
            .into(),
        ),
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
        self.queue_refresh_for_uri(&uri).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let Some(change) = params.content_changes.into_iter().next() else {
            return;
        };
        let uri = params.text_document.uri.clone();
        let mut docs = self.docs.write().await;
        docs.change_full(&uri, change.text);
        drop(docs);
        self.queue_refresh_for_uri(&uri).await;
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

        let entry_file = { self.settings.read().await.entry_file.clone() };
        if let Some(model_root) = detect_model_root_with_entry(&uri, &entry_file) {
            let has_docs_in_root = {
                let docs = self.docs.read().await;
                docs.docs
                    .keys()
                    .any(|open_uri| root_matches(open_uri, &model_root, &entry_file))
            };
            if !has_docs_in_root {
                self.clear_analysis_for_root(&model_root).await;
                let mut generations = self.refresh_generation_by_root.write().await;
                generations.remove(&model_root);
            }
        }
    }

    async fn did_change_configuration(&self, params: DidChangeConfigurationParams) {
        self.update_settings(&params.settings).await;
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let text_document_position_params = params.text_document_position_params;
        Ok(self
            .definition_for_position(
                &text_document_position_params.text_document.uri,
                text_document_position_params.position,
            )
            .await)
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;
        let docs = self.docs.read().await;
        let Some(text) = docs.get(&uri) else {
            return Ok(None);
        };
        Ok(hover_at(text, position))
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        Ok(self
            .completion_for_uri(&params.text_document_position.text_document.uri)
            .await)
    }

    async fn execute_command(&self, params: ExecuteCommandParams) -> Result<Option<Value>> {
        Ok(self.execute_command_for_params(params).await)
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        Ok(self
            .semantic_tokens_full_for_uri(&params.text_document.uri)
            .await)
    }

    async fn semantic_tokens_full_delta(
        &self,
        params: SemanticTokensDeltaParams,
    ) -> Result<Option<SemanticTokensFullDeltaResult>> {
        Ok(self
            .semantic_tokens_delta_for_uri(&params.text_document.uri, &params.previous_result_id)
            .await)
    }
}

#[cfg(test)]
fn detect_model_root(uri: &Url) -> Option<PathBuf> {
    detect_model_root_with_entry(uri, "model.cfdl")
}

fn detect_model_root_with_entry(uri: &Url, entry_file: &str) -> Option<PathBuf> {
    let file_path = uri.to_file_path().ok()?;
    let mut current = if file_path.is_dir() {
        file_path
    } else {
        file_path.parent()?.to_path_buf()
    };

    loop {
        if current.join(entry_file).is_file() {
            return Some(current);
        }
        if !current.pop() {
            return None;
        }
    }
}

fn root_matches(uri: &Url, model_root: &Path, entry_file: &str) -> bool {
    detect_model_root_with_entry(uri, entry_file)
        .as_ref()
        .map(|root| root == model_root)
        .unwrap_or(false)
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

fn build_analysis_context(model_root: &Path, settings: &LspSettings) -> Option<AnalysisContext> {
    let (resolve_output, symbols) = analyze_model_root(model_root).ok()?;
    let file_tokens = load_tokens_by_file(model_root, &resolve_output)?;
    let pack_context = build_pack_context(model_root, settings, &resolve_output);
    let symbol_index = build_symbol_index(
        model_root,
        &resolve_output,
        &symbols,
        &file_tokens,
        &pack_context,
    )?;
    Some(AnalysisContext {
        resolve_output,
        symbols,
        file_tokens,
        symbol_index,
        pack_context,
    })
}

fn build_symbol_index(
    model_root: &Path,
    resolve_output: &ResolveOutput,
    symbols: &SymbolTables,
    file_tokens: &HashMap<String, Vec<Token>>,
    pack_context: &PackContext,
) -> Option<SymbolIndex> {
    let mut index = SymbolIndex::default();
    let mut entity_targets: HashMap<String, Location> = HashMap::new();
    let mut phase_targets: HashMap<String, Location> = HashMap::new();

    for entity in symbols.entities.values() {
        let uri = file_uri(model_root, &entity.file)?;
        let name_span = file_tokens
            .get(&entity.file)
            .and_then(|tokens| {
                let (namespace, name) = split_entity_symbol(&entity.name)?;
                find_entity_decl_name_span(tokens, &entity.span, namespace, name)
            })
            .unwrap_or(entity.span);
        let range = lex_span_to_range(&name_span);
        entity_targets.insert(
            entity.name.clone(),
            Location {
                uri: uri.clone(),
                range,
            },
        );
        index.add_binding(uri.clone(), range, uri, range);
    }

    for stream in symbols.streams.values() {
        let uri = file_uri(model_root, &stream.file)?;
        let name_span = file_tokens
            .get(&stream.file)
            .and_then(|tokens| find_stream_decl_name_span(tokens, &stream.span, &stream.name))
            .unwrap_or(stream.span);
        let range = lex_span_to_range(&name_span);
        index.add_binding(uri.clone(), range, uri, range);
    }

    let mut contract_decls = BTreeMap::new();
    for source_stmt in &resolve_output.source_statements {
        if let Stmt::Contract(contract) = &source_stmt.statement {
            contract_decls
                .entry(contract.name.clone())
                .or_insert((source_stmt.file.clone(), contract.span));
        }
        if let Stmt::Phase(phase) = &source_stmt.statement {
            let Some(uri) = file_uri(model_root, &source_stmt.file) else {
                continue;
            };
            let name_span = file_tokens
                .get(&source_stmt.file)
                .and_then(|tokens| find_phase_decl_name_span(tokens, &phase.span, &phase.name))
                .unwrap_or(phase.span);
            let range = lex_span_to_range(&name_span);
            phase_targets.insert(
                phase.name.clone(),
                Location {
                    uri: uri.clone(),
                    range,
                },
            );
            index.add_binding(uri.clone(), range, uri, range);
        }
    }

    for (contract_name, (contract_file, contract_span)) in contract_decls {
        let uri = file_uri(model_root, &contract_file)?;
        let name_span = file_tokens
            .get(&contract_file)
            .and_then(|tokens| find_contract_decl_name_span(tokens, &contract_span, &contract_name))
            .unwrap_or(contract_span);
        let range = lex_span_to_range(&name_span);
        index.add_binding(uri.clone(), range, uri, range);
    }

    for source_stmt in &resolve_output.source_statements {
        if let Stmt::UsePack(use_pack) = &source_stmt.statement {
            let Some(target_uri) = pack_context.active_pack.as_ref().and_then(|active| {
                if active.name == use_pack.name && active.version == use_pack.version {
                    active.manifest_uri.clone()
                } else {
                    None
                }
            }) else {
                continue;
            };
            let Some(source_uri) = file_uri(model_root, &source_stmt.file) else {
                continue;
            };
            let Some(tokens) = file_tokens.get(&source_stmt.file) else {
                continue;
            };
            let Some(pack_name_span) =
                find_use_pack_name_span(tokens, &use_pack.span, &use_pack.name, &use_pack.version)
            else {
                continue;
            };
            index.add_binding(
                source_uri,
                lex_span_to_range(&pack_name_span),
                target_uri,
                Range::new(Position::new(0, 0), Position::new(0, 0)),
            );
        }
    }

    for source_stmt in &resolve_output.source_statements {
        let Stmt::Stream(stream) = &source_stmt.statement else {
            continue;
        };
        let Some(tokens) = file_tokens.get(&source_stmt.file) else {
            continue;
        };
        let Some(source_uri) = file_uri(model_root, &source_stmt.file) else {
            continue;
        };

        if let Some(target) = entity_targets.get(&stream.attached_entity).cloned() {
            if let Some(ref_span) =
                find_stream_entity_ref_span(tokens, &stream.span, &stream.attached_entity)
            {
                index.add_binding(
                    source_uri.clone(),
                    lex_span_to_range(&ref_span),
                    target.uri,
                    target.range,
                );
            }
        }

        if let Some(schedule) = &stream.schedule {
            let phase_name = match &schedule.kind {
                ScheduleKind::PhaseEnter { phase } | ScheduleKind::EveryPhase { phase } => {
                    Some(phase.as_str())
                }
                _ => None,
            };
            if let Some(phase_name) = phase_name {
                if let Some(target) = phase_targets.get(phase_name).cloned() {
                    for phase_ref in find_schedule_phase_ref_spans(tokens, &stream.span, phase_name)
                    {
                        index.add_binding(
                            source_uri.clone(),
                            lex_span_to_range(&phase_ref),
                            target.uri.clone(),
                            target.range,
                        );
                    }
                }
            }
        }
    }

    index.sort_bindings();
    Some(index)
}

fn completion_items(context: &AnalysisContext) -> Vec<CompletionItem> {
    let mut items = vec![
        keyword_completion("version"),
        keyword_completion("model"),
        keyword_completion("use"),
        keyword_completion("pack"),
        keyword_completion("import"),
        keyword_completion("time"),
        keyword_completion("phase"),
        keyword_completion("entity"),
        keyword_completion("contract"),
        keyword_completion("stream"),
        keyword_completion("schedule"),
        keyword_completion("every"),
        keyword_completion("on"),
    ];

    if let Some(active) = &context.pack_context.active_pack {
        for (alias, canonical) in &active.aliases {
            items.push(CompletionItem {
                label: alias.clone(),
                kind: Some(CompletionItemKind::CLASS),
                detail: Some(format!(
                    "{} ({}@{})",
                    canonical, active.name, active.version
                )),
                sort_text: Some(format!("2-{alias}")),
                ..CompletionItem::default()
            });
        }
        for template in &active.templates {
            items.push(CompletionItem {
                label: template.label.clone(),
                kind: Some(CompletionItemKind::SNIPPET),
                detail: Some(format!(
                    "template {} ({}@{})",
                    template.kind, active.name, active.version
                )),
                insert_text: Some(template.body.clone()),
                sort_text: Some(format!("3-{}", template.id)),
                ..CompletionItem::default()
            });
        }
    }

    for (name, sig, doc) in EXPR_BUILTINS {
        items.push(CompletionItem {
            label: (*name).to_string(),
            kind: Some(CompletionItemKind::FUNCTION),
            detail: Some((*sig).to_string()),
            documentation: Some(lsp_types::Documentation::String((*doc).to_string())),
            sort_text: Some(format!("4-{name}")),
            ..CompletionItem::default()
        });
    }

    items.sort_by(|a, b| a.label.cmp(&b.label));
    items
}

/// Builtin expression functions (see docs/03_expression_environment.md §4).
/// (name, signature, documentation)
pub const EXPR_BUILTINS: &[(&str, &str, &str)] = &[
    ("if", "if(cond, a, b)", "Lazy conditional: only the taken branch is evaluated."),
    ("min", "min(a, b, ...)", "Smallest of the arguments."),
    ("max", "max(a, b, ...)", "Largest of the arguments."),
    ("sum", "sum(a, b, ...)", "Sum of the arguments."),
    ("avg", "avg(a, b, ...)", "Arithmetic mean of the arguments."),
    ("abs", "abs(x)", "Absolute value."),
    ("round", "round(x, [digits])", "Excel-style rounding: half away from zero."),
    ("round_down", "round_down(x, [digits])", "Round toward zero."),
    ("round_up", "round_up(x, [digits])", "Round away from zero."),
    ("pow", "pow(base, exp)", "Function form of `^`. Integer exponents are decimal-exact; fractional exponents use the float64 escape."),
    ("clamp", "clamp(x, lo, hi)", "Constrain `x` to the range [lo, hi]."),
    ("pmt", "pmt(rate, nper, pv, [fv], [due])", "Periodic payment of an annuity (Excel sign conventions)."),
    ("pv", "pv(rate, nper, pmt, [fv], [due])", "Present value of an annuity."),
    ("fv", "fv(rate, nper, pmt, [pv], [due])", "Future value of an annuity."),
    ("nper", "nper(rate, pmt, pv, [fv], [due])", "Number of periods for an annuity."),
    ("rate", "rate(nper, pmt, pv, [fv], [due], [guess])", "Periodic interest rate (Newton solver, tolerance 1e-12)."),
    ("ipmt", "ipmt(rate, per, nper, pv, [fv])", "Interest portion of payment `per` (1-based) on a level-pay annuity (Excel IPMT)."),
    ("ppmt", "ppmt(rate, per, nper, pv, [fv])", "Principal portion of payment `per` (1-based) on a level-pay annuity (Excel PPMT)."),
    ("macrs_rate", "macrs_rate(year, life)", "MACRS GDS half-year depreciation percentage (IRS Pub 946); 5/7/15/20-year property, 0-based year."),
    ("cpr_to_smm", "cpr_to_smm(cpr)", "Convert an annual prepayment rate (CPR) to the single-monthly mortality rate (SMM)."),
    ("curve_value", "curve_value(name, date)", "Look up a model-declared `curve` at a date (step = flat-forward, or linear interpolation per the curve's declaration)."),
    ("date", "date(y, m, d)", "Construct a calendar date."),
    ("edate", "edate(d, months)", "Shift a date by whole months, clamping to month end (Excel EDATE)."),
    ("eomonth", "eomonth(d, months)", "End of the month `months` away (Excel EOMONTH)."),
    ("year_frac", "year_frac(d1, d2, basis)", "Year fraction per ISDA/SIFMA day-count basis: \"30/360\", \"act/360\", \"act/365\"."),
];

/// Well-known namespace variables available in expressions.
pub const EXPR_NAMESPACE_DOCS: &[(&str, &str)] = &[
    ("time.t", "0-based period index of the evaluation step."),
    ("time.date", "Calendar date of the evaluation step."),
    ("time.phase", "Active phase name, if any."),
    ("model.id", "Model identifier."),
    ("model.base_currency", "Model base currency code."),
];

/// Extract the identifier (with dots) under `position` in `text`.
fn word_at(text: &str, position: Position) -> Option<String> {
    let line = text.lines().nth(position.line as usize)?;
    let chars: Vec<char> = line.chars().collect();
    let col = (position.character as usize).min(chars.len().saturating_sub(1));
    let is_word = |c: char| c.is_ascii_alphanumeric() || c == '_' || c == '.';
    if chars.is_empty() || !is_word(chars[col]) {
        return None;
    }
    let mut start = col;
    while start > 0 && is_word(chars[start - 1]) {
        start -= 1;
    }
    let mut end = col;
    while end + 1 < chars.len() && is_word(chars[end + 1]) {
        end += 1;
    }
    Some(chars[start..=end].iter().collect())
}

fn hover_at(text: &str, position: Position) -> Option<Hover> {
    let word = word_at(text, position)?;
    let markdown = if let Some((_, sig, doc)) =
        EXPR_BUILTINS.iter().find(|(name, _, _)| *name == word)
    {
        format!("```cfdl\n{sig}\n```\n\n{doc}")
    } else if let Some((name, doc)) = EXPR_NAMESPACE_DOCS.iter().find(|(name, _)| *name == word) {
        format!("`{name}` — {doc}")
    } else {
        return None;
    };
    Some(Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: markdown,
        }),
        range: None,
    })
}

fn keyword_completion(label: &str) -> CompletionItem {
    CompletionItem {
        label: label.to_string(),
        kind: Some(CompletionItemKind::KEYWORD),
        sort_text: Some(format!("1-{label}")),
        ..CompletionItem::default()
    }
}

fn analyze_model_root(model_root: &Path) -> std::result::Result<(ResolveOutput, SymbolTables), ()> {
    let model_file = model_root.join("model.cfdl");
    let source = std::fs::read_to_string(&model_file).map_err(|_| ())?;
    let (tokens, lex_diags) = cfdl_lexer::lex(&source);
    if !lex_diags.is_empty() {
        return Err(());
    }

    let parse_result = cfdl_parser::parse("model.cfdl", &source, &tokens);
    if !parse_result.diagnostics.is_empty() {
        return Err(());
    }

    let root_ast = parse_result.ast.ok_or(())?;
    let root_module = RootModule {
        relative_path: "model.cfdl".to_string(),
        full_path: std::fs::canonicalize(&model_file).unwrap_or(model_file),
        ast: root_ast,
    };
    let resolve_output = cfdl_resolver::resolve_imports(model_root, root_module).map_err(|_| ())?;
    let symbols = cfdl_resolver::resolve_symbols(&resolve_output).map_err(|_| ())?;
    Ok((resolve_output, symbols))
}

fn source_parseable(file: &str, source: &str) -> bool {
    let (tokens, lex_diags) = cfdl_lexer::lex(source);
    if !lex_diags.is_empty() {
        return false;
    }
    let parse_result = cfdl_parser::parse(file, source, &tokens);
    parse_result.diagnostics.is_empty()
}

fn compile_options_for_root(model_root: &Path, settings: &LspSettings) -> CompileOptions {
    CompileOptions {
        packs_dir: Some(resolve_packs_root(model_root, settings)),
    }
}

fn resolve_packs_root(model_root: &Path, settings: &LspSettings) -> PathBuf {
    let configured = settings
        .packs_path
        .as_ref()
        .map(|raw| raw.trim())
        .filter(|raw| !raw.is_empty())
        .map(PathBuf::from);
    match configured {
        Some(path) if path.is_absolute() => path,
        Some(path) => model_root.join(path),
        None => model_root.join("packs"),
    }
}

fn build_pack_context(
    model_root: &Path,
    settings: &LspSettings,
    resolve_output: &ResolveOutput,
) -> PackContext {
    let pack_root = resolve_packs_root(model_root, settings);
    let registry = match PackRegistry::load_from_dir(&pack_root) {
        Ok(value) => value,
        Err(_) => {
            return PackContext::default();
        }
    };

    let mut loaded_packs = registry
        .list()
        .into_iter()
        .map(|pack| PackSummary {
            name: pack.manifest.name.clone(),
            version: pack.manifest.version.clone(),
        })
        .collect::<Vec<_>>();
    loaded_packs.sort_by(|a, b| a.name.cmp(&b.name).then(a.version.cmp(&b.version)));

    let active_use_pack = resolve_output
        .source_statements
        .iter()
        .find_map(|source_stmt| {
            let Stmt::UsePack(use_pack) = &source_stmt.statement else {
                return None;
            };
            Some((use_pack.name.clone(), use_pack.version.clone()))
        });
    let active_pack = active_use_pack.and_then(|(name, version)| {
        let loaded = registry.active_pack(&name, &version)?;
        let aliases = collect_aliases_for_pack(&registry, &loaded.name);
        let manifest_uri = find_pack_manifest_uri(&pack_root, &loaded.name, &loaded.version);
        let templates = collect_templates_for_pack(&registry, &loaded.name);
        Some(ActivePack {
            name: loaded.name,
            version: loaded.version,
            aliases,
            manifest_uri,
            templates,
        })
    });

    PackContext {
        loaded_packs,
        active_pack,
    }
}

fn collect_aliases_for_pack(registry: &PackRegistry, pack_name: &str) -> BTreeMap<String, String> {
    let mut aliases = BTreeMap::new();
    let Some(pack) = registry
        .list()
        .into_iter()
        .find(|loaded| loaded.manifest.name == pack_name)
    else {
        return aliases;
    };
    let keys = pack.aliases.keys().cloned().collect::<BTreeSet<_>>();
    for alias in keys {
        if let Some(canonical) = registry.lookup_alias(pack_name, &alias) {
            aliases.insert(alias, canonical.to_string());
        }
    }
    aliases
}

fn collect_templates_for_pack(registry: &PackRegistry, pack_name: &str) -> Vec<TemplateInfo> {
    let mut templates = registry
        .templates(pack_name)
        .into_iter()
        .map(|template| TemplateInfo {
            id: template.id.clone(),
            label: template
                .label
                .clone()
                .unwrap_or_else(|| template.id.clone()),
            kind: template
                .kind
                .clone()
                .unwrap_or_else(|| "template".to_string()),
            body: template.body.clone(),
            defaults: template.defaults.clone(),
        })
        .collect::<Vec<_>>();
    templates.sort_by(|a, b| a.id.cmp(&b.id).then(a.label.cmp(&b.label)));
    templates
}

fn find_pack_manifest_uri(pack_root: &Path, pack_name: &str, version: &str) -> Option<Url> {
    let mut dirs = std::fs::read_dir(pack_root)
        .ok()?
        .filter_map(|entry| entry.ok().map(|value| value.path()))
        .filter(|path| path.is_dir())
        .collect::<Vec<_>>();
    dirs.sort();
    for dir in dirs {
        let manifest = dir.join("pack.toml");
        if !manifest.is_file() {
            continue;
        }
        let Ok(raw) = std::fs::read_to_string(&manifest) else {
            continue;
        };
        let Ok(parsed) = toml::from_str::<cfdl_pack::PackManifest>(&raw) else {
            continue;
        };
        if parsed.name == pack_name && parsed.version == version {
            return Url::from_file_path(manifest).ok();
        }
    }
    None
}

fn load_tokens_by_file(
    model_root: &Path,
    output: &ResolveOutput,
) -> Option<HashMap<String, Vec<Token>>> {
    let mut files = output
        .source_statements
        .iter()
        .map(|stmt| stmt.file.clone())
        .collect::<Vec<_>>();
    files.sort();
    files.dedup();

    let mut map = HashMap::new();
    for file in files {
        let source = std::fs::read_to_string(model_root.join(&file)).ok()?;
        let (tokens, lex_diags) = cfdl_lexer::lex(&source);
        if !lex_diags.is_empty() {
            return None;
        }
        map.insert(file, tokens);
    }
    Some(map)
}

fn file_uri(model_root: &Path, relative_file: &str) -> Option<Url> {
    Url::from_file_path(model_root.join(relative_file)).ok()
}

fn split_entity_symbol(symbol: &str) -> Option<(&str, &str)> {
    let (namespace, name) = symbol.split_once('.')?;
    Some((namespace, name))
}

fn find_entity_decl_name_span(
    tokens: &[Token],
    stmt_span: &cfdl_lexer::Span,
    namespace: &str,
    name: &str,
) -> Option<cfdl_lexer::Span> {
    for window in tokens.windows(3) {
        if !window
            .iter()
            .all(|token| token_within_span(token, stmt_span))
        {
            continue;
        }
        if window[0].kind == TokenKind::Keyword(Keyword::Entity)
            && token_text(&window[1]) == Some(namespace)
            && token_text(&window[2]) == Some(name)
        {
            return Some(window[2].span);
        }
    }
    None
}

fn find_stream_decl_name_span(
    tokens: &[Token],
    stmt_span: &cfdl_lexer::Span,
    name: &str,
) -> Option<cfdl_lexer::Span> {
    for window in tokens.windows(2) {
        if !window
            .iter()
            .all(|token| token_within_span(token, stmt_span))
        {
            continue;
        }
        if window[0].kind == TokenKind::Keyword(Keyword::Stream)
            && token_text(&window[1]) == Some(name)
        {
            return Some(window[1].span);
        }
    }
    None
}

fn find_contract_decl_name_span(
    tokens: &[Token],
    stmt_span: &cfdl_lexer::Span,
    name: &str,
) -> Option<cfdl_lexer::Span> {
    for window in tokens.windows(3) {
        if !window
            .iter()
            .all(|token| token_within_span(token, stmt_span))
        {
            continue;
        }
        if window[0].kind == TokenKind::Keyword(Keyword::Contract)
            && token_text(&window[2]) == Some(name)
        {
            return Some(window[2].span);
        }
    }
    None
}

fn find_use_pack_name_span(
    tokens: &[Token],
    stmt_span: &cfdl_lexer::Span,
    name: &str,
    version: &str,
) -> Option<cfdl_lexer::Span> {
    for window in tokens.windows(5) {
        if !window
            .iter()
            .all(|token| token_within_span(token, stmt_span))
        {
            continue;
        }
        let matches_shape = window[0].kind == TokenKind::Keyword(Keyword::Use)
            && window[1].kind == TokenKind::Keyword(Keyword::Pack)
            && window[3].kind == TokenKind::Keyword(Keyword::Version);
        if matches_shape
            && token_string_value(&window[2]) == Some(name)
            && token_string_value(&window[4]) == Some(version)
        {
            return Some(window[2].span);
        }
    }
    None
}

fn find_phase_decl_name_span(
    tokens: &[Token],
    stmt_span: &cfdl_lexer::Span,
    name: &str,
) -> Option<cfdl_lexer::Span> {
    for window in tokens.windows(2) {
        if !window
            .iter()
            .all(|token| token_within_span(token, stmt_span))
        {
            continue;
        }
        if window[0].kind == TokenKind::Keyword(Keyword::Phase)
            && token_text(&window[1]) == Some(name)
        {
            return Some(window[1].span);
        }
    }
    None
}

fn find_stream_entity_ref_span(
    tokens: &[Token],
    stmt_span: &cfdl_lexer::Span,
    entity_ref: &str,
) -> Option<cfdl_lexer::Span> {
    for window in tokens.windows(3) {
        if !window
            .iter()
            .all(|token| token_within_span(token, stmt_span))
        {
            continue;
        }
        if window[0].kind == TokenKind::Keyword(Keyword::On)
            && window[1].kind == TokenKind::Keyword(Keyword::Entity)
            && token_text(&window[2]) == Some(entity_ref)
        {
            return Some(window[2].span);
        }
    }
    None
}

fn find_schedule_phase_ref_spans(
    tokens: &[Token],
    stmt_span: &cfdl_lexer::Span,
    phase: &str,
) -> Vec<cfdl_lexer::Span> {
    let mut spans = Vec::new();
    for window in tokens.windows(4) {
        if !window
            .iter()
            .all(|token| token_within_span(token, stmt_span))
        {
            continue;
        }
        let is_phase_ref = (window[0].kind == TokenKind::Keyword(Keyword::PhaseEnter)
            || window[0].kind == TokenKind::Keyword(Keyword::PhaseStart)
            || window[0].kind == TokenKind::Keyword(Keyword::PhaseEnd))
            && window[1].kind == TokenKind::Punct(cfdl_lexer::Punct::LParen)
            && window[3].kind == TokenKind::Punct(cfdl_lexer::Punct::RParen);
        if is_phase_ref && token_string_value(&window[2]) == Some(phase) {
            spans.push(window[2].span);
        }
    }
    spans
}

fn token_text(token: &Token) -> Option<&str> {
    match &token.kind {
        TokenKind::Ident(value) | TokenKind::Qname(value) => Some(value.as_str()),
        _ => None,
    }
}

fn token_string_value(token: &Token) -> Option<&str> {
    match &token.kind {
        TokenKind::String(value) => Some(value.as_str()),
        _ => None,
    }
}

fn token_within_span(token: &Token, outer: &cfdl_lexer::Span) -> bool {
    span_contains(outer, &token.span)
}

fn span_contains(outer: &cfdl_lexer::Span, inner: &cfdl_lexer::Span) -> bool {
    compare_line_col(
        inner.start_line,
        inner.start_col,
        outer.start_line,
        outer.start_col,
    ) != std::cmp::Ordering::Less
        && compare_line_col(inner.end_line, inner.end_col, outer.end_line, outer.end_col)
            != std::cmp::Ordering::Greater
}

fn lex_span_to_range(span: &cfdl_lexer::Span) -> Range {
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

fn range_contains_position(range: &Range, position: Position) -> bool {
    compare_positions(position, range.start) != std::cmp::Ordering::Less
        && compare_positions(position, range.end) != std::cmp::Ordering::Greater
}

fn compare_positions(a: Position, b: Position) -> std::cmp::Ordering {
    a.line.cmp(&b.line).then(a.character.cmp(&b.character))
}

fn compare_line_col(a_line: u32, a_col: u32, b_line: u32, b_col: u32) -> std::cmp::Ordering {
    a_line.cmp(&b_line).then(a_col.cmp(&b_col))
}

fn apply_settings_value(settings: &mut LspSettings, value: &Value) {
    let scoped = value.get("cfdl").unwrap_or(value);
    if let Some(entry_file) = scoped.get("entryFile").and_then(Value::as_str) {
        let trimmed = entry_file.trim();
        if !trimmed.is_empty() {
            settings.entry_file = trimmed.to_string();
        }
    }
    settings.packs_path = scoped
        .get("packsPath")
        .and_then(Value::as_str)
        .map(|value| value.to_string());
    if let Some(enable) = scoped
        .get("enableLoweringValidation")
        .and_then(Value::as_bool)
    {
        settings.enable_lowering_validation = enable;
    }
    let trace_value = scoped
        .get("trace")
        .and_then(|value| value.get("server"))
        .and_then(Value::as_str)
        .or_else(|| scoped.get("trace.server").and_then(Value::as_str));
    if let Some(trace) = trace_value {
        settings.trace_server = match trace {
            "messages" => TraceServer::Messages,
            "verbose" => TraceServer::Verbose,
            _ => TraceServer::Off,
        };
    }
}

fn command_uri(value: &Value) -> Option<Url> {
    if let Some(uri) = value.as_str() {
        return Url::parse(uri).ok();
    }
    value
        .get("uri")
        .and_then(Value::as_str)
        .and_then(|uri| Url::parse(uri).ok())
}

fn parse_apply_template_request(value: &Value) -> Option<ApplyTemplateRequest> {
    let uri = command_uri(value)?;
    let template_id = value.get("templateId")?.as_str()?.to_string();
    let mut params = BTreeMap::new();
    if let Some(object) = value.get("params").and_then(Value::as_object) {
        for (key, raw) in object {
            if let Some(as_str) = raw.as_str() {
                params.insert(key.clone(), as_str.to_string());
            } else if raw.is_number() || raw.is_boolean() {
                params.insert(key.clone(), raw.to_string());
            }
        }
    }
    Some(ApplyTemplateRequest {
        uri,
        template_id,
        params,
    })
}

fn semantic_tokens_legend() -> SemanticTokensLegend {
    SemanticTokensLegend {
        token_types: vec![
            SemanticTokenType::KEYWORD,
            SemanticTokenType::STRING,
            SemanticTokenType::NUMBER,
            SemanticTokenType::TYPE,
            SemanticTokenType::PROPERTY,
            SemanticTokenType::VARIABLE,
            SemanticTokenType::FUNCTION,
            SemanticTokenType::ENUM_MEMBER,
        ],
        token_modifiers: vec![
            SemanticTokenModifier::DECLARATION,
            SemanticTokenModifier::READONLY,
        ],
    }
}

fn encode_semantic_tokens(tokens: &[Token]) -> Vec<SemanticToken> {
    let mut atoms = tokens
        .iter()
        .filter_map(classify_semantic_atom)
        .collect::<Vec<_>>();
    atoms.sort_by(|a, b| {
        a.line
            .cmp(&b.line)
            .then(a.start.cmp(&b.start))
            .then(a.length.cmp(&b.length))
            .then(a.kind.cmp(&b.kind))
            .then(a.modifiers.cmp(&b.modifiers))
    });

    let mut encoded = Vec::with_capacity(atoms.len());
    let mut prev_line = 0u32;
    let mut prev_start = 0u32;
    for atom in atoms {
        let delta_line = atom.line.saturating_sub(prev_line);
        let delta_start = if delta_line == 0 {
            atom.start.saturating_sub(prev_start)
        } else {
            atom.start
        };
        encoded.push(SemanticToken {
            delta_line,
            delta_start,
            length: atom.length,
            token_type: atom.kind as u32,
            token_modifiers_bitset: atom.modifiers,
        });
        prev_line = atom.line;
        prev_start = atom.start;
    }
    encoded
}

fn classify_semantic_atom(token: &Token) -> Option<SemanticAtom> {
    let line = token.span.start_line.saturating_sub(1);
    let start = token.span.start_col.saturating_sub(1);
    let length = token
        .span
        .end_col
        .saturating_sub(token.span.start_col)
        .saturating_add(1);
    if length == 0 {
        return None;
    }

    let (kind, modifiers) = match &token.kind {
        TokenKind::Keyword(keyword) => {
            let semantic_kind = match keyword {
                Keyword::Model | Keyword::Entity | Keyword::Contract | Keyword::Type => {
                    SemanticKind::Type
                }
                Keyword::Phase | Keyword::Pack => SemanticKind::EnumMember,
                _ => SemanticKind::Keyword,
            };
            let mut modifier_kinds = Vec::new();
            if matches!(
                keyword,
                Keyword::Entity | Keyword::Stream | Keyword::Contract | Keyword::Phase
            ) {
                modifier_kinds.push(SemanticModifierKind::Declaration);
            }
            if matches!(keyword, Keyword::True | Keyword::False | Keyword::None) {
                modifier_kinds.push(SemanticModifierKind::Readonly);
            }
            let modifiers = modifier_bitset(&modifier_kinds);
            (semantic_kind, modifiers)
        }
        TokenKind::String(_) => (SemanticKind::String, 0),
        TokenKind::Number(_) | TokenKind::Date(_) => (SemanticKind::Number, 0),
        TokenKind::Ident(value) => {
            let semantic_kind = if value.starts_with("is_") || value.starts_with("has_") {
                SemanticKind::Function
            } else {
                SemanticKind::Variable
            };
            (semantic_kind, 0)
        }
        TokenKind::Qname(value) => {
            let semantic_kind = if value.contains('.') {
                SemanticKind::Property
            } else {
                SemanticKind::Type
            };
            (semantic_kind, 0)
        }
        TokenKind::Punct(_) | TokenKind::Eof => return None,
    };

    Some(SemanticAtom {
        line,
        start,
        length,
        kind,
        modifiers,
    })
}

fn modifier_bitset(modifiers: &[SemanticModifierKind]) -> u32 {
    modifiers
        .iter()
        .fold(0u32, |acc, modifier| acc | (1u32 << (*modifier as u32)))
}

fn next_semantic_result_id(tokens: &[SemanticToken]) -> String {
    format!("v{}-{}", tokens.len(), semantic_checksum(tokens))
}

fn compute_semantic_delta_result(
    previous: Option<&SemanticTokenCacheEntry>,
    previous_result_id: &str,
    next_data: Vec<SemanticToken>,
    next_result_id: String,
) -> SemanticTokensFullDeltaResult {
    match previous {
        Some(prev) if prev.result_id == previous_result_id => {
            if prev.data == next_data {
                SemanticTokensFullDeltaResult::Tokens(SemanticTokens {
                    result_id: Some(next_result_id),
                    data: next_data,
                })
            } else {
                let edit = SemanticTokensEdit {
                    start: 0,
                    delete_count: prev.data.len() as u32 * 5,
                    data: Some(next_data),
                };
                SemanticTokensFullDeltaResult::TokensDelta(SemanticTokensDelta {
                    result_id: Some(next_result_id),
                    edits: vec![edit],
                })
            }
        }
        _ => SemanticTokensFullDeltaResult::Tokens(SemanticTokens {
            result_id: Some(next_result_id),
            data: next_data,
        }),
    }
}

fn semantic_checksum(tokens: &[SemanticToken]) -> u64 {
    tokens.iter().fold(1469598103934665603u64, |acc, token| {
        let mut hash = acc;
        for value in [
            token.delta_line as u64,
            token.delta_start as u64,
            token.length as u64,
            token.token_type as u64,
            token.token_modifiers_bitset as u64,
        ] {
            hash ^= value.wrapping_add(0x9e3779b97f4a7c15);
            hash = hash.wrapping_mul(1099511628211);
        }
        hash
    })
}

fn path_relative_to_root(uri: &Url, model_root: &Path) -> Option<String> {
    let path = uri.to_file_path().ok()?;
    let relative = path.strip_prefix(model_root).ok()?;
    Some(relative.to_string_lossy().replace('\\', "/"))
}

#[cfg(test)]
mod tests {
    use super::{
        apply_settings_value, build_analysis_context, cfdl_diagnostic_to_lsp, command_uri,
        completion_items, compute_semantic_delta_result, detect_model_root,
        detect_model_root_with_entry, encode_semantic_tokens, group_diagnostics_by_uri, hover_at,
        modifier_bitset, parse_apply_template_request, resolve_packs_root, semantic_tokens_legend,
        server_capabilities, source_parseable, ApplyTemplateRequest, DocumentStore, LspSettings,
        SemanticModifierKind, TraceServer, CMD_APPLY_TEMPLATE, CMD_LIST_TEMPLATES,
    };
    use cfdl_compile::{Diagnostic as CfdlDiagnostic, Span as CfdlSpan};
    use cfdl_lexer::{Keyword, Token, TokenKind};
    use lsp_types::{CompletionItemKind, HoverContents};
    use lsp_types::{
        Position, SemanticToken, SemanticTokensFullDeltaResult, TextDocumentSyncCapability,
        TextDocumentSyncKind, Url,
    };
    use serde_json::json;
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
        assert!(capabilities.completion_provider.is_some());
        let command_provider = capabilities
            .execute_command_provider
            .expect("execute command provider");
        assert!(command_provider
            .commands
            .contains(&CMD_LIST_TEMPLATES.to_string()));
        assert!(command_provider
            .commands
            .contains(&CMD_APPLY_TEMPLATE.to_string()));
        assert!(capabilities.semantic_tokens_provider.is_some());
    }

    #[test]
    fn semantic_tokens_legend_is_stable() {
        let legend = semantic_tokens_legend();
        assert_eq!(legend.token_types.len(), 8);
        assert_eq!(legend.token_modifiers.len(), 2);
    }

    #[test]
    fn semantic_token_encoding_is_deterministic() {
        let tokens = vec![
            Token {
                kind: TokenKind::Keyword(Keyword::Model),
                span: cfdl_lexer::Span {
                    start_line: 1,
                    start_col: 1,
                    end_line: 1,
                    end_col: 5,
                },
            },
            Token {
                kind: TokenKind::String("demo".to_string()),
                span: cfdl_lexer::Span {
                    start_line: 1,
                    start_col: 7,
                    end_line: 1,
                    end_col: 12,
                },
            },
        ];
        let first = encode_semantic_tokens(&tokens);
        let second = encode_semantic_tokens(&tokens);
        assert_eq!(first, second);
    }

    #[test]
    fn semantic_token_modifier_bitset_is_consistent() {
        let declaration = modifier_bitset(&[SemanticModifierKind::Declaration]);
        let readonly = modifier_bitset(&[SemanticModifierKind::Readonly]);
        assert_eq!(declaration, 1);
        assert_eq!(readonly, 2);
    }

    #[test]
    fn semantic_delta_returns_edit_on_cache_hit_and_changed_tokens() {
        let previous = super::SemanticTokenCacheEntry {
            result_id: "old".to_string(),
            data: vec![SemanticToken {
                delta_line: 0,
                delta_start: 0,
                length: 5,
                token_type: 0,
                token_modifiers_bitset: 0,
            }],
        };
        let next = vec![SemanticToken {
            delta_line: 0,
            delta_start: 0,
            length: 6,
            token_type: 1,
            token_modifiers_bitset: 0,
        }];
        let result =
            compute_semantic_delta_result(Some(&previous), "old", next.clone(), "next".to_string());
        match result {
            SemanticTokensFullDeltaResult::TokensDelta(delta) => {
                assert_eq!(delta.edits.len(), 1);
                assert_eq!(delta.edits[0].start, 0);
                assert_eq!(delta.edits[0].delete_count, 5);
                assert_eq!(delta.edits[0].data, Some(next));
            }
            SemanticTokensFullDeltaResult::Tokens(_) => panic!("expected delta edits"),
            SemanticTokensFullDeltaResult::PartialTokensDelta { .. } => {
                panic!("expected delta edits with result id")
            }
        }
    }

    #[test]
    fn semantic_delta_falls_back_to_full_for_unknown_previous_id() {
        let previous = super::SemanticTokenCacheEntry {
            result_id: "known".to_string(),
            data: vec![],
        };
        let next = vec![SemanticToken {
            delta_line: 0,
            delta_start: 1,
            length: 3,
            token_type: 0,
            token_modifiers_bitset: 0,
        }];
        let result = compute_semantic_delta_result(
            Some(&previous),
            "unknown",
            next.clone(),
            "next".to_string(),
        );
        match result {
            SemanticTokensFullDeltaResult::Tokens(tokens) => {
                assert_eq!(tokens.data, next);
            }
            SemanticTokensFullDeltaResult::TokensDelta(_) => {
                panic!("expected full fallback tokens")
            }
            SemanticTokensFullDeltaResult::PartialTokensDelta { .. } => {
                panic!("expected full fallback tokens")
            }
        }
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

    #[test]
    fn symbol_index_is_deterministic_for_same_model() {
        let root = make_temp_dir("symbol-index-deterministic");
        fs::write(root.join("model.cfdl"), sample_model_source().as_bytes()).expect("write model");

        let settings = LspSettings::default();
        let first = build_analysis_context(&root, &settings).expect("first context");
        let second = build_analysis_context(&root, &settings).expect("second context");
        assert_eq!(
            format!("{:?}", first.symbol_index),
            format!("{:?}", second.symbol_index),
            "symbol index must be deterministic"
        );

        fs::remove_dir_all(root).expect("cleanup temp dir");
    }

    #[test]
    fn symbol_declaration_lookup_returns_own_location() {
        let root = make_temp_dir("symbol-decl-lookup");
        let source = sample_model_source();
        let model_file = root.join("model.cfdl");
        fs::write(&model_file, source.as_bytes()).expect("write model");
        let model_uri = Url::from_file_path(&model_file).expect("valid model uri");
        let settings = LspSettings::default();
        let context = build_analysis_context(&root, &settings).expect("analysis");
        let index = context.symbol_index;

        let entity_pos = position_of_first(&source, "borrower");
        let entity_def = index
            .lookup(&model_uri, entity_pos)
            .expect("entity definition");
        assert_eq!(entity_def.uri, model_uri);
        assert_eq!(entity_def.range.start, entity_pos);

        let stream_pos = position_of_first(&source, "cre.rent");
        let stream_def = index
            .lookup(&model_uri, stream_pos)
            .expect("stream definition");
        assert_eq!(stream_def.uri, model_uri);
        assert_eq!(stream_def.range.start, stream_pos);

        let contract_pos = position_of_first(&source, "cre.lease_main");
        let contract_def = index
            .lookup(&model_uri, contract_pos)
            .expect("contract definition");
        assert_eq!(contract_def.uri, model_uri);
        assert_eq!(contract_def.range.start, contract_pos);

        fs::remove_dir_all(root).expect("cleanup temp dir");
    }

    #[test]
    fn stream_entity_reference_resolves_to_entity_declaration() {
        let root = make_temp_dir("entity-ref-lookup");
        let source = sample_model_source();
        let model_file = root.join("model.cfdl");
        fs::write(&model_file, source.as_bytes()).expect("write model");
        let model_uri = Url::from_file_path(&model_file).expect("valid model uri");
        let settings = LspSettings::default();
        let context = build_analysis_context(&root, &settings).expect("analysis");
        let index = context.symbol_index;

        let reference_pos = position_of_last(&source, "legal.borrower");
        let definition = index
            .lookup(&model_uri, reference_pos)
            .expect("entity definition");
        let expected_entity_pos = position_of_first(&source, "borrower");

        assert_eq!(definition.uri, model_uri);
        assert_eq!(definition.range.start, expected_entity_pos);

        fs::remove_dir_all(root).expect("cleanup temp dir");
    }

    #[test]
    fn lookup_returns_none_for_non_symbol_position() {
        let root = make_temp_dir("symbol-none");
        let source = sample_model_source();
        let model_file = root.join("model.cfdl");
        fs::write(&model_file, source.as_bytes()).expect("write model");
        let model_uri = Url::from_file_path(&model_file).expect("valid model uri");
        let settings = LspSettings::default();
        let context = build_analysis_context(&root, &settings).expect("analysis");
        let index = context.symbol_index;

        let pos = position_of_first(&source, "version");
        assert!(index.lookup(&model_uri, pos).is_none());

        fs::remove_dir_all(root).expect("cleanup temp dir");
    }

    fn sample_model_source() -> String {
        r#"version 0.1
model "demo"
time calendar monthly from 2026-01 for 12
phase base from 2026-01 to 2026-12
entity legal borrower
stream cre.rent on entity legal.borrower {
  schedule on phase_enter("base")
}
contract core.lease cre.lease_main term 2026-01..2026-12
"#
        .to_string()
    }

    #[test]
    fn schedule_phase_reference_resolves_to_phase_declaration() {
        let root = make_temp_dir("phase-ref-lookup");
        let source = sample_model_source();
        let model_file = root.join("model.cfdl");
        fs::write(&model_file, source.as_bytes()).expect("write model");
        let model_uri = Url::from_file_path(&model_file).expect("valid model uri");
        let settings = LspSettings::default();
        let context = build_analysis_context(&root, &settings).expect("analysis");

        let reference_pos = position_of_last(&source, "\"base\"");
        let definition = context
            .symbol_index
            .lookup(&model_uri, reference_pos)
            .expect("phase definition");
        let expected_phase_pos = position_of_first(&source, "base");
        assert_eq!(definition.range.start, expected_phase_pos);

        fs::remove_dir_all(root).expect("cleanup temp dir");
    }

    #[test]
    fn parses_settings_defaults_and_overrides() {
        let mut settings = LspSettings::default();
        apply_settings_value(
            &mut settings,
            &json!({
                "cfdl": {
                    "packsPath": "/tmp/packs",
                    "entryFile": "entry.cfdl",
                    "enableLoweringValidation": false,
                    "trace": { "server": "verbose" }
                }
            }),
        );
        assert_eq!(settings.packs_path.as_deref(), Some("/tmp/packs"));
        assert_eq!(settings.entry_file, "entry.cfdl");
        assert!(!settings.enable_lowering_validation);
        assert_eq!(settings.trace_server, TraceServer::Verbose);
    }

    #[test]
    fn parseable_guard_recognizes_invalid_source() {
        assert!(source_parseable(
            "model.cfdl",
            "version 0.1\nmodel \"ok\"\n"
        ));
        assert!(!source_parseable(
            "model.cfdl",
            "version 0.1\nmodel \"unterminated\n"
        ));
    }

    #[test]
    fn analysis_context_rebuilds_consistently_across_source_changes() {
        let root = make_temp_dir("analysis-rebuild");
        let model_file = root.join("model.cfdl");
        let valid = sample_model_source();
        fs::write(&model_file, valid.as_bytes()).expect("write valid");
        let settings = LspSettings::default();
        assert!(build_analysis_context(&root, &settings).is_some());

        fs::write(&model_file, "version 0.1\nmodel \"broken\n".as_bytes()).expect("write broken");
        assert!(build_analysis_context(&root, &settings).is_none());

        fs::write(&model_file, valid.as_bytes()).expect("write valid again");
        assert!(build_analysis_context(&root, &settings).is_some());

        fs::remove_dir_all(root).expect("cleanup temp dir");
    }

    #[test]
    fn model_root_detection_uses_configured_entry_file() {
        let root = make_temp_dir("entry-detect");
        fs::write(root.join("entry.cfdl"), "version 0.1".as_bytes()).expect("write entry");
        let nested = root.join("a").join("b");
        fs::create_dir_all(&nested).expect("create nested");
        let file_path = nested.join("module.cfdl");
        fs::write(&file_path, "".as_bytes()).expect("write module");
        let uri = Url::from_file_path(file_path).expect("uri");
        assert!(detect_model_root(&uri).is_none());
        assert_eq!(
            detect_model_root_with_entry(&uri, "entry.cfdl").expect("root"),
            root
        );
        fs::remove_dir_all(root).expect("cleanup temp dir");
    }

    #[test]
    fn resolves_packs_root_with_configured_and_default_paths() {
        let root = make_temp_dir("packs-root");
        let mut settings = LspSettings::default();
        assert_eq!(resolve_packs_root(&root, &settings), root.join("packs"));

        settings.packs_path = Some("custom-packs".to_string());
        assert_eq!(
            resolve_packs_root(&root, &settings),
            root.join("custom-packs")
        );

        let abs_packs = std::env::temp_dir().join("abs-packs");
        settings.packs_path = Some(abs_packs.to_string_lossy().into_owned());
        assert_eq!(resolve_packs_root(&root, &settings), abs_packs);

        fs::remove_dir_all(root).expect("cleanup temp dir");
    }

    #[test]
    fn use_pack_adds_manifest_definition_and_aliases() {
        let root = make_temp_dir("use-pack-context");
        let packs_root = root.join("packs");
        let pack_dir = packs_root.join("testpack");
        fs::create_dir_all(pack_dir.join("lowering")).expect("create pack dirs");
        fs::write(
            pack_dir.join("pack.toml"),
            r#"name = "testpack"
version = "0.1.0"
[entrypoints]
aliases = "aliases.toml"
"#,
        )
        .expect("write manifest");
        fs::write(
            pack_dir.join("aliases.toml"),
            r#"[aliases]
Lease = "core.Contract"
Debt = "core.Debt"
"#,
        )
        .expect("write aliases");

        let source = r#"version 0.1
model "demo"
use pack "testpack" version "0.1.0"
time calendar monthly from 2026-01 for 12
phase base from 2026-01 to 2026-12
entity legal borrower
stream cre.rent on entity legal.borrower {
  schedule on phase_enter("base")
}
"#;
        let model_file = root.join("model.cfdl");
        fs::write(&model_file, source.as_bytes()).expect("write model");
        let model_uri = Url::from_file_path(&model_file).expect("uri");

        let settings = LspSettings::default();
        let context = build_analysis_context(&root, &settings).expect("analysis");
        let active = context.pack_context.active_pack.expect("active pack");
        assert_eq!(active.name, "testpack");
        assert_eq!(active.version, "0.1.0");
        assert_eq!(
            active.aliases.get("Lease"),
            Some(&"core.Contract".to_string())
        );

        let use_pack_pos = position_of_first(source, "testpack");
        let manifest_def = context
            .symbol_index
            .lookup(&model_uri, use_pack_pos)
            .expect("manifest definition");
        assert!(manifest_def
            .uri
            .as_str()
            .ends_with("/packs/testpack/pack.toml"));

        fs::remove_dir_all(root).expect("cleanup temp dir");
    }

    #[test]
    fn hover_documents_expression_builtins() {
        let text = "stream a.b on entity a.b {\n  amount = pmt(0.005, 360, 100000)\n}\n";
        // cursor on "pmt" (line 1, col 11)
        let hover = hover_at(text, Position::new(1, 11)).expect("hover on builtin");
        match hover.contents {
            HoverContents::Markup(m) => {
                assert!(m.value.contains("pmt(rate, nper, pv"), "{}", m.value)
            }
            other => panic!("unexpected hover contents: {other:?}"),
        }
        // cursor on "amount" (not a builtin) -> no hover
        assert!(hover_at(text, Position::new(1, 3)).is_none());
    }

    #[test]
    fn hover_documents_namespace_vars() {
        let text = "  active when time.t >= 6\n";
        let hover = hover_at(text, Position::new(0, 17)).expect("hover on time.t");
        match hover.contents {
            HoverContents::Markup(m) => assert!(m.value.contains("period index")),
            other => panic!("unexpected hover contents: {other:?}"),
        }
    }

    #[test]
    fn completion_includes_expression_builtins() {
        let root = make_temp_dir("builtin-completion");
        fs::write(
            root.join("model.cfdl"),
            b"version 0.1\nmodel \"demo\"\ntime calendar monthly from 2026-01 for 2\n",
        )
        .expect("write model");
        let settings = LspSettings::default();
        let context = build_analysis_context(&root, &settings).expect("analysis");
        let items = completion_items(&context);
        let pmt = items
            .iter()
            .find(|i| i.label == "pmt")
            .expect("pmt completion");
        assert_eq!(pmt.kind, Some(CompletionItemKind::FUNCTION));
        assert!(pmt.detail.as_deref().unwrap_or("").contains("rate, nper"));
        fs::remove_dir_all(root).expect("cleanup temp dir");
    }

    #[test]
    fn completion_items_include_sorted_pack_aliases() {
        let root = make_temp_dir("pack-completion");
        let packs_root = root.join("packs");
        let pack_dir = packs_root.join("testpack");
        fs::create_dir_all(&pack_dir).expect("create pack dir");
        fs::write(
            pack_dir.join("pack.toml"),
            r#"name = "testpack"
version = "0.1.0"
[entrypoints]
aliases = "aliases.toml"
"#,
        )
        .expect("write manifest");
        fs::write(
            pack_dir.join("aliases.toml"),
            r#"[aliases]
ZZZ = "core.z"
AAA = "core.a"
"#,
        )
        .expect("write aliases");
        fs::write(
            root.join("model.cfdl"),
            r#"version 0.1
model "demo"
use pack "testpack" version "0.1.0"
time calendar monthly from 2026-01 for 12
entity legal borrower
stream cre.rent on entity legal.borrower
"#
            .as_bytes(),
        )
        .expect("write model");
        let settings = LspSettings::default();
        let context = build_analysis_context(&root, &settings).expect("analysis");
        let labels = completion_items(&context)
            .into_iter()
            .map(|item| item.label)
            .collect::<Vec<_>>();
        let a_idx = labels
            .iter()
            .position(|label| label == "AAA")
            .expect("AAA completion");
        let z_idx = labels
            .iter()
            .position(|label| label == "ZZZ")
            .expect("ZZZ completion");
        assert!(a_idx < z_idx);

        fs::remove_dir_all(root).expect("cleanup temp dir");
    }

    #[test]
    fn completion_items_include_pack_templates() {
        let root = make_temp_dir("template-completion");
        let packs_root = root.join("packs");
        let pack_dir = packs_root.join("testpack");
        fs::create_dir_all(&pack_dir).expect("create pack dir");
        fs::write(
            pack_dir.join("pack.toml"),
            r#"name = "testpack"
version = "0.1.0"
[entrypoints]
templates = "templates.toml"
"#,
        )
        .expect("write manifest");
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
            root.join("model.cfdl"),
            r#"version 0.1
model "demo"
use pack "testpack" version "0.1.0"
time calendar monthly from 2026-01 for 12
entity legal borrower
stream cre.rent on entity legal.borrower
"#
            .as_bytes(),
        )
        .expect("write model");
        let settings = LspSettings::default();
        let context = build_analysis_context(&root, &settings).expect("analysis");
        let labels = completion_items(&context)
            .into_iter()
            .map(|item| item.label)
            .collect::<Vec<_>>();
        assert!(labels.iter().any(|label| label == "Lease Basic"));

        fs::remove_dir_all(root).expect("cleanup temp dir");
    }

    #[test]
    fn parse_apply_template_request_handles_payload() {
        let parsed = parse_apply_template_request(&json!({
            "uri": "file:///tmp/model.cfdl",
            "templateId": "lease.basic",
            "params": {
                "name": "lease_a",
                "term_start": "2026-01",
                "periods": 12
            }
        }))
        .expect("request parsed");
        let ApplyTemplateRequest {
            uri,
            template_id,
            params,
        } = parsed;
        assert_eq!(uri.as_str(), "file:///tmp/model.cfdl");
        assert_eq!(template_id, "lease.basic");
        assert_eq!(params.get("name"), Some(&"lease_a".to_string()));
        assert_eq!(params.get("periods"), Some(&"12".to_string()));
    }

    #[test]
    fn command_uri_accepts_plain_string_and_object() {
        assert_eq!(
            command_uri(&json!("file:///tmp/plain.cfdl"))
                .expect("uri")
                .as_str(),
            "file:///tmp/plain.cfdl"
        );
        assert_eq!(
            command_uri(&json!({ "uri": "file:///tmp/object.cfdl" }))
                .expect("uri")
                .as_str(),
            "file:///tmp/object.cfdl"
        );
    }

    fn position_of_first(source: &str, needle: &str) -> Position {
        let offset = source.find(needle).expect("needle present");
        offset_to_position(source, offset)
    }

    fn position_of_last(source: &str, needle: &str) -> Position {
        let offset = source.rfind(needle).expect("needle present");
        offset_to_position(source, offset)
    }

    fn offset_to_position(source: &str, offset: usize) -> Position {
        let mut line = 0u32;
        let mut character = 0u32;
        for (idx, ch) in source.char_indices() {
            if idx == offset {
                return Position::new(line, character);
            }
            if ch == '\n' {
                line += 1;
                character = 0;
            } else {
                character += 1;
            }
        }
        Position::new(line, character)
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
