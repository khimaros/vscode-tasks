extern crate clap;

extern crate parser;

use parser::{BaseTaskConfiguration, parse};

use std::path;
use std::env;
use std::io::{self, Write, Result};
use std::process::{Command, Output};

use clap::{Arg, App, SubCommand, crate_authors, crate_description, crate_name, crate_version};

fn task_name(label: String) -> String {
    label.replace(" ", "_").to_lowercase()
}

fn execute_task(task: &BaseTaskConfiguration) -> Result<Output> {
    let mut cmd = &mut Command::new(task.command.clone());
    for arg in task.args.clone() {
        cmd = cmd.arg(arg);
    }
    return cmd.output();
}

fn find_in_parents(path: &path::PathBuf, child: String) -> Result<path::PathBuf> {
    // canonicalize the input path, then search for ".vscode/tasks.json" relative to each parent
    // until the root of filesystem is reached.
    let mut running = true;
    let mut rp = path.canonicalize().unwrap();
    while running {
        let tp = rp.join(&child);
        if tp.is_file() {
            return Ok(tp);
        }
        running = rp.pop();
    }
    return Err(io::Error::new(io::ErrorKind::NotFound, "could not locate config"));
}

fn find_config_path() -> path::PathBuf {
    match env::var("VSCODE_TASKS_CONFIG_PATH") {
        // if VSCODE_TASKS_CONFIG_PATH env var is defined, use that.
        // otherwise, try to search recursively through parents to find `.vscode/tasks.json`
        Ok(val) => {
            let path = path::Path::new(&val);
            return path.to_path_buf();
        },
        Err(_) => {
            let cwd = env::current_dir().expect("could not determine current working directory");
            let cwd_path = path::Path::new(cwd.to_str().unwrap());
            return find_in_parents(&cwd_path.to_path_buf(), String::from(".vscode/tasks.json")).unwrap();
        }
    };
}

fn main() {
    let config = find_config_path();

    let tc = parse(config.as_path()).expect("parsing was unsuccessful");

    let mut app = App::new(crate_name!())
        .author(crate_authors!("\n"))
        .about(crate_description!())
        .version(crate_version!())
        .arg(
            Arg::with_name("list")
                .short("l")
                .long("list")
                .help("Displays the list of tasks."))
        .arg(
            Arg::with_name("v")
                .short("v")
                .multiple(true)
                .help("Sets the level of verbosity"));

    for task in &tc.tasks {
        let subcmd = SubCommand::with_name(task_name(task.label.clone()).as_str())
                .about(&*task.label);

        /*
        for opt in task.options {
            subcmd = subcmd.arg(
                Arg::with_name("ARG")
                    .help("arg name")
                    .index(1));
        }
        */

        app = app.subcommand(subcmd);
    }

    let matches = app.clone().get_matches();

    if matches.is_present("list") {
        println!("Task list:");
        for task in &tc.tasks {
            println!("  {} ({:?})", task_name(task.label.clone()).as_str(), task.task_type);
        }
    }
    else {
        for task in &tc.tasks {
            if matches.is_present(task_name(task.label.clone())) {
                println!("executing task {}", task.label);
                let output = execute_task(task).expect("failed to execute task");
                println!("{}", output.status);

                io::stdout().write_all(&output.stdout).unwrap();
                io::stderr().write_all(&output.stderr).unwrap();

                assert!(output.status.success());
                println!("task executed successfully");
                return;
            }
        }
        // else
        app.print_help().unwrap();
        println!();
    }
}
