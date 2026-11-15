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
        const nameMatch = manifestContent.match(/(?:\[info\]|\[package\])[^[]*?name\s*=\s*["']([^"']+)["']/s);
        if (nameMatch && nameMatch[1]) {
            return nameMatch[1].trim();
        }
    } catch {
        // Fall back to directory basename
    }
    return path.basename(projectRoot);
}

/**
 * Extracts declared dependencies from manifest.toml content as "name@version" pairs.
 */
function parseManifestDependencies(manifestContent: string): string[] {
    const dependencies: string[] = [];
    const depsMatch = manifestContent.match(/\[dependencies\]([\s\S]*?)(?:\[|$)/);
    if (depsMatch && depsMatch[1]) {
        const lines = depsMatch[1].split('\n');
        for (const line of lines) {
            const trimmedLine = line.trim();
            if (!trimmedLine || trimmedLine.startsWith('#')) {
                continue;
            }
            const match = trimmedLine.match(/^([a-zA-Z0-9_\-\.]+)\s*=\s*["']([^"']+)["']/);
            if (match && match[1] && match[2]) {
                dependencies.push(`${match[1]}@${match[2]}`);
            }
        }
    }
    return dependencies;
}

/**
 * Extracts canonical entry point symbol from manifest.toml if specified.
 */
function parseManifestEntryPoint(manifestContent: string): string | undefined {
    const entryMatch = manifestContent.match(/\[entrypoints\][\s\S]*?main\s*=\s*["']([^"']+)["']/);
    return entryMatch && entryMatch[1] ? entryMatch[1].trim() : undefined;
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
                    const extraArguments: string[] = [];

                    if (projectRoot) {
                        const manifestPath = path.join(projectRoot, 'manifest.toml');
                        if (fs.existsSync(manifestPath)) {
                            try {
                                const manifestContent = fs.readFileSync(manifestPath, 'utf8');
                                const dependencies = parseManifestDependencies(manifestContent);
                                for (const dependency of dependencies) {
                                    extraArguments.push('--deps', dependency);
                                }
                                const entryPoint = parseManifestEntryPoint(manifestContent);
                                if (entryPoint) {
                                    extraArguments.push('--entry', entryPoint);
                                }
                            } catch {
                                // Ignore manifest parse errors
                            }
                        }

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
                        ['compile', ...compilationSources, ...extraArguments, '-o', temporaryBinaryPath],
                        { encoding: 'utf8', timeout: 15000 }
                    );
                    inspectionTarget = temporaryBinaryPath;
                } catch (compilationError) {
                    // Fall back to existing compiled artifact if available in project build folder
                    let fallbackResolved = false;
                    const projectRoot = findProjectRoot(sourceFilePath);
                    if (projectRoot) {
                        const packageName = readPackageName(projectRoot);
                        const candidatePaths = [
                            path.join(projectRoot, 'build', activeBuildProfile, `${packageName}.slxarc`),
                            path.join(projectRoot, 'build', 'release', `${packageName}.slxarc`),
                            path.join(projectRoot, 'build', 'debug', `${packageName}.slxarc`),
                            path.join(projectRoot, 'build', '.cache', 'stage', activeBuildProfile, 'bin', `${packageName}.slxb`),
                            path.join(projectRoot, 'build', '.cache', 'stage', 'release', 'bin', `${packageName}.slxb`),
                            path.join(projectRoot, 'build', '.cache', 'stage', 'debug', 'bin', `${packageName}.slxb`),
                        ];
                        for (const candidatePath of candidatePaths) {
                            if (fs.existsSync(candidatePath)) {
                                inspectionTarget = candidatePath;
                                fallbackResolved = true;
                                break;
                            }
                        }
                    }
                    if (!fallbackResolved) {
                        return `// Compilation Failed for inspection of ${sourceFilePath}:\n// ${String(compilationError)}`;
                    }
                }
            }

            const executionOutput = childProcess.execFileSync(
                this.solixBinaryPath,
                ['info', this.commandFlag, inspectionTarget],
                { encoding: 'utf8', timeout: 15000 }
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
                    terminal.sendText(`cd "${projectRoot}" && "${solixBinaryPath}" build --release && "${solixBinaryPath}" run "build/release/${packageName}.slxarc"`);
                } else {
                    terminal.sendText(`"${solixBinaryPath}" run "${targetFilePath}"`);
                }
            } else {
                terminal.sendText(`"${solixBinaryPath}" run`);
            }
        })
    );

    // 1b. solix.debugMain
    extensionContext.subscriptions.push(
        vscode.commands.registerCommand('solix.debugMain', async (documentArgument?: string | vscode.Uri) => {
            await vscode.workspace.saveAll(false);
            const targetUri = resolveTargetUri(documentArgument);
            const targetPath = targetUri?.fsPath || vscode.window.activeTextEditor?.document.uri.fsPath;
            await vscode.debug.startDebugging(undefined, {
                type: 'solix',
                name: 'Debug Solix Main',
                request: 'launch',
                program: targetPath,
                stopOnEntry: false,
                testMode: false
            });
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

    // 2b. solix.debugTest
    extensionContext.subscriptions.push(
        vscode.commands.registerCommand('solix.debugTest', async (documentArgument?: string | vscode.Uri) => {
            await vscode.workspace.saveAll(false);
            const targetUri = resolveTargetUri(documentArgument);
            const targetPath = targetUri?.fsPath || vscode.window.activeTextEditor?.document.uri.fsPath;
            await vscode.debug.startDebugging(undefined, {
                type: 'solix',
                name: 'Debug Solix Test',
                request: 'launch',
                program: targetPath,
                stopOnEntry: false,
                testMode: true
            });
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
                    terminal.sendText(`cd "${projectRoot}" && "${solixBinaryPath}" reinstall --all`);
                    return;
                }
            }
            terminal.sendText(`"${solixBinaryPath}" reinstall --all`);
        })
    );

    // 5. solix.selectProfile
    profileStatusBarItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right, 99);
    profileStatusBarItem.command = 'solix.selectProfile';
    profileStatusBarItem.text = `$(tools) Solix: ${activeBuildProfile}`;
    profileStatusBarItem.tooltip = 'Click to switch active Solix build profile';
    profileStatusBarItem.show();
    extensionContext.subscriptions.push(profileStatusBarItem);

    // 5b. Build Button in Status Bar
    const buildStatusBarItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right, 100);
    buildStatusBarItem.command = 'solix.buildProject';
    buildStatusBarItem.text = '$(gear) Build';
    buildStatusBarItem.tooltip = `Build Solix Project (solix build --${activeBuildProfile})`;
    buildStatusBarItem.show();
    extensionContext.subscriptions.push(buildStatusBarItem);

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
                buildStatusBarItem.tooltip = `Build Solix Project (solix build --${activeBuildProfile})`;
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

    // 9. solix.showReferences (BUG-06)
    extensionContext.subscriptions.push(
        vscode.commands.registerCommand('solix.showReferences', async (uriStr?: string, position?: { line: number; character: number }) => {
            const activeEditor = vscode.window.activeTextEditor;
            const targetUri = uriStr ? vscode.Uri.parse(uriStr) : activeEditor?.document.uri;
            if (!targetUri) {
                return;
            }

            const targetPosition = position
                ? new vscode.Position(position.line, position.character)
                : activeEditor?.selection.active;
            if (!targetPosition) {
                return;
            }

            const locations = await vscode.commands.executeCommand<vscode.Location[]>(
                'vscode.executeReferenceProvider',
                targetUri,
                targetPosition
            );

            await vscode.commands.executeCommand(
                'editor.action.showReferences',
                targetUri,
                targetPosition,
                locations ?? []
            );
        })
    );
}
