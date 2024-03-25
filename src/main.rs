use std::io::{self, BufReader, Read, Write};
use std::path::Path;
use std::{env, path::PathBuf};
use std::{process, thread};

use clap::{Command, CommandFactory, Parser, Subcommand, ValueEnum, ValueHint};
use clap_complete::{Generator, Shell};
use console::style;

#[derive(Debug, Eq, PartialEq, PartialOrd, Ord, Copy, Clone, ValueEnum)]
enum Builders {
    Make,
    Task,
    Earthly,
    Mix,
    Cargo,
    Go,
    DockerCompose,
    Docker,
}

static RECIPES: [(&str, Builders); 8] = [
    ("Makefile", Builders::Make),
    ("Taskfile.yml", Builders::Task),
    ("Earthfile", Builders::Earthly),
    ("mix.exs", Builders::Mix),
    ("Cargo.toml", Builders::Cargo),
    ("go.mod", Builders::Go),
    ("Dockerfile", Builders::Docker),
    ("docker-compose.yml", Builders::DockerCompose),
];

#[derive(Clone, Debug, PartialEq, Eq)]
struct BuilderCmd {
    name: String,
    run: Vec<String>,
    test: Vec<String>,
    build: Vec<String>,
}

impl From<Builders> for BuilderCmd {
    fn from(builder: Builders) -> Self {
        match builder {
            Builders::Make => BuilderCmd::builder("make"),
            Builders::Task => BuilderCmd::builder("task"),
            Builders::Earthly => BuilderCmd::new("earthly")
                .run_arg("+run")
                .test_arg("+test")
                .build_arg("+build"),
            Builders::Mix => BuilderCmd::builder("mix"),
            Builders::Cargo => BuilderCmd::builder("cargo"),
            Builders::Go => BuilderCmd::new("go")
                .run_arg("run")
                .run_arg("./...")
                .test_arg("test")
                .test_arg("./...")
                .build_arg("build")
                .build_arg("./..."),
            Builders::DockerCompose => BuilderCmd::builder("docker-compose"),
            Builders::Docker => BuilderCmd::builder("docker"),
        }
    }
}

impl BuilderCmd {
    fn new(name: &str) -> Self {
        Self::create(
            name,
            Default::default(),
            Default::default(),
            Default::default(),
        )
    }

    fn builder(name: &str) -> Self {
        Self::new(name)
            .run_arg("run")
            .test_arg("test")
            .build_arg("build")
    }

    fn create(name: &str, run: Vec<String>, test: Vec<String>, build: Vec<String>) -> Self {
        BuilderCmd {
            name: name.to_string(),
            run,
            test,
            build,
        }
    }

    fn run_arg(mut self, arg: &str) -> Self {
        self.run.push(arg.to_string());
        self
    }

    fn test_arg(mut self, arg: &str) -> Self {
        self.test.push(arg.to_string());
        self
    }

    fn build_arg(mut self, arg: &str) -> Self {
        self.build.push(arg.to_string());
        self
    }

    fn run(&self) -> Vec<String> {
        self.run.clone()
    }

    fn test(&self) -> Vec<String> {
        self.test.clone()
    }

    fn build(&self) -> Vec<String> {
        self.build.clone()
    }

    fn name(&self) -> String {
        self.name.clone()
    }
}

#[derive(Debug)]
struct Builder {
    pwd: PathBuf,
    cmd: BuilderCmd,
}

impl Builder {
    fn new(pwd: PathBuf, cmd: BuilderCmd) -> Self {
        Builder { pwd, cmd }
    }

    fn cmd(&self) -> String {
        self.cmd.name()
    }

    fn run(&self) -> Vec<String> {
        self.cmd.run()
    }

    fn test(&self) -> Vec<String> {
        self.cmd.test()
    }

    fn build(&self) -> Vec<String> {
        self.cmd.build()
    }

    fn pwd(&self) -> PathBuf {
        self.pwd.clone()
    }
}

fn find(pwd: &Path) -> Option<Builders> {
    if !pwd.is_dir() {
        return None;
    }

    RECIPES.iter().find_map(|(name, builder)| {
        let path = pwd.join(name);
        if path.exists() {
            return Some(builder.to_owned());
        }

        None
    })
}

fn discover(pwd: PathBuf) -> Option<Builder> {
    find(&pwd).map(|builder| {
        let cmd = builder.into();
        Builder::new(pwd, cmd)
    })
}

fn exec(pwd: PathBuf, cmd: &str, args: Vec<String>) -> Result<(), io::Error> {
    let ce = duct::cmd(cmd, args).dir(pwd).stderr_to_stdout().reader()?;

    let t = thread::spawn(move || -> Result<(), io::Error> {
        let mut reader = BufReader::new(ce);

        let mut o = std::io::stdout();
        let mut buf = [0u8; 1024];

        while let Ok(s) = reader.read(&mut buf) {
            if s == 0 {
                break;
            }

            o.write_all(&buf[0..s])?;
        }

        Ok(())
    });

    t.join().unwrap()
}

#[derive(Parser)]
#[command(author, version, about)]
struct BuildAny {
    /// Shell completion.
    #[arg(short, long)]
    completion: Option<Shell>,

    /// Project build tool.
    #[arg(short, long, value_enum)]
    target: Option<Builders>,

    /// Project directory to execute the command.
    #[arg(short, long, value_hint=ValueHint::DirPath)]
    dir: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Build command.
    Build,

    /// Run command.
    Run,

    /// Test command.
    Test,
}

fn print_completions<G: Generator>(gen: G, cmd: &mut Command) {
    clap_complete::generate(gen, cmd, cmd.get_name().to_string(), &mut io::stdout());
}

fn main() {
    let cli = BuildAny::parse();
    if let Some(sh) = cli.completion {
        let mut cmd = BuildAny::command();
        print_completions::<clap_complete::Shell>(sh, &mut cmd);
        process::exit(0);
    }

    let pwd = cli.dir.or_else(|| env::current_dir().ok());
    let br = pwd.and_then(|p| {
        if let Some(b) = cli.target {
            Some(Builder::new(p, b.into()))
        } else {
            discover(p)
        }
    });

    if let Some(b) = br {
        eprintln!("⌛ Running: {}\n", style(b.cmd()).green());

        let res = match cli.command {
            Commands::Build => exec(b.pwd(), &b.cmd(), b.build()),
            Commands::Run => exec(b.pwd(), &b.cmd(), b.run()),
            Commands::Test => exec(b.pwd(), &b.cmd(), b.test()),
        };

        if let Err(e) = res {
            eprintln!("💀 Failed: {}", style(e).red());
        }
    };
}

#[cfg(test)]
mod test_buildany {
    use std::fs::File;

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_find_pwd_must_dir() -> Result<(), io::Error> {
        let tmp = tempdir::TempDir::new("buildany_find")?;
        let fp = tmp.path().join("Cargo.toml");
        let _f = File::create(&fp)?;

        assert_eq!(find(&fp.clone()), None);

        Ok(())
    }

    #[test]
    fn test_find_pwd_must_exists() -> Result<(), io::Error> {
        let tmp = tempdir::TempDir::new("buildany_find")?;
        let fp = tmp.path().join("nonexistent");

        assert_eq!(find(&fp), None);

        Ok(())
    }

    #[test]
    fn test_find_pwd_unsup() -> Result<(), io::Error> {
        let tmp = tempdir::TempDir::new("buildany_find")?;
        let fp = tmp.path().join("unsupported.yml");
        let _ = File::create(fp)?;

        assert_eq!(find(tmp.path()), None);

        Ok(())
    }

    #[test]
    fn test_find_ordered() -> Result<(), io::Error> {
        let tmp = tempdir::TempDir::new("buildany_find")?;

        let fpc = tmp.path().join("Cargo.toml");
        let _f = File::create(fpc)?;

        let make = "Makefile";
        let fp = tmp.path().join(make);
        let _f = File::create(fp)?;

        assert_eq!(find(tmp.path()), Some(Builders::Make));

        Ok(())
    }
}
