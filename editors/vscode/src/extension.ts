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

/**
 * Searches filesystem and environment for the Solix binary.
 */
function findSolixBinary(): { binaryPath: string; isToolchain: boolean } {
    // 1. Explicit user configuration
    const configuredPath = vscode.workspace.getConfiguration('solix').get<string>('lsp.path');
    if (configuredPath && configuredPath.trim().length > 0) {
        const trimmedPath = configuredPath.trim();
        const isToolchain = !trimmedPath.endsWith('solix-lsp');
        return { binaryPath: trimmedPath, isToolchain };
    }

    // 2. Search workspace folders for local compilation target outputs
    const workspaceFolders = vscode.workspace.workspaceFolders;
    if (workspaceFolders) {
        for (const folder of workspaceFolders) {
            const rootPath = folder.uri.fsPath;
            const candidates = [
                path.join(rootPath, 'target', 'release', 'solix'),
                path.join(rootPath, 'target', 'debug', 'solix'),
                path.join(rootPath, '..', '..', 'target', 'release', 'solix'),
                path.join(rootPath, '..', '..', 'target', 'debug', 'solix'),
                path.join(rootPath, 'target', 'release', 'solix-lsp'),
                path.join(rootPath, 'target', 'debug', 'solix-lsp'),
            ];

            for (const candidate of candidates) {
                if (fs.existsSync(candidate)) {
                    const isToolchain = !candidate.endsWith('solix-lsp');
                    return { binaryPath: candidate, isToolchain };
                }
            }
        }
    }

    // 3. User home cargo bin
    const homeDir = os.homedir();
    const cargoBinSolix = path.join(homeDir, '.cargo', 'bin', 'solix');
    if (fs.existsSync(cargoBinSolix)) {
        return { binaryPath: cargoBinSolix, isToolchain: true };
    }

    const cargoBinLsp = path.join(homeDir, '.cargo', 'bin', 'solix-lsp');
    if (fs.existsSync(cargoBinLsp)) {
        return { binaryPath: cargoBinLsp, isToolchain: false };
    }

    // 4. Default system PATH fallback
    return { binaryPath: 'solix', isToolchain: true };
}

/**
 * Extension activation callback.
 */
export async function activate(context: vscode.ExtensionContext): Promise<void> {
    const { binaryPath, isToolchain } = findSolixBinary();

    // Prepare language server execution arguments
    const serverArguments = isToolchain
        ? vscode.workspace.getConfiguration('solix').get<string[]>('lsp.arguments') || ['lsp', '--stdio']
        : [];

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
    context.subscriptions.push(statusBarItem);

    languageClient.onDidChangeState((stateChangeEvent) => {
        if (!statusBarItem) {
            return;
        }

        switch (stateChangeEvent.newState) {
            case State.Running:
                statusBarItem.text = '$(check) Solix LSP';
                statusBarItem.tooltip = 'Solix Language Server is active (Click to restart)';
                break;
            case State.Starting:
                statusBarItem.text = '$(sync~spin) Solix LSP';
                statusBarItem.tooltip = 'Solix Language Server is starting...';
                break;
            case State.Stopped:
                statusBarItem.text = '$(circle-slash) Solix LSP';
                statusBarItem.tooltip = 'Solix Language Server is stopped (Click to restart)';
                break;
        }
    });

    // Register interactive commands and task provider
    registerSolixCommands(context, languageClient, binaryPath);
    context.subscriptions.push(
        vscode.tasks.registerTaskProvider(
            SolixTaskProvider.taskType,
            new SolixTaskProvider(binaryPath)
        )
    );

    await languageClient.start();
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
