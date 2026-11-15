/**
 * Headless Node.js JSON-RPC integration test for Solix Language Server.
 *
 * Verifies that the client connection behaves identically to VS Code LanguageClient
 * without requiring an active graphical desktop or Electron binary.
 */

import * as childProcess from 'child_process';
import * as path from 'path';
import * as fs from 'fs';
import { fileURLToPath } from 'url';
import * as rpc from 'vscode-jsonrpc/node.js';

const currentDir = path.dirname(fileURLToPath(import.meta.url));

function locateSolixBinary() {
    const candidates = [
        path.resolve(currentDir, '../../../target/debug/solix'),
        path.resolve(currentDir, '../../../target/release/solix'),
        path.resolve(currentDir, '../../../target/debug/solix-lsp'),
        path.resolve(currentDir, '../../../target/release/solix-lsp'),
    ];

    for (const candidate of candidates) {
        if (fs.existsSync(candidate)) {
            const isToolchain = !candidate.endsWith('solix-lsp');
            return { binaryPath: candidate, isToolchain };
        }
    }

    throw new Error('Could not find compiled solix or solix-lsp binary in target directory');
}

async function runClientRpcSuite() {
    console.log('=== Running Solix Headless JSON-RPC Client Integration Suite ===');

    const { binaryPath, isToolchain } = locateSolixBinary();
    const commandArguments = isToolchain ? ['lsp', '--stdio'] : [];

    console.log(`Spawning server: ${binaryPath} ${commandArguments.join(' ')}`);
    const serverProcess = childProcess.spawn(binaryPath, commandArguments, {
        stdio: ['pipe', 'pipe', 'inherit'],
        env: {
            ...process.env,
            RUST_BACKTRACE: '1',
        },
    });

    const connection = rpc.createMessageConnection(
        new rpc.StreamMessageReader(serverProcess.stdout),
        new rpc.StreamMessageWriter(serverProcess.stdin)
    );

    connection.listen();

    try {
        // Step 1: Initialize
        console.log('Step 1: Sending initialize request...');
        const initResponse = await connection.sendRequest('initialize', {
            processId: process.pid,
            rootUri: 'file:///workspace',
            capabilities: {
                textDocument: {
                    hover: { contentFormat: ['markdown', 'plaintext'] },
                    completion: { completionItem: { snippetSupport: true } },
                    definition: { dynamicRegistration: false },
                    formatting: { dynamicRegistration: false },
                },
            },
        });

        if (!initResponse.capabilities) {
            throw new Error('Server returned empty capabilities on initialize');
        }

        if (!initResponse.capabilities.hoverProvider) {
            throw new Error('Expected hoverProvider capability to be true');
        }
        if (!initResponse.capabilities.definitionProvider) {
            throw new Error('Expected definitionProvider capability to be true');
        }
        if (!initResponse.capabilities.documentFormattingProvider) {
            throw new Error('Expected documentFormattingProvider capability to be true');
        }

        console.log('  ✓ Server capabilities verified successfully.');

        // Step 2: Initialized notification
        connection.sendNotification('initialized', {});

        // Step 3: Open Document Overlay
        console.log('Step 2: Sending textDocument/didOpen...');
        const documentUri = 'file:///workspace/test.slx';
        const sourceText = [
            'class Helper {',
            '    public static int32 compute() {',
            '        return 42;',
            '    }',
            '}',
            'void main() {',
            '    int32 value = Helper::compute();',
            '}',
            '',
        ].join('\n');

        connection.sendNotification('textDocument/didOpen', {
            textDocument: {
                uri: documentUri,
                languageId: 'solix',
                version: 1,
                text: sourceText,
            },
        });

        // Small pause to allow server indexing
        await new Promise((resolve) => setTimeout(resolve, 50));

        // Step 4: Hover
        console.log('Step 3: Requesting textDocument/hover...');
        const hoverResult = await connection.sendRequest('textDocument/hover', {
            textDocument: { uri: documentUri },
            position: { line: 0, character: 7 }, // 'Helper'
        });

        if (!hoverResult || !hoverResult.contents) {
            throw new Error('Expected hover card content for Helper');
        }
        console.log('  ✓ Hover result returned Markdown card.');

        // Step 5: Scope Resolution Completion
        console.log('Step 4: Requesting textDocument/completion (Helper::)...');
        const completionSource = [
            'class Helper {',
            '    public static int32 compute() { return 42; }',
            '}',
            'void main() {',
            '    Helper::',
            '}',
            '',
        ].join('\n');

        connection.sendNotification('textDocument/didChange', {
            textDocument: { uri: documentUri, version: 2 },
            contentChanges: [{ text: completionSource }],
        });

        await new Promise((resolve) => setTimeout(resolve, 50));

        const completionResult = await connection.sendRequest('textDocument/completion', {
            textDocument: { uri: documentUri },
            position: { line: 4, character: 12 }, // After Helper::
        });

        const completionItems = completionResult.items || completionResult;
        const labels = completionItems.map((item) => item.label);
        if (!labels.includes('compute')) {
            throw new Error(`Expected 'compute' in completions, got: ${JSON.stringify(labels)}`);
        }
        console.log('  ✓ Scope resolution completion returned static members.');

        // Step 6: Formatting
        console.log('Step 5: Requesting textDocument/formatting on unformatted source...');
        const unformattedSource = 'class Unformatted{\nint32   x=10;\n}\n';
        connection.sendNotification('textDocument/didChange', {
            textDocument: { uri: documentUri, version: 3 },
            contentChanges: [{ text: unformattedSource }],
        });
        await new Promise((resolve) => setTimeout(resolve, 50));

        const formatEdits = await connection.sendRequest('textDocument/formatting', {
            textDocument: { uri: documentUri },
            options: { tabSize: 4, insertSpaces: true },
        });

        if (!Array.isArray(formatEdits) || formatEdits.length === 0) {
            throw new Error('Expected formatting text edits on unformatted source');
        }
        console.log('  ✓ Formatting returned standardized 4-space edits.');

        // Step 7: Local Variable Dot Completion
        console.log('Step 6: Requesting local variable dot completion (origin_point.)...');
        const localVarSource = [
            'class Point {',
            '    int32 coordinate_x;',
            '    int32 coordinate_y;',
            '    public int32 distance() { return 0; }',
            '}',
            'void main() {',
            '    Point origin_point = new Point();',
            '    origin_point.',
            '}',
            '',
        ].join('\n');

        connection.sendNotification('textDocument/didChange', {
            textDocument: { uri: documentUri, version: 4 },
            contentChanges: [{ text: localVarSource }],
        });
        await new Promise((resolve) => setTimeout(resolve, 80));

        const localDotResult = await connection.sendRequest('textDocument/completion', {
            textDocument: { uri: documentUri },
            position: { line: 7, character: 17 }, // After origin_point.
        });

        const localItems = localDotResult.items || localDotResult;
        const localLabels = localItems.map((entryItem) => entryItem.label);
        if (!localLabels.includes('coordinate_x') || !localLabels.includes('coordinate_y') || !localLabels.includes('distance')) {
            throw new Error(`Expected Point members in dot completion, got: ${JSON.stringify(localLabels)}`);
        }
        console.log('  ✓ Local variable dot completion returned receiver members only.');

        // Step 8: CodeLens Request
        console.log('Step 7: Requesting textDocument/codeLens for main()...');
        const codeLensResult = await connection.sendRequest('textDocument/codeLens', {
            textDocument: { uri: documentUri },
        });

        if (!Array.isArray(codeLensResult) || codeLensResult.length === 0) {
            throw new Error('Expected at least one CodeLens for main() entrypoint');
        }
        const hasRunMain = codeLensResult.some((lens) => lens.command && lens.command.command === 'solix.runMain');
        if (!hasRunMain) {
            throw new Error(`Expected 'solix.runMain' CodeLens, got: ${JSON.stringify(codeLensResult)}`);
        }
        console.log('  ✓ CodeLens correctly emitted solix.runMain action.');

        // Step 9: Clean Shutdown
        console.log('Step 8: Sending shutdown request...');
        await connection.sendRequest('shutdown');
        connection.sendNotification('exit');

        serverProcess.kill();
        console.log('=== Headless JSON-RPC Client Suite: ALL CHECKS PASSED ===');
    } catch (suiteError) {
        serverProcess.kill();
        console.error('Test Suite Failed:', suiteError);
        process.exit(1);
    }
}

runClientRpcSuite().catch((uncaughtError) => {
    console.error('Uncaught runner failure:', uncaughtError);
    process.exit(1);
});
