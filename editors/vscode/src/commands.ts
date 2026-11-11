import * as vscode from 'vscode';
import * as childProcess from 'child_process';
import * as path from 'path';
import * as fs from 'fs';
import * as os from 'os';
import { LanguageClient } from 'vscode-languageclient/node';

/**
 * Text document content provider displaying synthesized read-only virtual buffers.
 */
class VirtualContentProvider implements vscode.TextDocumentContentProvider {
    private readonly onDidChangeEmitter = new vscode.EventEmitter<vscode.Uri>();
    public readonly onDidChange = this.onDidChangeEmitter.event;

    constructor(
        private readonly commandFlag: string,
        private readonly solixBinaryPath: string
    ) {}

    public provideTextDocumentContent(documentUri: vscode.Uri): string {
        let sourceFilePath = documentUri.path;
        if (sourceFilePath.endsWith('.slxdasm')) {
            sourceFilePath = sourceFilePath.slice(0, -'.slxdasm'.length);
        } else if (sourceFilePath.endsWith('.api.slx')) {
            sourceFilePath = sourceFilePath.slice(0, -'.api.slx'.length);
        }

        // If the target is a Solix source file (.slx), compile it to a temporary binary container first
        let inspectionTarget = sourceFilePath;
        let temporaryBinaryPath: string | undefined;

        try {
            if (sourceFilePath.endsWith('.slx')) {
                const temporaryDirectory = os.tmpdir();
                const randomSuffix = Math.floor(Math.random() * 1000000);
                temporaryBinaryPath = path.join(temporaryDirectory, `solix_inspect_${randomSuffix}.slxb`);

                try {
                    childProcess.execFileSync(
                        this.solixBinaryPath,
                        ['compile', sourceFilePath, '-o', temporaryBinaryPath],
                        { encoding: 'utf8', timeout: 8000 }
                    );
                    inspectionTarget = temporaryBinaryPath;
                } catch (compilationError) {
                    return `// Compilation Failed for inspection of ${sourceFilePath}:\n// ${String(compilationError)}`;
                }
            }

            const executionOutput = childProcess.execFileSync(
                this.solixBinaryPath,
                ['info', this.commandFlag, inspectionTarget],
                { encoding: 'utf8', timeout: 5000 }
            );
            return executionOutput;
        } catch (executionError) {
            return `// Failed to inspect ${this.commandFlag} for ${sourceFilePath}:\n// ${String(executionError)}`;
        } finally {
            if (temporaryBinaryPath && fs.existsSync(temporaryBinaryPath)) {
                try {
                    fs.unlinkSync(temporaryBinaryPath);
                } catch {
                    // Suppress temporary cleanup failure
                }
            }
        }
    }

    public refresh(targetUri: vscode.Uri): void {
        this.onDidChangeEmitter.fire(targetUri);
    }
}

/**
 * Registers all client-side Solix commands, virtual document providers, and CodeLens actions.
 */
export function registerSolixCommands(
    extensionContext: vscode.ExtensionContext,
    languageClient: LanguageClient,
    solixBinaryPath: string
): void {
    let activeTerminal: vscode.Terminal | undefined;

    const getOrCreateTerminal = (terminalName: string): vscode.Terminal => {
        if (!activeTerminal || activeTerminal.exitStatus !== undefined) {
            activeTerminal = vscode.window.createTerminal(terminalName);
        }
        return activeTerminal;
    };

    // 1. solix.runMain
    extensionContext.subscriptions.push(
        vscode.commands.registerCommand('solix.runMain', async (documentUri?: vscode.Uri) => {
            const terminal = getOrCreateTerminal('Solix Run');
            terminal.show();

            const targetPath = documentUri ? documentUri.fsPath : '';
            if (targetPath) {
                terminal.sendText(`${solixBinaryPath} run "${targetPath}"`);
            } else {
                terminal.sendText(`${solixBinaryPath} run`);
            }
        })
    );

    // 2. solix.runTest
    extensionContext.subscriptions.push(
        vscode.commands.registerCommand('solix.runTest', async () => {
            const terminal = getOrCreateTerminal('Solix Test');
            terminal.show();
            terminal.sendText(`${solixBinaryPath} run --test`);
        })
    );

    // 3. solix.buildProject
    extensionContext.subscriptions.push(
        vscode.commands.registerCommand('solix.buildProject', async () => {
            const terminal = getOrCreateTerminal('Solix Build');
            terminal.show();
            terminal.sendText(`${solixBinaryPath} build`);
        })
    );

    // 4. solix.syncPackages
    extensionContext.subscriptions.push(
        vscode.commands.registerCommand('solix.syncPackages', async () => {
            const terminal = getOrCreateTerminal('Solix Sync');
            terminal.show();
            terminal.sendText(`${solixBinaryPath} sync --all`);
        })
    );

    // 5. Virtual Provider: Disassembly (solix-disasm://)
    const disasmProvider = new VirtualContentProvider('--disasm', solixBinaryPath);
    extensionContext.subscriptions.push(
        vscode.workspace.registerTextDocumentContentProvider('solix-disasm', disasmProvider)
    );

    extensionContext.subscriptions.push(
        vscode.commands.registerCommand('solix.showDisassembly', async () => {
            const activeEditor = vscode.window.activeTextEditor;
            if (!activeEditor) {
                vscode.window.showInformationMessage('No active Solix document to disassemble.');
                return;
            }

            const activeUri = activeEditor.document.uri;
            const virtualUri = vscode.Uri.parse(`solix-disasm:${activeUri.fsPath}.slxdasm`);
            disasmProvider.refresh(virtualUri);

            const virtualDocument = await vscode.workspace.openTextDocument(virtualUri);
            await vscode.window.showTextDocument(virtualDocument, {
                viewColumn: vscode.ViewColumn.Beside,
                preserveFocus: true,
            });
        })
    );

    // 6. Virtual Provider: Synthesized API Header (solix-api://)
    const apiProvider = new VirtualContentProvider('--api', solixBinaryPath);
    extensionContext.subscriptions.push(
        vscode.workspace.registerTextDocumentContentProvider('solix-api', apiProvider)
    );

    extensionContext.subscriptions.push(
        vscode.commands.registerCommand('solix.showSynthesizedApi', async () => {
            const activeEditor = vscode.window.activeTextEditor;
            if (!activeEditor) {
                vscode.window.showInformationMessage('No active Solix document to inspect API.');
                return;
            }

            const activeUri = activeEditor.document.uri;
            const virtualUri = vscode.Uri.parse(`solix-api:${activeUri.fsPath}.api.slx`);
            apiProvider.refresh(virtualUri);

            const virtualDocument = await vscode.workspace.openTextDocument(virtualUri);
            await vscode.window.showTextDocument(virtualDocument, {
                viewColumn: vscode.ViewColumn.Beside,
                preserveFocus: true,
            });
        })
    );

    // 7. solix.restartServer
    extensionContext.subscriptions.push(
        vscode.commands.registerCommand('solix.restartServer', async () => {
            vscode.window.showInformationMessage('Restarting Solix Language Server...');
            try {
                if (languageClient.isRunning()) {
                    await languageClient.restart();
                } else {
                    await languageClient.start();
                }
                vscode.window.showInformationMessage('Solix Language Server restarted.');
            } catch (restartError) {
                vscode.window.showErrorMessage(`Failed to restart server: ${String(restartError)}`);
            }
        })
    );
}
