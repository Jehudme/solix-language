import * as path from 'path';
import * as fs from 'fs';

export async function run(): Promise<void> {
    const testsRoot = path.resolve(__dirname, '.');
    const testFiles = fs.readdirSync(testsRoot).filter((fileName) => fileName.endsWith('.test.js'));

    for (const testFile of testFiles) {
        const testFilePath = path.join(testsRoot, testFile);
        console.log(`Running suite: ${testFile}`);
        const testModule = await import(testFilePath);
        if (typeof testModule.runSuite === 'function') {
            await testModule.runSuite();
        }
    }
}
