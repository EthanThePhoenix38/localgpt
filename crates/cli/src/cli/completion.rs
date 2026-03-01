//! Shell completion generation for LocalGPT CLI
//!
//! Generates completion scripts for bash, zsh, fish, elvish, and powershell.
//!
//! Usage:
//!   localgpt completion bash > /etc/bash_completion.d/localgpt
//!   localgpt completion zsh > "${fpath[1]}/_localgpt"
//!   localgpt completion fish > ~/.config/fish/completions/localgpt.fish
//!   localgpt completion powershell > $PROFILE

use anyhow::Result;
use clap::{CommandFactory, ValueEnum};
use clap_complete::{generate, Shell};
use std::io;

#[derive(clap::Args, Debug)]
pub struct CompletionArgs {
    /// Shell type to generate completions for
    #[arg(value_enum)]
    pub shell: ShellArg,
}

/// Supported shells for completion generation
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ShellArg {
    /// Bourne Again SHell (bash)
    Bash,
    /// Z SHell (zsh)
    Zsh,
    /// Friendly Interactive SHell (fish)
    Fish,
    /// Elvish shell
    Elvish,
    /// PowerShell
    Powershell,
}

impl From<ShellArg> for Shell {
    fn from(shell: ShellArg) -> Self {
        match shell {
            ShellArg::Bash => Shell::Bash,
            ShellArg::Zsh => Shell::Zsh,
            ShellArg::Fish => Shell::Fish,
            ShellArg::Elvish => Shell::Elvish,
            ShellArg::Powershell => Shell::PowerShell,
        }
    }
}

pub fn run(args: CompletionArgs) -> Result<()> {
    let mut cmd = crate::cli::Cli::command();
    let shell: Shell = args.shell.into();

    generate(
        shell,
        &mut cmd,
        "localgpt",
        &mut io::stdout(),
    );

    Ok(())
}
