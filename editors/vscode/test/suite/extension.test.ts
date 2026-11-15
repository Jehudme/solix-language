import * as assert from 'assert';
import * as vscode from 'vscode';

export async function runSuite(): Promise<void> {
    console.log('Verifying extension activation and command registration...');

    const extension = vscode.extensions.getExtension('solix.solix');
    assert.ok(extension, 'Expected solix extension to be discovered by VS Code');

    if (!extension.isActive) {
        await extension.activate();
    }
    assert.strictEqual(extension.isActive, true, 'Expected solix extension to activate cleanly');

    const registeredCommands = await vscode.commands.getCommands(true);
    const expectedCommands = [
        'solix.runMain',
        'solix.runTest',
        'solix.buildProject',
        'solix.syncPackages',
        'solix.showDisassembly',
        'solix.showSynthesizedApi',
        'solix.restartServer',
    ];

    for (const expectedCommand of expectedCommands) {
        assert.ok(
            registeredCommands.includes(expectedCommand),
            `Expected ${expectedCommand} to be registered in VS Code command registry`
        );
    }

    console.log('  ✓ Extension activation and all 7 commands verified.');
}
