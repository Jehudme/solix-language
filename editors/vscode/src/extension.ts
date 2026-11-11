import * as vscode from 'vscode';
import * as path from 'path';
import * as fs from 'fs';
import * as os from 'os';
import {
    LanguageClient,
    LanguageClientOptions,
    ServerOptions,
    State
} from 'vscode-languageclient/node';
import { registerSolixCommands } from './commands';
import { SolixTaskProvider } from './tasks';

let languageClient: LanguageClient | undefined;
let statusBarItem: vscode.StatusBarItem | undefined;
let outputChannel: vscode.OutputChannel | undefined;

interface DiscoveredBinaries {
    readonly binaryPath: string;
    readonly isToolchain: boolean;
}

/**
 * Searches filesystem and environment for the Solix or Solix LSP binary.
 */
function findSolixBinary(serverLogChannel: vscode.OutputChannel): DiscoveredBinaries {
    // 1. Explicit user configuration
    const configuredPath = vscode.workspace.getConfiguration('solix').get<string>('lsp.path');
    if (configuredPath && configuredPath.trim().length > 0) {
        const trimmedPath = configuredPath.trim();
        const isToolchain = !trimmedPath.endsWith('solix-lsp');
        serverLogChannel.appendLine(`Using user-configured Solix binary: ${trimmedPath} (toolchain: ${isToolchain})`);
        return { binaryPath: trimmedPath, isToolchain };
    }

    // 2. Search workspace folders and walk up parent directories
    const workspaceFolders = vscode.workspace.workspaceFolders;
    if (workspaceFolders) {
        for (const folder of workspaceFolders) {
            let currentDirectory = folder.uri.fsPath;
            for (let depthIndex = 0; depthIndex < 6; depthIndex += 1) {
                const workspaceCandidates = [
                    path.join(currentDirectory, 'target', 'release', 'solix'),
                    path.join(currentDirectory, 'target', 'debug', 'solix'),
                    path.join(currentDirectory, 'target', 'release', 'solix-lsp'),
                    path.join(currentDirectory, 'target', 'debug', 'solix-lsp'),
                ];

                for (const candidate of workspaceCandidates) {
                    if (fs.existsSync(candidate)) {
                        const isToolchain = !candidate.endsWith('solix-lsp');
                        serverLogChannel.appendLine(`Found local build artifact: ${candidate} (toolchain: ${isToolchain})`);
                        return { binaryPath: candidate, isToolchain };
                    }
                }

                const parentDirectory = path.dirname(currentDirectory);
                if (parentDirectory === currentDirectory) {
                    break;
                }
                currentDirectory = parentDirectory;
            }
        }
    }

    // 3. User local bin (~/.local/bin)
    const homeDirectory = os.homedir();
    const localBinSolix = path.join(homeDirectory, '.local', 'bin', 'solix');
    if (fs.existsSync(localBinSolix)) {
        serverLogChannel.appendLine(`Found ~/.local/bin Solix toolchain: ${localBinSolix}`);
        return { binaryPath: localBinSolix, isToolchain: true };
    }

    const localBinLsp = path.join(homeDirectory, '.local', 'bin', 'solix-lsp');
    if (fs.existsSync(localBinLsp)) {
        serverLogChannel.appendLine(`Found ~/.local/bin Solix LSP: ${localBinLsp}`);
        return { binaryPath: localBinLsp, isToolchain: false };
    }

    // 4. User home cargo bin (~/.cargo/bin)
    const cargoBinSolix = path.join(homeDirectory, '.cargo', 'bin', 'solix');
    if (fs.existsSync(cargoBinSolix)) {
        serverLogChannel.appendLine(`Found ~/.cargo/bin Solix toolchain: ${cargoBinSolix}`);
        return { binaryPath: cargoBinSolix, isToolchain: true };
    }

    const cargoBinLsp = path.join(homeDirectory, '.cargo', 'bin', 'solix-lsp');
    if (fs.existsSync(cargoBinLsp)) {
        serverLogChannel.appendLine(`Found ~/.cargo/bin Solix LSP: ${cargoBinLsp}`);
        return { binaryPath: cargoBinLsp, isToolchain: false };
    }

    // 5. Default system PATH fallback
    serverLogChannel.appendLine('Falling back to default system PATH resolution for "solix"');
    return { binaryPath: 'solix', isToolchain: true };
}

/**
 * Extension activation callback.
 */
export async function activate(extensionContext: vscode.ExtensionContext): Promise<void> {
    outputChannel = vscode.window.createOutputChannel('Solix Language Server');
    extensionContext.subscriptions.push(outputChannel);
    outputChannel.appendLine('Activating Solix Language Extension...');

    const { binaryPath, isToolchain } = findSolixBinary(outputChannel);

    // Prepare language server execution arguments
    const serverArguments = isToolchain
        ? vscode.workspace.getConfiguration('solix').get<string[]>('lsp.arguments') || ['lsp', '--stdio']
        : [];

    outputChannel.appendLine(`Launching server executable: ${binaryPath} with arguments: ${JSON.stringify(serverArguments)}`);

    const serverOptions: ServerOptions = {
        command: binaryPath,
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
            fileEvents: vscode.workspace.createFileSystemWatcher('**/{*.slx,manifest.toml}')
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

    // Register interactive commands and task provider immediately before server start
    registerSolixCommands(extensionContext, languageClient, binaryPath);
    extensionContext.subscriptions.push(
        vscode.tasks.registerTaskProvider(
            SolixTaskProvider.taskType,
            new SolixTaskProvider(binaryPath)
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
