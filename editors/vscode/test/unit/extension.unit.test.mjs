import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import Module from 'node:module';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const currentDir = path.dirname(fileURLToPath(import.meta.url));
const rootDir = path.resolve(currentDir, '../..');
const unitBuildDir = path.join(rootDir, 'build', 'test-out');

function createVscodeMock(workspaceRoot) {
    const registeredCommands = new Map();
    const registeredTaskProviders = [];
    const registeredDebugFactories = [];
    const registeredDebugConfigProviders = [];
    const registeredInlineValueProviders = [];
    const registeredTextContentProviders = new Map();
    const createdStatusBarItems = [];
    const clientInstances = [];

    class MockDisposable {
        constructor(fn = () => {}) {
            this.disposeFn = fn;
        }
        dispose() {
            this.disposeFn();
        }
    }

    class MockEventEmitter {
        constructor() {
            this.listeners = [];
            this.event = (listener) => {
                this.listeners.push(listener);
                return new MockDisposable();
            };
        }
        fire(data) {
            for (const listener of this.listeners) {
                listener(data);
            }
        }
        dispose() {}
    }

    class MockUri {
        constructor(scheme, fsPath) {
            this.scheme = scheme;
            this.fsPath = fsPath;
        }
        static parse(raw) {
            const colonIdx = raw.indexOf(':');
            if (colonIdx > 0) {
                const scheme = raw.slice(0, colonIdx);
                const rest = raw.slice(colonIdx + 1).replace(/^\/\//, '');
                return new MockUri(scheme, rest);
            }
            return new MockUri('file', raw);
        }
        static file(fsPath) {
            return new MockUri('file', fsPath);
        }
        toString() {
            return `${this.scheme}:${this.fsPath}`;
        }
    }

    class MockPosition {
        constructor(line, character) {
            this.line = line;
            this.character = character;
        }
    }

    class MockRange {
        constructor(startLine, startCharacter, endLine, endCharacter) {
            this.start = { line: startLine, character: startCharacter };
            this.end = { line: endLine, character: endCharacter };
        }
    }

    class MockTask {
        constructor(definition, scope, name, source, execution, problemMatchers) {
            this.definition = definition;
            this.scope = scope;
            this.name = name;
            this.source = source;
            this.execution = execution;
            this.problemMatchers = problemMatchers;
            this.group = undefined;
        }
    }

    class MockProcessExecution {
        constructor(processPath, args, options) {
            this.process = processPath;
            this.args = args;
            this.options = options;
        }
    }

    class MockDebugAdapterExecutable {
        constructor(command, args) {
            this.command = command;
            this.args = args;
        }
    }

    class MockDebugAdapterServer {
        constructor(port, host) {
            this.port = port;
            this.host = host;
        }
    }

    const vscodeMock = {
        StatusBarAlignment: { Left: 1, Right: 2 },
        TaskScope: { Workspace: 'Workspace' },
        TaskGroup: { Build: 'Build', Test: 'Test' },
        ViewColumn: { Beside: 2 },
        Uri: MockUri,
        Position: MockPosition,
        Range: MockRange,
        EventEmitter: MockEventEmitter,
        Disposable: MockDisposable,
        Task: MockTask,
        ProcessExecution: MockProcessExecution,
        DebugAdapterExecutable: MockDebugAdapterExecutable,
        DebugAdapterServer: MockDebugAdapterServer,
        InlineValueVariableLookup: class {
            constructor(range, variableName, caseSensitiveLookup) {
                this.range = range;
                this.variableName = variableName;
                this.caseSensitiveLookup = caseSensitiveLookup;
            }
        },
        workspace: {
            workspaceFolders: workspaceRoot
                ? [{ uri: MockUri.file(workspaceRoot), name: 'test-ws', index: 0 }]
                : [],
            getConfiguration: () => ({
                get: (key) => {
                    if (key === 'lsp.arguments') {
                        return ['lsp', '--stdio'];
                    }
                    return '';
                },
            }),
            createFileSystemWatcher: () => new MockDisposable(),
            registerTextDocumentContentProvider: (scheme, provider) => {
                registeredTextContentProviders.set(scheme, provider);
                return new MockDisposable();
            },
            openTextDocument: async (uri) => ({ uri, getText: () => '' }),
        },
        window: {
            activeTextEditor: undefined,
            createOutputChannel: () => ({
                appendLine: () => {},
                dispose: () => {},
            }),
            createStatusBarItem: (alignment, priority) => {
                const item = { alignment, priority, text: '', tooltip: '', command: undefined, show: () => {}, dispose: () => {} };
                createdStatusBarItems.push(item);
                return item;
            },
            showInformationMessage: async () => undefined,
            showErrorMessage: async () => undefined,
            showQuickPick: async () => undefined,
            showTextDocument: async () => undefined,
        },
        commands: {
            registerCommand: (name, handler) => {
                registeredCommands.set(name, handler);
                return new MockDisposable(() => registeredCommands.delete(name));
            },
            executeCommand: async () => [],
        },
        tasks: {
            registerTaskProvider: (taskType, provider) => {
                registeredTaskProviders.push({ taskType, provider });
                return new MockDisposable();
            },
        },
        debug: {
            registerDebugAdapterDescriptorFactory: (debugType, factory) => {
                registeredDebugFactories.push({ debugType, factory });
                return new MockDisposable();
            },
            registerDebugConfigurationProvider: (debugType, provider) => {
                registeredDebugConfigProviders.push({ debugType, provider });
                return new MockDisposable();
            },
        },
        languages: {
            registerInlineValuesProvider: (lang, provider) => {
                registeredInlineValueProviders.push({ lang, provider });
                return new MockDisposable();
            },
        },
    };

    class MockLanguageClient {
        constructor(id, name, serverOptions, clientOptions) {
            this.id = id;
            this.name = name;
            this.serverOptions = serverOptions;
            this.clientOptions = clientOptions;
            this.running = false;
            clientInstances.push(this);
        }
        onDidChangeState(listener) {
            this.stateListener = listener;
            return new MockDisposable();
        }
        isRunning() {
            return this.running;
        }
        async start() {
            this.running = true;
        }
        async restart() {
            this.running = true;
        }
        async stop() {
            this.running = false;
        }
    }

    const languageClientMock = {
        LanguageClient: MockLanguageClient,
        State: { Stopped: 1, Running: 2, Starting: 3 },
    };

    return {
        vscodeMock,
        languageClientMock,
        state: {
            registeredCommands,
            registeredTaskProviders,
            registeredDebugFactories,
            registeredDebugConfigProviders,
            registeredInlineValueProviders,
            registeredTextContentProviders,
            createdStatusBarItems,
            clientInstances,
        },
    };
}

test('Layer 1: activate registers all commands, providers, and tasks with mocked vscode API', async () => {
    const tempWs = fs.mkdtempSync(path.join(os.tmpdir(), 'solix-vscode-unit-'));
    fs.writeFileSync(
        path.join(tempWs, 'manifest.toml'),
        '[configuration]\nname = "unit_demo"\nversion = "0.1.0"\n[profiles.release]\nsources = ["src"]\n'
    );
    fs.mkdirSync(path.join(tempWs, 'src'));
    fs.writeFileSync(path.join(tempWs, 'src', 'main.slx'), 'int32 main() { return 0; }\n');

    const { vscodeMock, languageClientMock, state } = createVscodeMock(tempWs);
    const origLoad = Module._load;
    Module._load = function patchedLoad(request, parent, isMain) {
        if (request === 'vscode') {
            return vscodeMock;
        }
        if (request === 'vscode-languageclient/node') {
            return languageClientMock;
        }
        return origLoad.call(this, request, parent, isMain);
    };

    const prevHeadless = process.env.SOLIX_HEADLESS_DAP;
    process.env.SOLIX_HEADLESS_DAP = '1';

    try {
        const extModule = await import(`file://${path.join(unitBuildDir, 'extension.js')}?t=${Date.now()}`);
        const tasksModule = await import(`file://${path.join(unitBuildDir, 'tasks.js')}?t=${Date.now()}`);
        const debugModule = await import(`file://${path.join(unitBuildDir, 'debug.js')}?t=${Date.now()}`);

        const context = { subscriptions: [] };
        await extModule.activate(context);

        const expectedCommands = [
            'solix.runMain',
            'solix.debugMain',
            'solix.runTest',
            'solix.debugTest',
            'solix.buildProject',
            'solix.syncPackages',
            'solix.showDisassembly',
            'solix.showSynthesizedApi',
            'solix.selectProfile',
            'solix.restartServer',
        ];
        for (const cmdName of expectedCommands) {
            assert.ok(
                state.registeredCommands.has(cmdName),
                `Command '${cmdName}' must be registered during activate()`
            );
        }

        assert.strictEqual(state.registeredTaskProviders.length, 1);
        assert.strictEqual(state.registeredDebugFactories.length, 1);
        assert.strictEqual(state.registeredDebugConfigProviders.length, 1);
        assert.strictEqual(state.registeredInlineValueProviders.length, 1);
        assert.ok(state.registeredTextContentProviders.has('solix-disasm'));
        assert.ok(state.registeredTextContentProviders.has('solix-api'));
        assert.strictEqual(state.clientInstances.length, 1);
        assert.strictEqual(state.clientInstances[0].isRunning(), true);

        // Verify SolixTaskProvider tasks
        const taskProvider = new tasksModule.SolixTaskProvider('/tmp/solix');
        const providedTasks = taskProvider.provideTasks();
        assert.ok(providedTasks.length >= 4, 'Must provide build debug/release/test and run tasks');
        assert.ok(providedTasks.some((t) => t.name === 'solix build --release'));
        assert.ok(providedTasks.some((t) => t.name === 'solix build --debug'));
        assert.ok(providedTasks.some((t) => t.name === 'solix build --test'));

        // Verify DebugConfigurationProvider resolves standalone file without spawning external toolchain
        const standaloneFile = path.join(os.tmpdir(), 'standalone_sample.slx');
        fs.writeFileSync(standaloneFile, 'int32 main() { return 0; }\n');
        try {
            const debugConfigProvider = new debugModule.SolixDebugConfigurationProvider('/tmp/solix');
            const resolvedConfig = debugConfigProvider.resolveDebugConfiguration(
                undefined,
                { type: 'solix', name: 'Debug Standalone', request: 'launch', program: standaloneFile }
            );
            assert.ok(resolvedConfig, 'Must resolve standalone file debug configuration');
            assert.strictEqual(resolvedConfig.type, 'solix');
            assert.strictEqual(resolvedConfig.request, 'launch');
            assert.strictEqual(resolvedConfig.program, standaloneFile);
        } finally {
            fs.rmSync(standaloneFile, { force: true });
        }

        // Verify InlineValuesProvider extracts declared variables above stopped line
        const inlineProvider = new debugModule.SolixInlineValuesProvider();
        const inlineValues = inlineProvider.provideInlineValues(
            { lineAt: (lineIdx) => ({ text: lineIdx === 0 ? '    int32 score = 42;' : '    return score;' }) },
            new vscodeMock.Range(0, 0, 1, 20),
            { stoppedLocation: new vscodeMock.Range(1, 0, 1, 0) },
            {}
        );
        assert.strictEqual(inlineValues.length, 1);
        assert.strictEqual(inlineValues[0].variableName, 'score');

        // Verify DebugAdapterDescriptorFactory uses stdio executable under SOLIX_HEADLESS_DAP=1
        const dapFactory = new debugModule.SolixDebugAdapterDescriptorFactory('/tmp/solix');
        const descriptor = await dapFactory.createDebugAdapterDescriptor({
            configuration: { console: 'internalConsole' },
        });
        assert.strictEqual(descriptor.command, '/tmp/solix');
        assert.deepStrictEqual(descriptor.args, ['dap', '--stdio']);

        await extModule.deactivate();
        assert.strictEqual(state.clientInstances[0].isRunning(), false);
    } finally {
        Module._load = origLoad;
        if (prevHeadless === undefined) {
            delete process.env.SOLIX_HEADLESS_DAP;
        } else {
            process.env.SOLIX_HEADLESS_DAP = prevHeadless;
        }
        fs.rmSync(tempWs, { recursive: true, force: true });
    }
});
