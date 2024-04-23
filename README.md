# VSCODE TASKS

low-dependency command line interface to vscode tasks.json files

# fetch

    $ git clone ...
    $ cd vscode-tasks

# build

    $ cargo build --release

# usage

By default, `vscode-tasks` checks recursively from current working directory through
parent directories for a file named `.vscode/tasks.json`.

help is displayed by default:

    $ ./target/release/vscode-tasks

display usage for a specific task:

    $ ./target/release/vscode-tasks help <task>
    $ ./target/release/vscode-tasks <task> --help

run a task:

    $ ./target/release/vscode-tasks <task> [options]

# configuration

`tasks.json` search path can be overridden with the `VSCODE_TASKS_CONFIG_PATH`
environment variable.

    $ VSCODE_TASKS_CONFIG_PATH="examples/tasks.json" cargo run -- <task>

# contributing

## tests

    $ cargo test

## references

[tasks.json variables reference](https://code.visualstudio.com/docs/editor/variables-reference)

[processTaskSystem.run() implementation](https://github.com/Microsoft/vscode/blob/b2ac8d2dcfab452b3279057133335017b572d8a2/src/vs/workbench/parts/tasks/node/processTaskSystem.ts#L96)

[taskConfiguration code](https://github.com/Microsoft/vscode/blob/b2ac8d2dcfab452b3279057133335017b572d8a2/src/vs/workbench/parts/tasks/node/taskConfiguration.ts)

[schema for tasks.json](https://code.visualstudio.com/docs/editor/tasks-appendix)
