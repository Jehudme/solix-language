import * as vscode from 'vscode';
import * as childProcess from 'child_process';
import * as path from 'path';
import * as fs from 'fs';
import * as os from 'os';
import { LanguageClient } from 'vscode-languageclient/node';

let activeBuildProfile: 'debug' | 'release' | 'test' = 'debug';
let profileStatusBarItem: vscode.StatusBarItem | undefined;

/**
 * Searches parent directory tree starting from target path for a manifest.toml file.
 */
function findProjectRoot(initialFilePath: string): string | undefined {
    try {
        let currentDirectory = fs.statSync(initialFilePath).isDirectory()
            ? initialFilePath
            : path.dirname(initialFilePath);

        for (let depthIndex = 0; depthIndex < 10; depthIndex += 1) {
            const manifestCandidate = path.join(currentDirectory, 'manifest.toml');
            if (fs.existsSync(manifestCandidate)) {
                return currentDirectory;
            }
            const parentDirectory = path.dirname(currentDirectory);
            if (parentDirectory === currentDirectory) {
                break;
            }
            currentDirectory = parentDirectory;
        }
    } catch {
        return undefined;
    }
    return undefined;
}

/**
 * Reads the package name defined under [package] name = "..." in manifest.toml.
 */
function readPackageName(projectRoot: string): string {
    const manifestPath = path.join(projectRoot, 'manifest.toml');
    try {
        const manifestContent = fs.readFileSync(manifestPath, 'utf8');
        const nameMatch = manifestContent.match(/\[package\][^[]*?name\s*=\s*["']([^"']+)["']/s);
        if (nameMatch && nameMatch[1]) {
            return nameMatch[1].trim();
        }
    } catch {
        // Fall back to directory basename
    }
    return path.basename(projectRoot);
}

/**
 * Resolves a target document URI from CodeLens arguments or active text editor.
 */
function resolveTargetUri(documentArgument?: string | vscode.Uri): vscode.Uri | undefined {
    if (typeof documentArgument === 'string') {
        try {
            return vscode.Uri.parse(documentArgument);
        } catch {
            return undefined;
        }
    } else if (documentArgument instanceof vscode.Uri) {
        return documentArgument;
    }
    return vscode.window.activeTextEditor?.document.uri;
}

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
        let sourceFilePath = documentUri.fsPath;
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
                    const projectRoot = findProjectRoot(sourceFilePath);
                    let compilationSources = [sourceFilePath];

                    if (projectRoot) {
                        const sourceDirectory = path.join(projectRoot, 'src');
                        if (fs.existsSync(sourceDirectory)) {
                            const discoveredFiles = fs.readdirSync(sourceDirectory)
                                .filter((fileName) => fileName.endsWith('.slx'))
                                .map((fileName) => path.join(sourceDirectory, fileName));
                            if (discoveredFiles.length > 0) {
                                compilationSources = discoveredFiles;
                            }
                        }
                    }

                    childProcess.execFileSync(
                        this.solixBinaryPath,
                        ['compile', ...compilationSources, '-o', temporaryBinaryPath],
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
        vscode.commands.registerCommand('solix.runMain', async (documentArgument?: string | vscode.Uri) => {
            await vscode.workspace.saveAll(false);
            const targetUri = resolveTargetUri(documentArgument);
            const terminal = getOrCreateTerminal('Solix Run');
            terminal.show();

            if (targetUri && targetUri.fsPath) {
                const targetFilePath = targetUri.fsPath;
                const projectRoot = findProjectRoot(targetFilePath);
                if (projectRoot) {
                    const packageName = readPackageName(projectRoot);
                    terminal.sendText(`cd "${projectRoot}" && "${solixBinaryPath}" build --${activeBuildProfile} && "${solixBinaryPath}" run "build/${activeBuildProfile}/${packageName}.slxarc"`);
                } else {
                    terminal.sendText(`"${solixBinaryPath}" run "${targetFilePath}"`);
                }
            } else {
                terminal.sendText(`"${solixBinaryPath}" run`);
            }
        })
    );

    // 2. solix.runTest
    extensionContext.subscriptions.push(
        vscode.commands.registerCommand('solix.runTest', async (documentArgument?: string | vscode.Uri) => {
            await vscode.workspace.saveAll(false);
            const targetUri = resolveTargetUri(documentArgument);
            const terminal = getOrCreateTerminal('Solix Test');
            terminal.show();

            if (targetUri && targetUri.fsPath) {
                const targetFilePath = targetUri.fsPath;
                const projectRoot = findProjectRoot(targetFilePath);
                if (projectRoot) {
                    const packageName = readPackageName(projectRoot);
                    terminal.sendText(`cd "${projectRoot}" && "${solixBinaryPath}" build --test && "${solixBinaryPath}" run --test "build/test/${packageName}.slxarc"`);
                } else {
                    terminal.sendText(`"${solixBinaryPath}" run --test "${targetFilePath}"`);
                }
            } else {
                terminal.sendText(`"${solixBinaryPath}" run --test`);
            }
        })
    );

    // 3. solix.buildProject
    extensionContext.subscriptions.push(
        vscode.commands.registerCommand('solix.buildProject', async () => {
            await vscode.workspace.saveAll(false);
            const terminal = getOrCreateTerminal('Solix Build');
            terminal.show();

            const activeUri = vscode.window.activeTextEditor?.document.uri;
            if (activeUri && activeUri.fsPath) {
                const projectRoot = findProjectRoot(activeUri.fsPath);
                if (projectRoot) {
                    terminal.sendText(`cd "${projectRoot}" && "${solixBinaryPath}" build --${activeBuildProfile}`);
                    return;
                }
            }
            terminal.sendText(`"${solixBinaryPath}" build --${activeBuildProfile}`);
        })
    );

    // 4. solix.syncPackages
    extensionContext.subscriptions.push(
        vscode.commands.registerCommand('solix.syncPackages', async () => {
            await vscode.workspace.saveAll(false);
            const terminal = getOrCreateTerminal('Solix Sync');
            terminal.show();

            const activeUri = vscode.window.activeTextEditor?.document.uri;
            if (activeUri && activeUri.fsPath) {
                const projectRoot = findProjectRoot(activeUri.fsPath);
                if (projectRoot) {
                    terminal.sendText(`cd "${projectRoot}" && "${solixBinaryPath}" sync --all`);
                    return;
                }
            }
            terminal.sendText(`"${solixBinaryPath}" sync --all`);
        })
    );

    // 5. solix.selectProfile
    profileStatusBarItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right, 99);
    profileStatusBarItem.command = 'solix.selectProfile';
    profileStatusBarItem.text = `$(tools) Solix: ${activeBuildProfile}`;
    profileStatusBarItem.tooltip = 'Click to switch active Solix build profile';
    profileStatusBarItem.show();
    extensionContext.subscriptions.push(profileStatusBarItem);

    extensionContext.subscriptions.push(
        vscode.commands.registerCommand('solix.selectProfile', async () => {
            const profileSelection = await vscode.window.showQuickPick(
                [
                    { label: 'debug', description: 'Development build profile with full debugging symbols' },
                    { label: 'release', description: 'Optimized build profile with bytecode inlining' },
                    { label: 'test', description: 'Test build profile with unit testing harnesses enabled' }
                ],
                { placeHolder: 'Select active Solix build profile' }
            );

            if (profileSelection) {
                activeBuildProfile = profileSelection.label as 'debug' | 'release' | 'test';
                if (profileStatusBarItem) {
                    profileStatusBarItem.text = `$(tools) Solix: ${activeBuildProfile}`;
                }
                vscode.window.showInformationMessage(`Active Solix build profile set to: ${activeBuildProfile}`);
            }
        })
    );

    // 6. Virtual Provider: Disassembly (solix-disasm://)
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

    // 7. Virtual Provider: Synthesized API Header (solix-api://)
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

    // 8. solix.restartServer
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
