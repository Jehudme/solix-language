/**
 * Headless Node.js DAP integration test for Solix Debug Adapter.
 *
 * Verifies that `solix dap --stdio` handles full DAP sessions:
 * initialize -> launch -> setBreakpoints -> configurationDone -> hit breakpoint ->
 * stackTrace -> scopes -> variables -> expand fields -> step -> evaluate -> continue -> terminated/exited.
 * Tests both direct .slx source execution and compiled .slxarc archive container execution.
 */

import * as childProcess from 'child_process';
import * as path from 'path';
import * as fs from 'fs';
import * as assert from 'assert';
import { fileURLToPath } from 'url';

const currentDir = path.dirname(fileURLToPath(import.meta.url));

function locateSolixBinary() {
    if (process.env.SOLIX_BIN && fs.existsSync(process.env.SOLIX_BIN)) {
        return process.env.SOLIX_BIN;
    }
    const exeSuffix = process.platform === 'win32' ? '.exe' : '';
    const candidates = [
        path.resolve(currentDir, `../../../target/debug/solix${exeSuffix}`),
        path.resolve(currentDir, `../../../target/release/solix${exeSuffix}`),
    ];

    for (const candidate of candidates) {
        if (fs.existsSync(candidate)) {
            return candidate;
        }
    }

    throw new Error('Could not find compiled solix binary in target directory');
}

class DapClient {
    constructor(processRef) {
        this.process = processRef;
        this.sequenceNumber = 1;
        this.pendingRequests = new Map();
        this.eventListeners = [];
        this.collectedOutputMessages = [];
        this.buffer = '';

        this.process.stdout.on('data', (chunk) => {
            this.buffer += chunk.toString('utf8');
            this.processBuffer();
        });
    }

    processBuffer() {
        while (true) {
            const headerEndIndex = this.buffer.indexOf('\r\n\r\n');
            if (headerEndIndex === -1) {
                break;
            }

            const headerText = this.buffer.slice(0, headerEndIndex);
            const contentLengthMatch = headerText.match(/Content-Length:\s*(\d+)/i);
            if (!contentLengthMatch) {
                break;
            }

            const contentLength = parseInt(contentLengthMatch[1], 10);
            const bodyStartIndex = headerEndIndex + 4;
            if (this.buffer.length < bodyStartIndex + contentLength) {
                break;
            }

            const messageBody = this.buffer.slice(bodyStartIndex, bodyStartIndex + contentLength);
            this.buffer = this.buffer.slice(bodyStartIndex + contentLength);

            try {
                const parsedMessage = JSON.parse(messageBody);
                this.handleMessage(parsedMessage);
            } catch (jsonError) {
                console.error('Failed to parse DAP JSON-RPC body:', jsonError, messageBody);
            }
        }
    }

    handleMessage(message) {
        if (message.type === 'response') {
            const pendingCallback = this.pendingRequests.get(message.request_seq);
            if (pendingCallback) {
                this.pendingRequests.delete(message.request_seq);
                pendingCallback(message);
            }
        } else if (message.type === 'event') {
            if (message.event === 'output' && message.body && message.body.output) {
                this.collectedOutputMessages.push(message.body.output);
            }
            for (const listener of this.eventListeners) {
                listener(message);
            }
        }
    }

    sendRequest(commandName, requestArguments) {
        const currentSeq = this.sequenceNumber;
        this.sequenceNumber += 1;

        const requestObject = {
            seq: currentSeq,
            type: 'request',
            command: commandName,
        };
        if (requestArguments !== undefined) {
            requestObject.arguments = requestArguments;
        }

        const payloadString = JSON.stringify(requestObject);
        const headerString = `Content-Length: ${Buffer.byteLength(payloadString, 'utf8')}\r\n\r\n`;

        return new Promise((resolve, reject) => {
            const timeoutTimer = setTimeout(() => {
                this.pendingRequests.delete(currentSeq);
                reject(new Error(`DAP Request '${commandName}' timed out after 10000ms`));
            }, 10000);

            this.pendingRequests.set(currentSeq, (responseMessage) => {
                clearTimeout(timeoutTimer);
                if (responseMessage.success) {
                    resolve(responseMessage.body);
                } else {
                    reject(new Error(responseMessage.message || `DAP request '${commandName}' failed`));
                }
            });

            this.process.stdin.write(headerString + payloadString);
        });
    }

    waitForEvent(eventName, timeoutMs = 10000) {
        return new Promise((resolve, reject) => {
            const timeoutTimer = setTimeout(() => {
                reject(new Error(`Timed out waiting for event '${eventName}' after ${timeoutMs}ms`));
            }, timeoutMs);

            const listener = (eventMessage) => {
                if (eventMessage.event === eventName) {
                    clearTimeout(timeoutTimer);
                    const listenerIndex = this.eventListeners.indexOf(listener);
                    if (listenerIndex >= 0) {
                        this.eventListeners.splice(listenerIndex, 1);
                    }
                    resolve(eventMessage.body);
                }
            };

            this.eventListeners.push(listener);
        });
    }
}

async function executeDapSession(targetProgramPath, mainSourcePath, workspaceRoot, testLabel) {
    console.log(`\n--- Running Session: ${testLabel} ---`);
    console.log(`Target Program: ${targetProgramPath}`);

    const solixBinary = locateSolixBinary();

    const dapProcess = childProcess.spawn(solixBinary, ['dap', '--stdio'], {
        stdio: ['pipe', 'pipe', 'inherit'],
        env: {
            ...process.env,
            RUST_BACKTRACE: '1'
        }
    });

    const dapClient = new DapClient(dapProcess);

    try {
        // 1. Initialize
        const initPromise = dapClient.sendRequest('initialize', {
            adapterID: 'solix',
            clientID: 'vscode-headless-test',
            linesStartAt1: true,
            columnsStartAt1: true,
            supportsVariableType: true
        });
        const initializedEventPromise = dapClient.waitForEvent('initialized');
        const capabilities = await initPromise;
        assert.ok(capabilities.supportsConfigurationDoneRequest);
        await initializedEventPromise;

        // 2. Launch
        await dapClient.sendRequest('launch', {
            program: targetProgramPath,
            cwd: workspaceRoot,
            stopOnEntry: false
        });

        // 3. SetBreakpoints on line 47 of main.slx
        const breakpointsResponse = await dapClient.sendRequest('setBreakpoints', {
            source: {
                path: mainSourcePath,
                name: 'main.slx'
            },
            breakpoints: [
                { line: 47 }
            ]
        });
        assert.ok(breakpointsResponse.breakpoints.length > 0);
        assert.strictEqual(breakpointsResponse.breakpoints[0].verified, true);
        assert.strictEqual(breakpointsResponse.breakpoints[0].line, 47);

        // 4. ConfigurationDone
        const stoppedEventPromise = dapClient.waitForEvent('stopped');
        await dapClient.sendRequest('configurationDone');

        // 5. Stopped Event at Breakpoint
        const stoppedEventBody = await stoppedEventPromise;
        assert.strictEqual(stoppedEventBody.reason, 'breakpoint');

        // 6. StackTrace
        const stackTraceResponse = await dapClient.sendRequest('stackTrace', { threadId: 1 });
        assert.ok(stackTraceResponse.stackFrames.length >= 1);
        const topFrame = stackTraceResponse.stackFrames[0];
        assert.ok(topFrame.name === 'main' || topFrame.name.endsWith('::main'), 'Top frame must be main function');
        assert.strictEqual(topFrame.line, 47, 'Top frame must be on line 47');
        assert.ok(topFrame.source && topFrame.source.path, 'Top frame must have source path');
        assert.ok(topFrame.source.path.endsWith('main.slx'), 'Top frame path must point to main.slx');
        assert.ok(path.isAbsolute(topFrame.source.path), 'Top frame path must be canonical absolute path');

        // 7. Scopes
        const scopesResponse = await dapClient.sendRequest('scopes', { frameId: topFrame.id });
        assert.ok(scopesResponse.scopes.length >= 2);
        const localsScope = scopesResponse.scopes.find(scope => scope.name === 'Locals & Parameters');
        assert.ok(localsScope, 'Locals & Parameters scope must be present');
        assert.ok(localsScope.variablesReference > 0);

        // 8. Variables (Locals)
        const variablesResponse = await dapClient.sendRequest('variables', {
            variablesReference: localsScope.variablesReference
        });
        const originPoint = variablesResponse.variables.find(variable => variable.name === 'origin_point');
        assert.ok(originPoint, 'origin_point must be in locals');
        assert.match(originPoint.value, /^Point@0x[0-9A-Fa-f]+/, 'origin_point value must format as Point@0x...');
        assert.strictEqual(originPoint.type, 'Point');
        assert.ok(originPoint.variablesReference > 0, 'origin_point must have child variablesReference');

        const calculatedDistance = variablesResponse.variables.find(variable => variable.name === 'calculated_distance');
        assert.ok(calculatedDistance, 'calculated_distance must be in locals');
        assert.strictEqual(calculatedDistance.value, '30');
        assert.strictEqual(calculatedDistance.type, 'int32');

        // 9. Inspect object fields of origin_point
        const fieldsResponse = await dapClient.sendRequest('variables', {
            variablesReference: originPoint.variablesReference
        });
        const coordX = fieldsResponse.variables.find(field => field.name === 'coordinate_x');
        const coordY = fieldsResponse.variables.find(field => field.name === 'coordinate_y');
        assert.ok(coordX, 'coordinate_x must be in origin_point fields');
        assert.ok(coordY, 'coordinate_y must be in origin_point fields');
        assert.strictEqual(coordX.value, '10');
        assert.strictEqual(coordY.value, '20');

        // 10. Step Over (Next) to line 47
        const stepOverStoppedPromise = dapClient.waitForEvent('stopped');
        await dapClient.sendRequest('next', { threadId: 1 });
        const stepOverBody = await stepOverStoppedPromise;
        assert.strictEqual(stepOverBody.reason, 'step');

        const steppedStackTrace = await dapClient.sendRequest('stackTrace', { threadId: 1 });
        const steppedTopFrame = steppedStackTrace.stackFrames[0];
        assert.ok(steppedTopFrame.name === 'main' || steppedTopFrame.name.endsWith('::main'), 'Stepped top frame must be main function');
        assert.strictEqual(steppedTopFrame.line, 48, 'Stepped frame must be on line 48');

        // After step over, static_result should be 100
        const steppedScopes = await dapClient.sendRequest('scopes', { frameId: steppedTopFrame.id });
        const steppedLocalsScope = steppedScopes.scopes.find(scope => scope.name === 'Locals & Parameters');
        const steppedVariables = await dapClient.sendRequest('variables', {
            variablesReference: steppedLocalsScope.variablesReference
        });
        const staticResult = steppedVariables.variables.find(variable => variable.name === 'static_result');
        assert.ok(staticResult, 'static_result must be in locals after stepping line 47');
        assert.strictEqual(staticResult.value, '100');

        // 11. Evaluate Expression
        const evaluateResponse = await dapClient.sendRequest('evaluate', {
            expression: '15 + 25',
            frameId: steppedTopFrame.id
        });
        assert.strictEqual(evaluateResponse.result, '40');

        // 12. Continue to Completion
        const terminatedPromise = dapClient.waitForEvent('terminated');
        const exitedPromise = dapClient.waitForEvent('exited');
        await dapClient.sendRequest('continue', { threadId: 1 });

        await terminatedPromise;
        const exitedBody = await exitedPromise;
        assert.strictEqual(exitedBody.exitCode, 0);

        // 13. Verify Output Events
        const allOutputs = dapClient.collectedOutputMessages.join('');
        assert.ok(allOutputs.includes('Calculated distance from origin: 30'), 'Must capture distance println output');
        assert.ok(allOutputs.includes('Static addition result: 100'), 'Must capture static result println output');

        console.log(`[PASS] ${testLabel} completed with 100% strict assertions verified!`);
    } finally {
        dapProcess.kill();
    }
}

async function runFullDapSuite() {
    console.log('=== Running Solix Headless Industrial DAP E2E Conformance Suite ===');

    const os = await import('os');
    const tempWorkspace = fs.mkdtempSync(path.join(os.tmpdir(), 'solix-dap-rpc-'));
    const srcDir = path.join(tempWorkspace, 'src');
    const buildDir = path.join(tempWorkspace, 'build', 'debug');
    fs.mkdirSync(srcDir, { recursive: true });
    fs.mkdirSync(buildDir, { recursive: true });

    const mainSourceContent = [
        'package dap_test_workspace;',
        '',
        'native void println(string msg);',
        '',
        '',
        '/// Represents a 2D coordinate point with reference-counted heap allocation.',
        'class Point {',
        '    public int32 coordinate_x;',
        '    public int32 coordinate_y;',
        '',
        '    public Point(int32 initial_x, int32 initial_y) {',
        '        this.coordinate_x = initial_x;',
        '        this.coordinate_y = initial_y;',
        '    }',
        '',
        '    /// Returns the Manhattan distance from origin.',
        '    public int32 distance_from_origin() {',
        '        return this.coordinate_x + this.coordinate_y;',
        '    }',
        '',
        '    /// Deterministic destructor invoked upon scope exit.',
        '    ~Point() {',
        '        // Resource cleanup upon destruction',
        '    }',
        '}',
        '',
        '/// Static mathematical utilities class.',
        'class MathHelper {',
        '    /// Computes the sum of two integers.',
        '    public static int32 add(int32 operand_first, int32 operand_second) {',
        '        return operand_first + operand_second;',
        '    }',
        '}',
        '',
        '/// Unit test recognized by the Run Test CodeLens button.',
        'void test_addition() {',
        '    int32 computed_sum = MathHelper::add(15, 25);',
        '    assert(computed_sum == 40, "Sum calculation mismatch");',
        '}',
        '',
        '/// Application entry point recognized by the Run Main CodeLens button.',
        'void main() {',
        '    Point origin_point = new Point(10, 20);',
        '    int32 calculated_distance = origin_point.distance_from_origin();',
        '    println("Calculated distance from origin: 30");',
        '',
        '    int32 static_result = MathHelper::add(50, 50);',
        '    println("Static addition result: 100");',
        '}',
        '',
    ].join('\n');

    const sourceFilePath = path.join(srcDir, 'main.slx');
    fs.writeFileSync(sourceFilePath, mainSourceContent, 'utf8');

    const solixBinary = locateSolixBinary();
    const compiledBinaryPath = path.join(buildDir, 'dap_test_workspace.slxb');
    const compileResult = childProcess.spawnSync(
        solixBinary,
        ['compile', sourceFilePath, '--profile', 'debug', '-o', compiledBinaryPath],
        { encoding: 'utf8' }
    );
    assert.strictEqual(
        compileResult.status,
        0,
        `solix compile for DAP fixture failed: ${compileResult.stderr}`
    );

    try {
        // Run Suite A: Direct .slx Single Source File Session
        await executeDapSession(
            sourceFilePath,
            sourceFilePath,
            tempWorkspace,
            'Direct Source File (.slx) DAP Session'
        );

        // Run Suite B: Precompiled Binary (.slxb) Session
        await executeDapSession(
            compiledBinaryPath,
            sourceFilePath,
            tempWorkspace,
            'Precompiled Binary (.slxb) DAP Session'
        );
    } finally {
        fs.rmSync(tempWorkspace, { recursive: true, force: true });
    }

    console.log('\n=== ALL INDUSTRIAL DAP E2E CONFORMANCE TESTS PASSED CLEANLY! ===\n');
}

runFullDapSuite().catch((error) => {
    console.error('Industrial DAP E2E Suite Failed:', error);
    process.exit(1);
});
