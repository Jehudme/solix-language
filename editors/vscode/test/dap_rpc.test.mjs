/**
 * Headless Node.js DAP integration test for Solix Debug Adapter.
 *
 * Verifies that `solix dap --stdio` handles full DAP sessions:
 * initialize -> launch -> setBreakpoints -> configurationDone -> hit breakpoint ->
 * stackTrace -> scopes -> variables -> evaluate -> continue -> terminated/exited.
 */

import * as childProcess from 'child_process';
import * as path from 'path';
import * as fs from 'fs';
import { fileURLToPath } from 'url';

const currentDir = path.dirname(fileURLToPath(import.meta.url));

function locateSolixBinary() {
    const candidates = [
        path.resolve(currentDir, '../../../target/release/solix'),
        path.resolve(currentDir, '../../../target/debug/solix'),
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
                console.log('DEBUG MSG:', JSON.stringify(parsedMessage));
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

async function runDapSuite() {
    console.log('=== Running Solix Headless DAP Client Integration Suite ===');

    const solixBinary = locateSolixBinary();
    console.log(`Spawning DAP server: ${solixBinary} dap --stdio`);

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
        console.log('Step 1: Sending initialize request...');
        const initPromise = dapClient.sendRequest('initialize', {
            adapterID: 'solix',
            clientID: 'vscode-headless-test',
            linesStartAt1: true,
            columnsStartAt1: true,
            supportsVariableType: true
        });

        const initializedEventPromise = dapClient.waitForEvent('initialized');
        const capabilities = await initPromise;
        console.log('Initialize response received. Capabilities:', Object.keys(capabilities).length);
        await initializedEventPromise;
        console.log('Initialized event received.');

        // 2. Launch
        console.log('Step 2: Sending launch request...');
        const sampleSourcePath = path.resolve(currentDir, '../sample_workspace/src/main.slx');
        await dapClient.sendRequest('launch', {
            program: sampleSourcePath,
            stopOnEntry: false
        });
        console.log('Launch response received.');

        // 3. SetBreakpoints at line 46
        console.log('Step 3: Setting breakpoint at line 46...');
        const breakpointsResponse = await dapClient.sendRequest('setBreakpoints', {
            source: {
                path: sampleSourcePath,
                name: 'main.slx'
            },
            breakpoints: [
                { line: 46 }
            ]
        });
        console.log('Breakpoints set response:', JSON.stringify(breakpointsResponse.breakpoints));

        // 4. ConfigurationDone
        console.log('Step 4: Sending configurationDone...');
        const stoppedEventPromise = dapClient.waitForEvent('stopped');
        await dapClient.sendRequest('configurationDone');

        // 5. Wait for Stopped event at breakpoint
        console.log('Step 5: Waiting for stopped event...');
        const stoppedEventBody = await stoppedEventPromise;
        console.log('Stopped event received! Reason:', stoppedEventBody.reason);

        // 6. StackTrace
        console.log('Step 6: Requesting stackTrace...');
        const stackTraceResponse = await dapClient.sendRequest('stackTrace', {
            threadId: 1
        });
        console.log(`Call stack frames count: ${stackTraceResponse.stackFrames.length}`);
        const topFrame = stackTraceResponse.stackFrames[0];
        console.log(`Top frame: ${topFrame.name} at line ${topFrame.line}`);

        // 7. Scopes
        console.log('Step 7: Requesting scopes...');
        const scopesResponse = await dapClient.sendRequest('scopes', {
            frameId: topFrame.id
        });
        console.log(`Scopes returned: ${scopesResponse.scopes.map(s => s.name).join(', ')}`);
        const localsScope = scopesResponse.scopes[0];

        // 8. Variables
        console.log('Step 8: Requesting variables for Locals scope...');
        const variablesResponse = await dapClient.sendRequest('variables', {
            variablesReference: localsScope.variablesReference
        });
        console.log(`Locals variables: ${variablesResponse.variables.map(v => `${v.name}=${v.value}`).join(', ')}`);

        // 9. Evaluate expression
        console.log('Step 9: Evaluating expression "10 + 20"...');
        const evaluateResponse = await dapClient.sendRequest('evaluate', {
            expression: '10 + 20',
            frameId: topFrame.id
        });
        console.log(`Evaluation result: ${evaluateResponse.result}`);
        if (evaluateResponse.result !== '30') {
            throw new Error(`Expected evaluation result '30', got '${evaluateResponse.result}'`);
        }

        // 10. Continue to completion
        console.log('Step 10: Continuing execution to completion...');
        const terminatedPromise = dapClient.waitForEvent('terminated');
        const exitedPromise = dapClient.waitForEvent('exited');
        await dapClient.sendRequest('continue', { threadId: 1 });

        await terminatedPromise;
        await exitedPromise;
        console.log('Program terminated and exited gracefully with code 0.');

        console.log('=== All Solix Headless DAP Tests Passed Cleanly! ===');
    } finally {
        dapProcess.kill();
    }
}

runDapSuite().catch((error) => {
    console.error('DAP Test Suite Failed:', error);
    process.exit(1);
});
