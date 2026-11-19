import * as vscode from 'vscode';
import * as path from 'path';
import * as fs from 'fs';
import * as os from 'os';
import * as childProcess from 'child_process';
import {
    LanguageClient,
    LanguageClientOptions,
    ServerOptions,
    State
} from 'vscode-languageclient/node';
import { registerSolixCommands } from './commands';
import { SolixTaskProvider } from './tasks';
import {
    SolixDebugAdapterDescriptorFactory,
    SolixDebugConfigurationProvider,
    SolixInlineValuesProvider
} from './debug';

let languageClient: LanguageClient | undefined;
let statusBarItem: vscode.StatusBarItem | undefined;
let outputChannel: vscode.LogOutputChannel | undefined;

interface DiscoveredBinaries {
    readonly lspBinaryPath: string;
    readonly isToolchain: boolean;
    readonly toolchainBinaryPath: string;
}

/**
 * Checks whether a candidate binary is an LSP-capable toolchain by invoking `lsp --help`.
 */
function isLspCapableToolchain(binaryPath: string): boolean {
    try {
        const executionResult = childProcess.spawnSync(binaryPath, ['lsp', '--help'], {
            encoding: 'utf8',
            timeout: 2000
        });
        return executionResult.status === 0;
    } catch {
        return false;
    }
}

/**
 * Searches filesystem and environment specifically for the Solix toolchain binary (`solix`).
 */
function findToolchainBinary(serverLogChannel: vscode.OutputChannel): string {
    const configuredPath = vscode.workspace.getConfiguration('solix').get<string>('toolchain.path');
    if (configuredPath && configuredPath.trim().length > 0) {
        const trimmedPath = configuredPath.trim();
        serverLogChannel.appendLine(`Using user-configured Solix toolchain binary: ${trimmedPath}`);
        return trimmedPath;
    }

    const workspaceFolders = vscode.workspace.workspaceFolders;
    if (workspaceFolders) {
        const discoveredCandidates: string[] = [];
        for (const folder of workspaceFolders) {
            let currentDirectory = folder.uri.fsPath;
            for (let depthIndex = 0; depthIndex < 6; depthIndex += 1) {
                const candidates = [
                    path.join(currentDirectory, 'target', 'release', 'solix'),
                    path.join(currentDirectory, 'target', 'debug', 'solix'),
                ];
                for (const candidate of candidates) {
                    if (fs.existsSync(candidate) && !discoveredCandidates.includes(candidate)) {
                        discoveredCandidates.push(candidate);
                    }
                }
                const parentDirectory = path.dirname(currentDirectory);
                if (parentDirectory === currentDirectory) {
                    break;
                }
                currentDirectory = parentDirectory;
            }
        }

        discoveredCandidates.sort((firstCandidate, secondCandidate) => {
            const firstIsRelease = firstCandidate.includes(path.join('target', 'release'));
            const secondIsRelease = secondCandidate.includes(path.join('target', 'release'));
            if (firstIsRelease !== secondIsRelease) {
                return firstIsRelease ? -1 : 1;
            }
            try {
                return fs.statSync(secondCandidate).mtimeMs - fs.statSync(firstCandidate).mtimeMs;
            } catch {
                return 0;
            }
        });

        for (const candidate of discoveredCandidates) {
            serverLogChannel.appendLine(`Found local Solix toolchain binary: ${candidate}`);
            return candidate;
        }
    }

    const homeDirectory = os.homedir();
    const localBinSolix = path.join(homeDirectory, '.local', 'bin', 'solix');
    if (fs.existsSync(localBinSolix)) {
        serverLogChannel.appendLine(`Found ~/.local/bin Solix toolchain: ${localBinSolix}`);
        return localBinSolix;
    }

    const cargoBinSolix = path.join(homeDirectory, '.cargo', 'bin', 'solix');
    if (fs.existsSync(cargoBinSolix)) {
        serverLogChannel.appendLine(`Found ~/.cargo/bin Solix toolchain: ${cargoBinSolix}`);
        return cargoBinSolix;
    }

    return 'solix';
}

/**
 * Searches filesystem and environment for the Solix LSP and toolchain binaries.
 */
function findSolixBinaries(serverLogChannel: vscode.OutputChannel): DiscoveredBinaries {
    const toolchainBinaryPath = findToolchainBinary(serverLogChannel);

    // 1. Explicit user configuration for LSP
    const configuredLspPath = vscode.workspace.getConfiguration('solix').get<string>('lsp.path');
    if (configuredLspPath && configuredLspPath.trim().length > 0) {
        const trimmedPath = configuredLspPath.trim();
        const isToolchain = !trimmedPath.endsWith('solix-lsp');
        serverLogChannel.appendLine(`Using user-configured Solix LSP binary: ${trimmedPath} (toolchain: ${isToolchain})`);
        return { lspBinaryPath: trimmedPath, isToolchain, toolchainBinaryPath };
    }

    // 2. Search workspace folders and walk up parent directories
    const workspaceFolders = vscode.workspace.workspaceFolders;
    if (workspaceFolders) {
        const discoveredCandidates: string[] = [];

        for (const folder of workspaceFolders) {
            let currentDirectory = folder.uri.fsPath;
            for (let depthIndex = 0; depthIndex < 6; depthIndex += 1) {
                const workspaceCandidates = [
                    path.join(currentDirectory, 'target', 'release', 'solix'),
                    path.join(currentDirectory, 'target', 'debug', 'solix'),
                ];

                for (const candidate of workspaceCandidates) {
                    if (fs.existsSync(candidate) && !discoveredCandidates.includes(candidate)) {
                        discoveredCandidates.push(candidate);
                    }
                }

                const parentDirectory = path.dirname(currentDirectory);
                if (parentDirectory === currentDirectory) {
                    break;
                }
                currentDirectory = parentDirectory;
            }
        }

        // Sort candidates by modification time (newest first)
        discoveredCandidates.sort((firstPath, secondPath) => {
            try {
                return fs.statSync(secondPath).mtimeMs - fs.statSync(firstPath).mtimeMs;
            } catch {
                return 0;
            }
        });

        for (const candidate of discoveredCandidates) {
            if (isLspCapableToolchain(candidate)) {
                serverLogChannel.appendLine(`Found verified LSP-capable build artifact: ${candidate}`);
                return { lspBinaryPath: candidate, isToolchain: true, toolchainBinaryPath: candidate };
            } else {
                serverLogChannel.appendLine(`Skipping stale build artifact without 'lsp' subcommand: ${candidate}`);
            }
        }
    }

    // 3. User local bin (~/.local/bin)
    const homeDirectory = os.homedir();
    const localBinSolix = path.join(homeDirectory, '.local', 'bin', 'solix');
    if (fs.existsSync(localBinSolix) && isLspCapableToolchain(localBinSolix)) {
        serverLogChannel.appendLine(`Found verified ~/.local/bin Solix toolchain: ${localBinSolix}`);
        return { lspBinaryPath: localBinSolix, isToolchain: true, toolchainBinaryPath: localBinSolix };
    }

    // 4. User home cargo bin (~/.cargo/bin)
    const cargoBinSolix = path.join(homeDirectory, '.cargo', 'bin', 'solix');
    if (fs.existsSync(cargoBinSolix) && isLspCapableToolchain(cargoBinSolix)) {
        serverLogChannel.appendLine(`Found verified ~/.cargo/bin Solix toolchain: ${cargoBinSolix}`);
        return { lspBinaryPath: cargoBinSolix, isToolchain: true, toolchainBinaryPath: cargoBinSolix };
    }

    // 5. Default system PATH fallback
    serverLogChannel.appendLine('Falling back to default system PATH resolution for "solix"');
    return { lspBinaryPath: 'solix', isToolchain: true, toolchainBinaryPath: 'solix' };
}

/**
 * Extension activation callback.
 */
export async function activate(extensionContext: vscode.ExtensionContext): Promise<void> {
    outputChannel = vscode.window.createOutputChannel('Solix Language Server', { log: true });
    extensionContext.subscriptions.push(outputChannel);
    outputChannel.appendLine('Activating Solix Language Extension...');

    const { lspBinaryPath, isToolchain, toolchainBinaryPath } = findSolixBinaries(outputChannel);

    // Prepare language server execution arguments
    const serverArguments = isToolchain
        ? vscode.workspace.getConfiguration('solix').get<string[]>('lsp.arguments') || ['lsp', '--stdio']
        : [];

    outputChannel.appendLine(`Launching server executable: ${lspBinaryPath} with arguments: ${JSON.stringify(serverArguments)}`);

    const serverOptions: ServerOptions = {
        command: lspBinaryPath,
        args: serverArguments,
        options: {
            env: {
                ...process.env,
                RUST_BACKTRACE: '1'
            }
        }
    };

    const clientOptions: LanguageClientOptions = {
        documentSelector: [{ scheme: 'file', language: 'solix' }],
        outputChannel: outputChannel,
        synchronize: {
            fileEvents: vscode.workspace.createFileSystemWatcher('**/{*.slx,manifest.yaml,manifest.yml,manifest.toml}')
        }
    };

    languageClient = new LanguageClient(
        'solix-lsp',
        'Solix Language Server',
        serverOptions,
        clientOptions
    );

    // Initialize Status Bar item
    statusBarItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right, 100);
    statusBarItem.command = 'solix.restartServer';
    statusBarItem.text = '$(sync~spin) Solix LSP';
    statusBarItem.tooltip = 'Solix Language Server is initializing...';
    statusBarItem.show();
    extensionContext.subscriptions.push(statusBarItem);

    languageClient.onDidChangeState((stateChangeEvent) => {
        if (!statusBarItem) {
            return;
        }

        switch (stateChangeEvent.newState) {
            case State.Running:
                statusBarItem.text = '$(check) Solix LSP';
                statusBarItem.tooltip = 'Solix Language Server is active (Click to restart)';
                outputChannel?.appendLine('Solix Language Server transitioned to RUNNING state.');
                break;
            case State.Starting:
                statusBarItem.text = '$(sync~spin) Solix LSP';
                statusBarItem.tooltip = 'Solix Language Server is starting...';
                outputChannel?.appendLine('Solix Language Server is STARTING...');
                break;
            case State.Stopped:
                statusBarItem.text = '$(circle-slash) Solix LSP';
                statusBarItem.tooltip = 'Solix Language Server is stopped (Click to restart)';
                outputChannel?.appendLine('Solix Language Server STOPPED.');
                break;
        }
    });

    // Register interactive commands and task provider with the toolchain binary
    registerSolixCommands(extensionContext, languageClient, toolchainBinaryPath);
    extensionContext.subscriptions.push(
        vscode.tasks.registerTaskProvider(
            SolixTaskProvider.taskType,
            new SolixTaskProvider(toolchainBinaryPath)
        )
    );

    // Register debugger contributions
    extensionContext.subscriptions.push(
        vscode.debug.registerDebugAdapterDescriptorFactory(
            'solix',
            new SolixDebugAdapterDescriptorFactory(toolchainBinaryPath)
        )
    );
    extensionContext.subscriptions.push(
        vscode.debug.registerDebugConfigurationProvider(
            'solix',
            new SolixDebugConfigurationProvider(toolchainBinaryPath)
        )
    );
    extensionContext.subscriptions.push(
        vscode.languages.registerInlineValuesProvider(
            'solix',
            new SolixInlineValuesProvider()
        )
    );

    try {
        await languageClient.start();
        outputChannel.appendLine('Solix Language Server initialized successfully.');
    } catch (startupError) {
        const errorMessage = `Failed to start Solix Language Server: ${String(startupError)}`;
        outputChannel.appendLine(errorMessage);
        vscode.window.showErrorMessage(errorMessage);
    }
}

/**
 * Extension deactivation callback.
 */
export async function deactivate(): Promise<void> {
    if (statusBarItem) {
        statusBarItem.dispose();
    }
    if (languageClient) {
        await languageClient.stop();
    }
}
