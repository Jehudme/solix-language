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

export interface ToolchainVersionInfo {
    readonly majorVersion: number;
    readonly minorVersion: number;
    readonly patchVersion: number;
    readonly rawVersionString: string;
}

export interface ToolchainCompatibilityStatus {
    readonly isCompatible: boolean;
    readonly versionInfo?: ToolchainVersionInfo;
    readonly warningMessage?: string;
    readonly isMajorMismatch: boolean;
}

/**
 * Parses raw version command output into structured SemVer components.
 */
export function parseToolchainVersionOutput(versionOutput: string): ToolchainVersionInfo | undefined {
    const semverPattern = /(?:solix(?:-toolchain)?\s+)?v?(\d+)\.(\d+)\.(\d+)/i;
    const match = semverPattern.exec(versionOutput.trim());
    if (!match) {
        return undefined;
    }
    return {
        majorVersion: parseInt(match[1], 10),
        minorVersion: parseInt(match[2], 10),
        patchVersion: parseInt(match[3], 10),
        rawVersionString: `${match[1]}.${match[2]}.${match[3]}`
    };
}

/**
 * Validates whether the discovered Solix toolchain version satisfies extension constraints.
 */
export function checkToolchainCompatibility(
    versionOutput: string,
    expectedMajorVersion: number,
    minimumVersionString: string
): ToolchainCompatibilityStatus {
    const versionInfo = parseToolchainVersionOutput(versionOutput);
    if (!versionInfo) {
        return {
            isCompatible: false,
            warningMessage: `Failed to parse Solix toolchain version from output: "${versionOutput.trim()}"`,
            isMajorMismatch: false,
        };
    }

    if (versionInfo.majorVersion !== expectedMajorVersion) {
        return {
            isCompatible: false,
            versionInfo,
            warningMessage: `Solix extension requires toolchain major version ${expectedMajorVersion}.x, but found v${versionInfo.rawVersionString}. Please update your Solix toolchain.`,
            isMajorMismatch: true,
        };
    }

    const minVersionParsed = parseToolchainVersionOutput(minimumVersionString);
    if (minVersionParsed) {
        const isBelowMinimum =
            versionInfo.majorVersion < minVersionParsed.majorVersion ||
            (versionInfo.majorVersion === minVersionParsed.majorVersion &&
                (versionInfo.minorVersion < minVersionParsed.minorVersion ||
                    (versionInfo.minorVersion === minVersionParsed.minorVersion &&
                        versionInfo.patchVersion < minVersionParsed.patchVersion)));

        if (isBelowMinimum) {
            return {
                isCompatible: true,
                versionInfo,
                warningMessage: `Installed Solix toolchain (v${versionInfo.rawVersionString}) is older than recommended minimum v${minimumVersionString}. Some language features may be unavailable.`,
                isMajorMismatch: false,
            };
        }
    }

    return {
        isCompatible: true,
        versionInfo,
        isMajorMismatch: false,
    };
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

    // 2. Explicit user configuration for Toolchain path
    const configuredToolchainPath = vscode.workspace.getConfiguration('solix').get<string>('toolchain.path');
    if (configuredToolchainPath && configuredToolchainPath.trim().length > 0) {
        const trimmedToolchain = configuredToolchainPath.trim();
        if (isLspCapableToolchain(trimmedToolchain)) {
            serverLogChannel.appendLine(`Using user-configured Solix toolchain for LSP: ${trimmedToolchain}`);
            return { lspBinaryPath: trimmedToolchain, isToolchain: true, toolchainBinaryPath: trimmedToolchain };
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

    // Verify toolchain semantic version compatibility
    const expectedMajorVersion =
        vscode.workspace.getConfiguration('solix').get<number>('toolchain.compatibleMajor') ?? 2;
    const minimumVersionString =
        vscode.workspace.getConfiguration('solix').get<string>('toolchain.minVersion') ?? '1.9.0';

    try {
        const versionProcessResult = childProcess.spawnSync(toolchainBinaryPath, ['--version'], {
            encoding: 'utf8',
            timeout: 3000,
        });

        if (versionProcessResult.status === 0 && versionProcessResult.stdout) {
            const compatibilityStatus = checkToolchainCompatibility(
                versionProcessResult.stdout,
                expectedMajorVersion,
                minimumVersionString
            );

            if (compatibilityStatus.warningMessage) {
                outputChannel.appendLine(`[Toolchain Compatibility] ${compatibilityStatus.warningMessage}`);
                if (compatibilityStatus.isMajorMismatch) {
                    void vscode.window
                        .showWarningMessage(
                            compatibilityStatus.warningMessage,
                            'View Installation Guide',
                            'Dismiss'
                        )
                        .then((userSelection) => {
                            if (userSelection === 'View Installation Guide') {
                                void vscode.env.openExternal(
                                    vscode.Uri.parse(
                                        'https://github.com/Jehudme/solix-language#installation'
                                    )
                                );
                            }
                        });
                }
            } else if (compatibilityStatus.versionInfo) {
                outputChannel.appendLine(
                    `Verified compatible Solix toolchain v${compatibilityStatus.versionInfo.rawVersionString} (target major: ${expectedMajorVersion}.x)`
                );
            }
        }
    } catch (versionCheckError) {
        outputChannel.appendLine(
            `[Toolchain Compatibility] Could not determine toolchain version: ${versionCheckError}`
        );
    }

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

export { findToolchainBinary, findSolixBinaries };
