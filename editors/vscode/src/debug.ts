import * as vscode from 'vscode';
import * as path from 'path';
import * as fs from 'fs';
import * as childProcess from 'child_process';
import * as net from 'net';

const MANIFEST_FILENAMES = ['manifest.yaml', 'manifest.yml', 'manifest.toml'];

function resolveManifestFile(directory: string): string | undefined {
    for (const filename of MANIFEST_FILENAMES) {
        const candidate = path.join(directory, filename);
        if (fs.existsSync(candidate)) {
            return candidate;
        }
    }
    return undefined;
}

/**
 * Searches parent directories for manifest.yaml (or manifest.toml) to identify project roots.
 */
function findProjectRoot(initialFilePath: string): string | undefined {
    try {
        let currentDirectory = fs.statSync(initialFilePath).isDirectory()
            ? initialFilePath
            : path.dirname(initialFilePath);

        for (let depthIndex = 0; depthIndex < 10; depthIndex += 1) {
            if (resolveManifestFile(currentDirectory)) {
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
 * Reads package name from manifest.yaml or manifest.toml.
 */
function readPackageName(projectRoot: string): string {
    const manifestPath = resolveManifestFile(projectRoot);
    if (!manifestPath) {
        return path.basename(projectRoot);
    }
    try {
        const manifestContent = fs.readFileSync(manifestPath, 'utf8');
        if (manifestPath.endsWith('.yaml') || manifestPath.endsWith('.yml')) {
            const yamlMatch = manifestContent.match(/^configuration:\s*\r?\n(?:\s+.*\r?\n)*?\s+name:\s*["']?([^\s'"#]+)["']?/m);
            if (yamlMatch && yamlMatch[1]) {
                return yamlMatch[1].trim();
            }
        }
        const nameMatch = manifestContent.match(/(?:\[configuration\]|\[info\]|\[package\])[^[]*?name\s*=\s*["']([^"']+)["']/s);
        if (nameMatch && nameMatch[1]) {
            return nameMatch[1].trim();
        }
    } catch {
        // Fall back to directory basename
    }
    return path.basename(projectRoot);
}

function getAvailablePort(): Promise<number> {
    return new Promise((resolve, reject) => {
        const server = net.createServer();
        server.unref();
        server.on('error', reject);
        server.listen(0, '127.0.0.1', () => {
            const address = server.address();
            if (address && typeof address === 'object') {
                const port = address.port;
                server.close(() => resolve(port));
            } else {
                server.close(() => reject(new Error('Failed to obtain ephemeral port')));
            }
        });
    });
}

function waitForTcpPort(port: number, timeoutMs: number = 3000): Promise<void> {
    const startTime = Date.now();
    return new Promise((resolve, reject) => {
        function attempt() {
            const socket = net.createConnection({ host: '127.0.0.1', port }, () => {
                socket.end();
                resolve();
            });
            socket.on('error', () => {
                socket.destroy();
                if (Date.now() - startTime >= timeoutMs) {
                    reject(new Error(`Timeout waiting for Solix DAP on port ${port}`));
                } else {
                    setTimeout(attempt, 25);
                }
            });
        }
        attempt();
    });
}

/**
 * Spawns the Solix Debug Adapter via Integrated Terminal TCP mode or headless stdio.
 */
export class SolixDebugAdapterDescriptorFactory implements vscode.DebugAdapterDescriptorFactory {
    constructor(private readonly solixBinaryPath: string) {}

    public async createDebugAdapterDescriptor(
        session: vscode.DebugSession,
        _executable: vscode.DebugAdapterExecutable | undefined
    ): Promise<vscode.DebugAdapterDescriptor> {
        const useInternalConsole = session.configuration.console === 'internalConsole'
            || process.env.SOLIX_HEADLESS_DAP === '1';

        if (useInternalConsole) {
            return new vscode.DebugAdapterExecutable(
                this.solixBinaryPath,
                ['dap', '--stdio'],
                {
                    env: {
                        ...process.env,
                        RUST_BACKTRACE: '1'
                    }
                }
            );
        }

        const port = await getAvailablePort();
        let terminal = vscode.window.terminals.find((term) => term.name === 'Solix Debug');
        const cwd = session.configuration.cwd || undefined;
        if (!terminal) {
            terminal = vscode.window.createTerminal({
                name: 'Solix Debug',
                cwd
            });
        }
        terminal.show(true);
        terminal.sendText(`"${this.solixBinaryPath}" dap --port ${port}`);
        await waitForTcpPort(port, 3000);
        return new vscode.DebugAdapterServer(port, '127.0.0.1');
    }
}

/**
 * Provides zero-config F5 resolution and automatic pre-launch debug compilation.
 */
export class SolixDebugConfigurationProvider implements vscode.DebugConfigurationProvider {
    constructor(private readonly solixBinaryPath: string) {}

    public provideDebugConfigurations(
        _folder: vscode.WorkspaceFolder | undefined,
        _token?: vscode.CancellationToken
    ): vscode.ProviderResult<vscode.DebugConfiguration[]> {
        return [
            {
                name: 'Debug Solix Program',
                type: 'solix',
                request: 'launch',
                program: '${file}',
                stopOnEntry: false
            }
        ];
    }

    public resolveDebugConfiguration(
        folder: vscode.WorkspaceFolder | undefined,
        debugConfiguration: vscode.DebugConfiguration,
        _token?: vscode.CancellationToken
    ): vscode.ProviderResult<vscode.DebugConfiguration> {
        // If launch.json is missing or empty, configure defaults
        if (!debugConfiguration.type && !debugConfiguration.request && !debugConfiguration.name) {
            const activeEditor = vscode.window.activeTextEditor;
            if (activeEditor && activeEditor.document.languageId === 'solix') {
                debugConfiguration.type = 'solix';
                debugConfiguration.name = 'Debug Current File';
                debugConfiguration.request = 'launch';
                debugConfiguration.program = activeEditor.document.uri.fsPath;
                debugConfiguration.stopOnEntry = false;
            } else if (folder) {
                debugConfiguration.type = 'solix';
                debugConfiguration.name = 'Debug Solix Project';
                debugConfiguration.request = 'launch';
                debugConfiguration.program = folder.uri.fsPath;
                debugConfiguration.stopOnEntry = false;
            } else {
                vscode.window.showErrorMessage('Cannot find a Solix source file or project to debug.');
                return undefined;
            }
        }

        let targetPath = debugConfiguration.program;
        if (!targetPath) {
            const activeEditor = vscode.window.activeTextEditor;
            if (activeEditor) {
                targetPath = activeEditor.document.uri.fsPath;
            } else if (folder) {
                targetPath = folder.uri.fsPath;
            }
        }

        if (targetPath) {
            const projectRoot = findProjectRoot(targetPath);
            if (projectRoot) {
                debugConfiguration.cwd = projectRoot;
                const packageName = readPackageName(projectRoot);
                const buildModeFlag = debugConfiguration.testMode ? '--test' : '--debug';
                const outputFolder = debugConfiguration.testMode ? 'test' : 'debug';

                // Automatically build with debug symbols before launching session
                try {
                    vscode.window.showInformationMessage(`Building Solix project in ${outputFolder} mode before debugging...`);
                    childProcess.execSync(`"${this.solixBinaryPath}" build ${buildModeFlag}`, {
                        cwd: projectRoot,
                        encoding: 'utf8'
                    });
                } catch (buildError) {
                    vscode.window.showErrorMessage(`Solix debug build failed: ${String(buildError)}`);
                    return undefined;
                }

                const expectedArchivePath = path.join(projectRoot, 'build', outputFolder, `${packageName}.slxarc`);
                if (fs.existsSync(expectedArchivePath)) {
                    debugConfiguration.program = expectedArchivePath;
                }
            } else {
                debugConfiguration.program = targetPath;
            }
        }

        return debugConfiguration;
    }
}

/**
 * Provides live variable values inline in the active editor while paused at a breakpoint.
 */
export class SolixInlineValuesProvider implements vscode.InlineValuesProvider {
    public provideInlineValues(
        document: vscode.TextDocument,
        viewPort: vscode.Range,
        context: vscode.InlineValueContext,
        _token: vscode.CancellationToken
    ): vscode.ProviderResult<vscode.InlineValue[]> {
        const inlineValues: vscode.InlineValue[] = [];
        const stoppedLine = context.stoppedLocation.end.line;

        for (let currentLine = viewPort.start.line; currentLine <= stoppedLine; currentLine += 1) {
            const lineText = document.lineAt(currentLine).text;
            // Match variable declaration patterns: `<type> <name> = ...`
            const variableMatches = lineText.matchAll(/(?:int8|int16|int32|int64|uint8|uint16|uint32|uint64|float32|float64|bool|char|string|[A-Z][a-zA-Z0-9_]*)\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*=/g);

            for (const match of variableMatches) {
                if (match.index !== undefined) {
                    const variableName = match[1];
                    const nameIndex = lineText.indexOf(variableName, match.index);
                    if (nameIndex >= 0) {
                        const wordRange = new vscode.Range(
                            currentLine,
                            nameIndex,
                            currentLine,
                            nameIndex + variableName.length
                        );
                        inlineValues.push(
                            new vscode.InlineValueVariableLookup(wordRange, variableName, false)
                        );
                    }
                }
            }
        }

        return inlineValues;
    }
}
