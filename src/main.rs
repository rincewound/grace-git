use std::path::PathBuf;

use clap::{Arg, Command};

mod grace;

fn cli() -> Command {
    Command::new("grace")
        .about("Your Git Nanny")
        .subcommand_required(true)
        .subcommand(Command::new("init").about("Initialize a new project"))
        .subcommand(
            Command::new("registry")
                .about("Interact with package registries")
                .subcommand_required(true)
                .subcommand(
                    Command::new("add")
                        .arg_required_else_help(true)
                        .arg(Arg::new("uri").help("The URI of the registry")),
                )
                .subcommand(Command::new("update"))
                .subcommand(Command::new("remove")),
        )
        .subcommand(
            Command::new("package")
                .about("Interact with packages")
                .subcommand_required(true)
                .subcommand(Command::new("install").about("Installs all packages from grace.toml"))
                .subcommand(
                    Command::new("update").about(
                        "Updates all packages from grace.toml to the latest compatible version",
                    ),
                )
                .subcommand(Command::new("publish")),
        )
}

fn main() {
    let matches = cli().get_matches();

    match matches.subcommand() {
        Some(("init", _)) => {
            init_project();
        }
        Some(("registry", submatches)) => {
            do_registry_command(submatches);
        }
        Some(("package", submatches)) => {
            do_package_command(submatches);
        }
        _ => unreachable!(),
    }
}

fn do_package_command(submatches: &clap::ArgMatches) {
    match submatches.subcommand() {
        Some(("install", _)) => {
            grace::package::PackageDependency::install(PathBuf::from("."))
        }

        _ => unreachable!(),
    }
}

fn init_project() {
    let cwd = PathBuf::from(".");
    grace::project::Project::init(cwd);
}

fn do_registry_command(submatches: &clap::ArgMatches) {
    match submatches.subcommand() {
        Some(("add", submatches)) => {
            let project = grace::project::Project::open(PathBuf::from("."));
            project.add_registry(submatches.get_one::<String>("uri").unwrap().clone());
        }

        Some(("update", _)) => {
            let project = grace::project::Project::open(PathBuf::from("."));
            project.update_registries();
        }

        _ => unreachable!(),
    }
}
