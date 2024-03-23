use std::io::{BufRead, BufReader, Error, ErrorKind, Read, Write};
use std::process::{Command, Stdio};
use std::thread;
use std::{env, path::PathBuf};

use clap::{Parser, Subcommand};

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

#[allow(dead_code)]
#[derive(Debug)]
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
            println!("{:?} ", path);
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

fn exec(pwd: PathBuf, cmd: &str, args: Vec<String>) -> Result<(), Error> {
    let mut child = Command::new(cmd)
        .current_dir(pwd)
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let mut stdout = child
        .stdout
        .take()
        .ok_or_else(|| Error::new(ErrorKind::Other, "Could not capture standard output."))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| Error::new(ErrorKind::Other, "Could not capture standard error."))?;

    // let mut rout = BufReader::new(stdout);
    let rerr = BufReader::new(stderr);

    let tout = thread::spawn(move || -> Result<(), Error> {
        let mut buf = [0u8; 1024];
        loop {
            let num_read = stdout.read(&mut buf)?;
            if num_read == 0 {
                break;
            }

            let buf = &buf[..num_read];
            std::io::stdout().write_all(buf)?;
        }
        Ok(())
        // rout.lines()
        // // .map_while(|line| line.ok())
        // .filter_map(|line| line.ok())
        // .for_each(|line| {
        //     // std::io::stdout().write_all(line.as_bytes()).unwrap();
        //     println!("{}", line)
        // });
        // rout.bytes()
        //     .map_while(|b| b.ok())
        //     .map(|b| std::io::stdout().write_all(&[b]))
    });

    let terr = thread::spawn(move || {
        rerr.bytes()
            .map_while(|b| b.ok())
            .map(|b| std::io::stderr().write_all(&[b]))
    });

    let _ = tout.join();
    jjterr.join();

    child.wait()?;

    Ok(())
}

#[derive(Parser)]
#[command(author, version, about)]
struct BuildAny {
    #[command(subcommand)]
    command: Commands,

    pwd: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    Build,
    Run,
    Test,
}

fn main() {
    let cli = BuildAny::parse();

    let pwd = cli.pwd.or_else(|| env::current_dir().ok());
    println!("{:?} ", pwd);
    let br = pwd.and_then(discover);
    println!("{:?} ", br);
    if let Some(b) = br {
        let res = match cli.command {
            Commands::Build => exec(b.pwd, &b.cmd, b.build),
            Commands::Run => exec(b.pwd, &b.cmd, b.run),
            Commands::Test => exec(b.pwd, &b.cmd, b.test),
        };
        if let Err(e) = res {
            eprintln!("{}", e);
        }
    };
    println!("{:?} ", env::var("PWD"));
}
