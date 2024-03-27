# buildany

Simple tool that auto detect and executes a build system target.

## Supported

1. [Make](https://www.gnu.org/software/make/manual/make.html)
2. [Taskfile](https://taskfile.dev/) (**searches for *Taskfile.yml***)
3. [Earthly](https://earthly.dev/)
4. [Mix](https://hexdocs.pm/elixir/introduction-to-mix.html) (**only build, test**)
5. [Cargo](https://doc.rust-lang.org/stable/cargo/)
6. [go](https://pkg.go.dev/cmd/go)
7. [Docker compose](https://docs.docker.com/compose/reference/) (**only build**)
8. [Docker](https://www.docker.com/) (**only build**)

## Install

Download and extract under a `$PATH` registered directory.

## Usage

```sh
Usage: buildany [OPTIONS] <COMMAND>

Commands:
  build  Build command
  run    Run command
  test   Test command
  help   Print this message or the help of the given subcommand(s)

Options:
  -c, --completion <COMPLETION>  Shell completion [possible values: bash, elvish, fish, powershell, zsh]
  -t, --target <TARGET>          Project build tool [possible values: make, task, earthly, mix, cargo, go, docker-compose, docker]
  -d, --dir <DIR>                Project directory to execute the command
  -h, --help                     Print help
  -V, --version                  Print version

```

## Examples

We can use this command to create common keybindings for multiple build systems.

### Build keybind for "Windows Terminal"

We can add a **sendInput action**.[^1] Press `ctrl+shift+p` search for open settings json and add the following:

```json
{ "command": {"action": "sendInput", "input": "buildany build\r"}, "keys": "ctrl+shift+b" }
```

### Build keybind for "fish shell"

Declare a new **bind**[^2] in **config.fish**, open `$HOME/.config/fish/config` and
add the following line to bind `ctrl+b` for build `bind \cb 'buildany build'` .

[^1]: [Windows Terminal - tips and tricks](https://learn.microsoft.com/en-us/windows/terminal/tips-and-tricks#send-input-commands-with-a-key-binding)
[^2]: [fish - handle bind](https://fishshell.com/docs/current/cmds/bind.html)

## License

[BSD 2-Clause](https://choosealicense.com/licenses/bsd-2-clause/)
