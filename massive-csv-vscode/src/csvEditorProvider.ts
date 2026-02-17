import * as vscode from "vscode";
import * as path from "path";
import * as fs from "fs";
import { CsvDocument, openCsvDocument } from "./backend";

export class CsvEditorProvider implements vscode.CustomReadonlyEditorProvider {
  public static readonly viewType = "massiveCsv.editor";

  constructor(private readonly context: vscode.ExtensionContext) {}

  public openCustomDocument(
    uri: vscode.Uri,
    _openContext: vscode.CustomDocumentOpenContext,
    _token: vscode.CancellationToken
  ): vscode.CustomDocument {
    return { uri, dispose: () => {} };
  }

  public resolveCustomEditor(
    document: vscode.CustomDocument,
    webviewPanel: vscode.WebviewPanel,
    _token: vscode.CancellationToken
  ): void {
    const filePath = document.uri.fsPath;

    let doc: CsvDocument;
    try {
      doc = openCsvDocument(filePath);
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      vscode.window.showErrorMessage(`Failed to open CSV: ${msg}`);
      return;
    }

    const info = doc.getInfo();

    // Configure webview
    webviewPanel.webview.options = {
      enableScripts: true,
      localResourceRoots: [
        vscode.Uri.file(path.join(this.context.extensionPath, "media")),
      ],
    };

    // Load webview HTML
    const mediaPath = path.join(this.context.extensionPath, "media");
    const htmlPath = path.join(mediaPath, "webview.html");
    let html = fs.readFileSync(htmlPath, "utf-8");

    // Replace resource URIs for CSP
    const webviewUri = (file: string) =>
      webviewPanel.webview
        .asWebviewUri(vscode.Uri.file(path.join(mediaPath, file)))
        .toString();

    // Set CSP nonce for inline scripts
    const nonce = getNonce();
    html = html.replace(/{{nonce}}/g, nonce);
    html = html.replace(
      /{{cspSource}}/g,
      webviewPanel.webview.cspSource
    );

    webviewPanel.webview.html = html;

    // Send initial data once webview is ready
    const sendInit = () => {
      webviewPanel.webview.postMessage({
        type: "init",
        headers: info.headers,
        rowCount: info.rowCount,
        delimiter: info.delimiter,
        filePath: info.filePath,
        fileSize: getFileSize(filePath),
      });
    };

    // Handle messages from webview
    webviewPanel.webview.onDidReceiveMessage(
      (message) => {
        switch (message.type) {
          case "ready":
            sendInit();
            break;

          case "getRows": {
            try {
              const rows = doc.getRows(message.start, message.end);
              webviewPanel.webview.postMessage({
                type: "rowData",
                requestId: message.requestId,
                start: message.start,
                rows,
              });
            } catch (err: unknown) {
              const msg = err instanceof Error ? err.message : String(err);
              vscode.window.showErrorMessage(`Error reading rows: ${msg}`);
            }
            break;
          }

          case "search": {
            try {
              const results = doc.search(message.query, {
                column: message.column || undefined,
                caseSensitive: message.caseSensitive,
                maxResults: message.maxResults || 1000,
              });
              webviewPanel.webview.postMessage({
                type: "searchResults",
                results,
                query: message.query,
              });
            } catch (err: unknown) {
              const msg = err instanceof Error ? err.message : String(err);
              vscode.window.showErrorMessage(`Search error: ${msg}`);
            }
            break;
          }

          case "editCell": {
            try {
              doc.setCell(message.row, message.col, message.value);
              webviewPanel.webview.postMessage({
                type: "editAck",
                row: message.row,
                col: message.col,
                editCount: doc.editCount,
              });
            } catch (err: unknown) {
              const msg = err instanceof Error ? err.message : String(err);
              vscode.window.showErrorMessage(`Edit error: ${msg}`);
            }
            break;
          }

          case "save": {
            try {
              doc.save();
              const newInfo = doc.getInfo();
              webviewPanel.webview.postMessage({
                type: "saveComplete",
                rowCount: newInfo.rowCount,
              });
              vscode.window.showInformationMessage("CSV saved successfully.");
            } catch (err: unknown) {
              const msg = err instanceof Error ? err.message : String(err);
              vscode.window.showErrorMessage(`Save error: ${msg}`);
            }
            break;
          }

          case "revertAll": {
            doc.revertAll();
            webviewPanel.webview.postMessage({
              type: "revertComplete",
              editCount: 0,
            });
            break;
          }
        }
      },
      undefined,
      this.context.subscriptions
    );
  }
}

function getNonce(): string {
  let text = "";
  const possible =
    "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
  for (let i = 0; i < 32; i++) {
    text += possible.charAt(Math.floor(Math.random() * possible.length));
  }
  return text;
}

function getFileSize(filePath: string): string {
  try {
    const stats = fs.statSync(filePath);
    const bytes = stats.size;
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    if (bytes < 1024 * 1024 * 1024)
      return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
    return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
  } catch {
    return "unknown";
  }
}
