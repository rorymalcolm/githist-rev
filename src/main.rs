mod githist;

use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const GIT_COMMAND_HISTORY_FILE_PATH: &str = ".git_command_history";

#[derive(Debug, Parser)]
#[command(name = "git-history-wrapper", version = "0.1.0")]
struct GitHistoryWrapper {
    #[clap(subcommand)]
    command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    CommandHistoryInit,
    MutateActions,
    #[clap(external_subcommand)]
    Other(Vec<String>),
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = GitHistoryWrapper::parse();
    match args.command {
        Some(Commands::CommandHistoryInit) => {
            let conn = rusqlite::Connection::open(GIT_COMMAND_HISTORY_FILE_PATH)?;
            conn.execute(
                "CREATE TABLE IF NOT EXISTS git_command_history (
                id TEXT PRIMARY KEY,
                command TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            )",
                [],
            )?;
        }
        Some(Commands::MutateActions) => {
            let conn = rusqlite::Connection::open(GIT_COMMAND_HISTORY_FILE_PATH)?;
            let mut stmt = conn.prepare("SELECT * FROM git_command_history")?;
            let mut rows = stmt.query([])?;
            while let Some(row) = rows.next()? {
                let id: String = row.get(0)?;
                let command: String = row.get(1)?;
                if !command_is_mutate(&command) {
                    continue;
                }
                let created_at: String = row.get(2)?;
                println!("{} {} {}", id, command, created_at);
            }
        }
        Some(Commands::Other(args)) => {
            // here we've received a git command, we should forward it to git
            // and then save it to the database
            let command = args.join(" ");
            let output = std::process::Command::new("git")
                .args(args)
                .output()
                .expect("failed to execute process");
            let output = String::from_utf8(output.stdout).unwrap();
            println!("{}", output);
            let conn = rusqlite::Connection::open(GIT_COMMAND_HISTORY_FILE_PATH)?;
            add_command_history(&conn, &command)?;
        }
        None => {
            println!("No subcommand was used");
        }
    }
    Ok(())
}

fn add_command_history(
    conn: &rusqlite::Connection,
    command: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let command = GitCommandState::new(command);
    conn.execute(
        "INSERT INTO git_command_history (id, command, created_at) VALUES (?1, ?2, ?3)",
        [
            Uuid::new_v4().to_string(),
            serde_json::to_string(&command)?,
            time::OffsetDateTime::now_utc().to_string(),
        ],
    )?;
    Ok(())
}

fn command_is_mutate(command: &str) -> bool {
    match command {
        "add" => true,
        "apply" => true,
        "bisect" => true,
        "branch" => true,
        "checkout" => true,
        "cherry-pick" => true,
        "clean" => true,
        "clone" => true,
        "commit" => true,
        "fetch" => true,
        "filter-branch" => true,
        "fsck" => true,
        "gc" => true,
        "init" => true,
        "merge" => true,
        "mv" => true,
        "pull" => true,
        "push" => true,
        "rebase" => true,
        "remote" => true,
        "reset" => true,
        "restore" => true,
        "rm" => true,
        "stash" => true,
        "submodule" => true,
        "switch" => true,
        "tag" => true,
        "update-index" => true,
        "update-ref" => true,
        "write-tree" => true,
        _ => false,
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum GitCommand {
    Add,
    Apply,
    Bisect,
    Branch,
    Checkout,
    CherryPick,
    Clean,
    Clone,
    Commit,
    Fetch,
    FilterBranch,
    Fsck,
    Gc,
    Init,
    Merge,
    Mv,
    Pull,
    Push,
    Rebase,
    Remote,
    Reset,
    Restore,
    Rm,
    Stash,
    Submodule,
    Switch,
    Tag,
    UpdateIndex,
    UpdateRef,
    WriteTree,
    InvalidCommand,
}

#[derive(Serialize, Deserialize)]
struct GitCommandState {
    command: GitCommand,
    files_affected: Vec<String>,
    current_branch: String,
    current_commit: String,
}

fn get_current_commit() -> String {
    let output = std::process::Command::new("git")
        .args(&["rev-parse", "HEAD"])
        .output()
        .expect("failed to execute process");
    String::from_utf8(output.stdout).unwrap()
}

fn get_current_branch() -> String {
    let output = std::process::Command::new("git")
        .args(&["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .expect("failed to execute process");
    String::from_utf8(output.stdout).unwrap()
}

impl GitCommandState {
    fn extract_git_command(command: &str) -> Result<GitCommand, Box<dyn std::error::Error>> {
        return match command.split(" ").next().unwrap_or_else(|| "") {
            "add" => Ok(GitCommand::Add),
            "apply" => Ok(GitCommand::Apply),
            "bisect" => Ok(GitCommand::Bisect),
            "branch" => Ok(GitCommand::Branch),
            "checkout" => Ok(GitCommand::Checkout),
            "cherry-pick" => Ok(GitCommand::CherryPick),
            "clean" => Ok(GitCommand::Clean),
            "clone" => Ok(GitCommand::Clone),
            "commit" => Ok(GitCommand::Commit),
            "fetch" => Ok(GitCommand::Fetch),
            "filter-branch" => Ok(GitCommand::FilterBranch),
            "fsck" => Ok(GitCommand::Fsck),
            "gc" => Ok(GitCommand::Gc),
            "init" => Ok(GitCommand::Init),
            "merge" => Ok(GitCommand::Merge),
            "mv" => Ok(GitCommand::Mv),
            "pull" => Ok(GitCommand::Pull),
            "push" => Ok(GitCommand::Push),
            "rebase" => Ok(GitCommand::Rebase),
            "remote" => Ok(GitCommand::Remote),
            "reset" => Ok(GitCommand::Reset),
            "restore" => Ok(GitCommand::Restore),
            "rm" => Ok(GitCommand::Rm),
            "stash" => Ok(GitCommand::Stash),
            "submodule" => Ok(GitCommand::Submodule),
            "switch" => Ok(GitCommand::Switch),
            "tag" => Ok(GitCommand::Tag),
            "update-index" => Ok(GitCommand::UpdateIndex),
            "update-ref" => Ok(GitCommand::UpdateRef),
            "write-tree" => Ok(GitCommand::WriteTree),
            _ => Err("No command found".into()),
        };
    }

    // This is really quite a naive implementation, but it should work for now.
    fn process_affected_files(command: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let mut files_affected = vec![];
        for string in command.split(" ") {
            if std::path::Path::new(string).exists() {
                files_affected.push(string.to_string());
            }
        }
        Ok(files_affected)
    }

    fn new(command: &str) -> GitCommandState {
        let git_command = { GitCommandState::extract_git_command(command) }
            .unwrap_or_else(|_| GitCommand::InvalidCommand);
        GitCommandState {
            command: git_command,
            files_affected: GitCommandState::process_affected_files(command)
                .unwrap_or_else(|_| vec![]),
            current_branch: get_current_branch(),
            current_commit: get_current_commit(),
        }
    }
}
