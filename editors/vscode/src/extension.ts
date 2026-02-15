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
