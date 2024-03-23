use std::fs::DirEntry;
use std::{env, fs, path::PathBuf};

use clap::{Parser, Subcommand};

const PWD_KEY: &str = "PWD";

#[derive(Debug, Eq, PartialEq, PartialOrd, Ord, Copy, Clone)]
enum Builders {
    Make,
    Task,
    Earth,
    Mix,
    Cargo,
    Go,
    DockerCompose,
    Docker,
}

static RECIPES: [(&str, Builders); 8] = [
    ("Makefile", Builders::Make),
    ("Taskfile.yml", Builders::Task),
    ("Earthfile", Builders::Earth),
    ("mix.exs", Builders::Mix),
    ("Cargo.toml", Builders::Cargo),
    ("go.mod", Builders::Go),
    ("Dockerfile", Builders::Docker),
    ("docker-compose.yml", Builders::DockerCompose),
];

struct Builder {
    pwd: PathBuf,
    cmd: String,
    run: Vec<String>,
    test: Vec<String>,
    build: Vec<String>,
    recipe: String,
    priority: Builders,
}

fn discover(pwd: PathBuf) -> Option<Builder> {
    if !pwd.is_dir() {
        return None;
    }

    RECIPES
        .iter()
        .find_map(|(name, builder)| {
            let path = pwd.join(name);
            if path.exists() {
                return Some((pwd.clone(), name, builder));
            }
            None
        })
        .map(|(pwd, name, builder)| {
            let (dr, dt, db) = (
                vec!["run".to_string()],
                vec!["test".to_string()],
                vec!["build".to_string()],
            );
            let (c, r, t, b) = match builder {
                Builders::Make => ("make".to_string(), dr, dt, db),
                Builders::Task => ("task".to_string(), dr, dt, db),
                Builders::Earth => (
                    "earth".to_string(),
                    vec!["+run".to_string()],
                    vec!["+test".to_string()],
                    vec!["+build".to_string()],
                ),
                Builders::Mix => ("mix".to_string(), dr, dt, db),
                Builders::Cargo => ("cargo".to_string(), dr, dt, db),
                Builders::Go => (
                    "go".to_string(),
                    vec!["run".to_string(), "./...".to_string()],
                    vec!["test".to_string(), "./...".to_string()],
                    vec!["build".to_string(), "./...".to_string()],
                ),
                Builders::DockerCompose => ("docker-compose".to_string(), dr, dt, db),
                Builders::Docker => ("docker".to_string(), dr, dt, db),
            };
            Builder {
                pwd,
                cmd: c,
                run: r,
                test: t,
                build: b,
                recipe: name.to_string(),
                priority: *builder,
            }
        })
}

#[derive(Parser)]
#[command(author, version, about)]
struct BuildAny {
    #[command(subcommand)]
    command: Commands,

    pwd: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    Build,
    Run,
    Test,
}

fn main() {
    let cli = BuildAny::parse();

    let pwd = cli.pwd.or_else(|| env::var(PWD_KEY).ok());

    // match cli.command {
    //     // Build => ,
    //     // Test => ,
    // }
}
