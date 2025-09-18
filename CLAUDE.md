# ut

`ut` is a CLI based utility toolkit of most commonly used tools by software developers and IT professionals.
It includes tools like hashing utilities, secret generators, file server etc.

## Architecture

- Each tool in `ut` implements the `crate::tool::Tool` trait.
  - The trait requires you to define a `clap::Command` (preferably using the derive syntax) that will get the input for the tool.
  - The execution entrypoint of the tool.
  - The tool optionally also returns a serde_json result.
- Each tool will be defined in its own file under `src/tools/`. Similar kind of tools must be grouped though.
- The tool will be accessible as a subcommand on the `ut` command.

## Commands

- The project uses standard `cargo` commands.
  - So, for example, `cargo run` to run the project.
  - And, `cargo fmt` for formatting the project.

## Guidelines

- Use stdlib wherever possible.
- Use popular cargo crates if necessary.
- Wherever possible, tools should return values instead of printing them.
