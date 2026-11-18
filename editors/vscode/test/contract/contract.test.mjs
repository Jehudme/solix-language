import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const currentDir = path.dirname(fileURLToPath(import.meta.url));
const vscodeRoot = path.resolve(currentDir, '../..');

function readJson(relPath) {
    const fullPath = path.join(vscodeRoot, relPath);
    return JSON.parse(fs.readFileSync(fullPath, 'utf8'));
}

test('Layer 3: package.json entrypoints, commands, and clean build layout contract', () => {
    const pkg = readJson('package.json');

    assert.strictEqual(pkg.main, './build/dist/extension.js', 'Extension main must point to ./build/dist/extension.js');
    assert.ok(!fs.existsSync(path.join(vscodeRoot, 'sample_workspace')), 'sample_workspace directory must be removed');

    const contributedCommandIds = new Set(pkg.contributes.commands.map((cmd) => cmd.command));
    for (const expectedCmd of [
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
    ]) {
        assert.ok(contributedCommandIds.has(expectedCmd), `Missing contributed command '${expectedCmd}'`);
    }

    for (const eventName of pkg.activationEvents) {
        if (eventName.startsWith('onCommand:')) {
            const cmdName = eventName.slice('onCommand:'.length);
            assert.ok(
                contributedCommandIds.has(cmdName),
                `Activation event '${eventName}' references undeclared command '${cmdName}'`
            );
        }
    }

    const srcCombined =
        fs.readFileSync(path.join(vscodeRoot, 'src', 'extension.ts'), 'utf8') +
        fs.readFileSync(path.join(vscodeRoot, 'src', 'commands.ts'), 'utf8');
    for (const cmdId of contributedCommandIds) {
        assert.ok(
            srcCombined.includes(cmdId),
            `Contributed command '${cmdId}' must be wired in src/`
        );
    }

    const configProps = pkg.contributes.configuration.properties;
    assert.ok(configProps['solix.lsp.path'], 'Must expose solix.lsp.path setting');
    assert.ok(configProps['solix.lsp.arguments'], 'Must expose solix.lsp.arguments setting');
    assert.ok(configProps['solix.trace.server'], 'Must expose solix.trace.server setting');
});

test('Layer 3: language-configuration, snippets, and TextMate grammars are valid and modern', () => {
    const langConfig = readJson('language-configuration.json');
    assert.ok(langConfig.comments && langConfig.brackets, 'language-configuration.json must define comments & brackets');

    const snippets = readJson('snippets/solix.code-snippets');
    assert.ok(Object.keys(snippets).length >= 3, 'Must ship Solix code snippets');
    for (const [name, snip] of Object.entries(snippets)) {
        assert.ok(snip.prefix, `Snippet '${name}' must define prefix`);
        assert.ok(Array.isArray(snip.body) && snip.body.length > 0, `Snippet '${name}' must have non-empty body`);
    }

    for (const grammarRel of ['syntaxes/solix.tmLanguage.json', 'syntaxes/solix-disasm.tmLanguage.json']) {
        const grammar = readJson(grammarRel);
        assert.ok(grammar.scopeName, `${grammarRel} must declare scopeName`);
        assert.ok(Array.isArray(grammar.patterns) && grammar.patterns.length > 0, `${grammarRel} must define patterns`);
        if (grammarRel.endsWith('solix.tmLanguage.json')) {
            assert.ok(grammar.repository && typeof grammar.repository === 'object', `${grammarRel} must define repository`);
        }
    }
});
