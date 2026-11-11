import * as vscode from 'vscode';

/**
 * Task provider exposing standard Solix build and execution workflows.
 */
export class SolixTaskProvider implements vscode.TaskProvider {
    static readonly taskType = 'solix';

    constructor(private readonly solixBinaryPath: string) {}

    public provideTasks(): vscode.Task[] {
        return this.createTasks();
    }

    public resolveTask(task: vscode.Task): vscode.Task | undefined {
        const taskDefinition = task.definition;
        if (taskDefinition.type === SolixTaskProvider.taskType) {
            return this.buildTask(
                taskDefinition.command || 'build',
                taskDefinition.args || [],
                task.scope || vscode.TaskScope.Workspace
            );
        }
        return undefined;
    }

    private createTasks(): vscode.Task[] {
        const workspaceFolders = vscode.workspace.workspaceFolders;
        if (!workspaceFolders || workspaceFolders.length === 0) {
            return [];
        }

        const taskList: vscode.Task[] = [];

        // 1. Build Debug Task
        const buildDebugTask = this.buildTask('build', ['--debug'], workspaceFolders[0]);
        buildDebugTask.group = vscode.TaskGroup.Build;
        taskList.push(buildDebugTask);

        // 2. Build Release Task
        const buildReleaseTask = this.buildTask('build', ['--release'], workspaceFolders[0]);
        buildReleaseTask.group = vscode.TaskGroup.Build;
        taskList.push(buildReleaseTask);

        // 3. Build Test Task
        const buildTestTask = this.buildTask('build', ['--test'], workspaceFolders[0]);
        buildTestTask.group = vscode.TaskGroup.Test;
        taskList.push(buildTestTask);

        // 4. Run Task
        const runTask = this.buildTask('run', [], workspaceFolders[0]);
        taskList.push(runTask);

        return taskList;
    }

    private buildTask(
        subcommand: string,
        extraArguments: string[],
        scope: vscode.WorkspaceFolder | vscode.TaskScope
    ): vscode.Task {
        const allArguments = [subcommand, ...extraArguments];
        const execution = new vscode.ProcessExecution(this.solixBinaryPath, allArguments);

        const taskName = `solix ${allArguments.join(' ')}`;
        return new vscode.Task(
            { type: SolixTaskProvider.taskType, command: subcommand, args: extraArguments },
            scope,
            taskName,
            'solix',
            execution,
            ['$solix']
        );
    }
}
