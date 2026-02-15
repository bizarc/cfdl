use std::collections::HashMap;
use std::sync::Arc;

use lsp_types::{
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    InitializeParams, InitializeResult, MessageType, ServerCapabilities,
    TextDocumentSyncCapability, TextDocumentSyncKind, Url,
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
}

impl Backend {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            docs: Arc::new(RwLock::new(DocumentStore::default())),
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
        let mut docs = self.docs.write().await;
        docs.open(params.text_document.uri, params.text_document.text);
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let Some(change) = params.content_changes.into_iter().next() else {
            return;
        };
        let mut docs = self.docs.write().await;
        docs.change_full(&params.text_document.uri, change.text);
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let mut docs = self.docs.write().await;
        docs.close(&params.text_document.uri);
    }
}

#[cfg(test)]
mod tests {
    use super::{server_capabilities, DocumentStore};
    use lsp_types::{TextDocumentSyncCapability, TextDocumentSyncKind, Url};

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
}
