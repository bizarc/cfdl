import * as fs from "node:fs";
import * as path from "node:path";
import * as vscode from "vscode";
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind,
} from "vscode-languageclient/node";

let client: LanguageClient | undefined;
const LIST_TEMPLATES_COMMAND = "cfdl.listTemplates";
const APPLY_TEMPLATE_COMMAND = "cfdl.applyTemplate";

export async function activate(context: vscode.ExtensionContext): Promise<void> {
  const serverPath = resolveServerPath();
  if (!serverPath) {
    void vscode.window.showErrorMessage(
      "CFDL: could not locate `cfdl-lsp`. Set `cfdl.serverPath` to an absolute binary path."
    );
    return;
  }

  const serverOptions: ServerOptions = {
    command: serverPath,
    transport: TransportKind.stdio,
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ language: "cfdl" }],
  };

  client = new LanguageClient("cfdl-lsp", "CFDL Language Server", serverOptions, clientOptions);
  context.subscriptions.push(client);
  await client.start();
  context.subscriptions.push(
    vscode.commands.registerTextEditorCommand("cfdl.applyPackTemplate", async (editor) => {
      if (!client) {
        return;
      }
      if (editor.document.languageId !== "cfdl") {
        void vscode.window.showWarningMessage("CFDL: active editor is not a CFDL document.");
        return;
      }
      const uri = editor.document.uri.toString();
      const templates = (await client.sendRequest("workspace/executeCommand", {
        command: LIST_TEMPLATES_COMMAND,
        arguments: [{ uri }],
      })) as Array<{ id: string; label: string; kind: string; pack: string }> | undefined;
      if (!templates || templates.length === 0) {
        void vscode.window.showInformationMessage("CFDL: no pack templates available.");
        return;
      }
      const pick = await vscode.window.showQuickPick(
        templates.map((template) => ({
          label: template.label,
          description: template.id,
          detail: `${template.kind} (${template.pack})`,
          templateId: template.id,
        })),
        { placeHolder: "Select a pack template to insert" }
      );
      if (!pick) {
        return;
      }
      const rawParams = await vscode.window.showInputBox({
        prompt: "Template parameters as JSON object (optional)",
        value: "{}",
      });
      if (rawParams === undefined) {
        return;
      }
      let params: Record<string, string> = {};
      const trimmed = rawParams.trim();
      if (trimmed.length > 0) {
        try {
          const parsed = JSON.parse(trimmed) as Record<string, unknown>;
          params = Object.fromEntries(
            Object.entries(parsed).map(([key, value]) => [key, String(value)])
          );
        } catch {
          void vscode.window.showErrorMessage(
            "CFDL: template params must be valid JSON object."
          );
          return;
        }
      }
      const response = (await client.sendRequest("workspace/executeCommand", {
        command: APPLY_TEMPLATE_COMMAND,
        arguments: [{ uri, templateId: pick.templateId, params }],
      })) as { text?: string } | undefined;
      const text = response?.text;
      if (!text) {
        void vscode.window.showErrorMessage("CFDL: template expansion failed.");
        return;
      }
      await editor.edit((editBuilder) => {
        editBuilder.replace(editor.selection, text);
      });
    })
  );
}

export async function deactivate(): Promise<void> {
  if (client) {
    await client.stop();
    client = undefined;
  }
}

function resolveServerPath(): string | undefined {
  const configuredPath = vscode.workspace.getConfiguration("cfdl").get<string>("serverPath");
  if (configuredPath && configuredPath.trim().length > 0) {
    if (path.isAbsolute(configuredPath) && fs.existsSync(configuredPath)) {
      return configuredPath;
    }
    return undefined;
  }

  const workspaceFolder = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
  if (!workspaceFolder) {
    return undefined;
  }

  const binaryName = process.platform === "win32" ? "cfdl-lsp.exe" : "cfdl-lsp";
  const fallbackPath = path.join(workspaceFolder, "target", "debug", binaryName);
  if (fs.existsSync(fallbackPath)) {
    return fallbackPath;
  }
  return undefined;
}
