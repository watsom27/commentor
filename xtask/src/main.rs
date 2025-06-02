use clap::{Parser, Subcommand};
use xshell::{cmd, Shell};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// Command to run
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Hook {
        #[clap(subcommand)]
        subcommand: Hook,
    },
}

#[derive(Debug, Subcommand)]
enum Hook {
    /// Git pre-commit hook
    PreCommit,

    /// Git pre-push hook
    PrePush,
}

fn main() -> anyhow::Result<()> {
    let args: Cli = Cli::parse();
    let sh = Shell::new()?;

    match args.command {
        Command::Hook { subcommand } => match subcommand {
            Hook::PreCommit => {
                cmd!(sh, "cargo +nightly fmt --check").run()?;
            },
            Hook::PrePush => {
                cmd!(sh, "cargo check").run()?;

                sh.set_var("RUSTDOCFLAGS", "-D warnings");
                cmd!(sh, "cargo doc").run()?;
            },
        },
    };

    Ok(())
}
