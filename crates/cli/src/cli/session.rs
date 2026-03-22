//! Session management CLI commands

use anyhow::Result;
use clap::{Args, Subcommand};

use localgpt_core::agent::{Session, get_sessions_dir_for_agent, list_sessions_for_agent};

const DEFAULT_AGENT_ID: &str = "main";

#[derive(Args)]
pub struct SessionArgs {
    #[command(subcommand)]
    pub command: SessionCommands,
}

#[derive(Subcommand)]
pub enum SessionCommands {
    /// List recent sessions
    List {
        /// Agent ID (default: "main")
        #[arg(long, default_value = DEFAULT_AGENT_ID)]
        agent: String,
    },
    /// Create a branch from an existing session
    Branch {
        /// Session ID to branch from
        from_id: String,
        /// Agent ID (default: "main")
        #[arg(long, default_value = DEFAULT_AGENT_ID)]
        agent: String,
    },
}

pub async fn run(args: SessionArgs) -> Result<()> {
    match args.command {
        SessionCommands::List { agent } => {
            let sessions = list_sessions_for_agent(&agent)?;
            if sessions.is_empty() {
                println!("No sessions found for agent '{}'.", agent);
            } else {
                println!("Sessions for agent '{}':", agent);
                for (i, info) in sessions.iter().enumerate().take(20) {
                    println!("  {}. {} ({})", i + 1, info.id, info.created_at);
                }
                if sessions.len() > 20 {
                    println!("  ... and {} more", sessions.len() - 20);
                }
            }
        }
        SessionCommands::Branch { from_id, agent } => {
            let original = Session::load_for_agent(&from_id, &agent)?;
            let branched = original.branch();
            let new_id = branched.id().to_string();
            let msg_count = branched.message_count();

            let dir = get_sessions_dir_for_agent(&agent)?;
            std::fs::create_dir_all(&dir)?;
            let path = dir.join(format!("{}.jsonl", new_id));
            branched.save_to_path(&path)?;

            println!("Branched session created:");
            println!("  From: {}", from_id);
            println!("  New:  {}", new_id);
            println!("  Messages inherited: {}", msg_count);
        }
    }

    Ok(())
}
