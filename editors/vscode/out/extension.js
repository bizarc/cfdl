"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
exports.activate = activate;
exports.deactivate = deactivate;
const fs = __importStar(require("node:fs"));
const path = __importStar(require("node:path"));
const vscode = __importStar(require("vscode"));
const node_1 = require("vscode-languageclient/node");
let client;
async function activate(context) {
    const serverPath = resolveServerPath();
    if (!serverPath) {
        void vscode.window.showErrorMessage("CFDL: could not locate `cfdl-lsp`. Set `cfdl.serverPath` to an absolute binary path.");
        return;
    }
    const serverOptions = {
        command: serverPath,
        transport: node_1.TransportKind.stdio,
    };
    const clientOptions = {
        documentSelector: [{ language: "cfdl" }],
    };
    client = new node_1.LanguageClient("cfdl-lsp", "CFDL Language Server", serverOptions, clientOptions);
    context.subscriptions.push(client);
    await client.start();
}
async function deactivate() {
    if (client) {
        await client.stop();
        client = undefined;
    }
}
function resolveServerPath() {
    const configuredPath = vscode.workspace.getConfiguration("cfdl").get("serverPath");
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
//# sourceMappingURL=extension.js.map