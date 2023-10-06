#![feature(async_closure)]
use clap::{Parser, Subcommand};
use guy::prelude::*;
use tokio::{io::AsyncReadExt, process::Command};

use crate::prelude::*;

pub mod commands;
pub mod error;
pub mod prelude;
pub mod store;
pub mod utils;

#[macro_export]
macro_rules! print_error {
    ($($arg:tt)*) => {{
        eprintln!("❌ {}", format!( $($arg)* ));
    }}
}

#[macro_export]
macro_rules! print_warning {
    ($($arg:tt)*) => {{
        eprintln!("⚡ {}", format!( $($arg)* ));
    }}
}

#[macro_export]
macro_rules! print_success {
    ($($arg:tt)*) => {{
        eprintln!("✅ {}", format!( $($arg)* ));
    }}
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[arg(short, long, help = "The store's path", default_value = ".iarc")]
    store: PathBuf,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Initialize a new store")]
    Init {},
    #[command(about = "Manage guys")]
    Guy {
        #[arg(short, long)]
        name: Option<String>,
        #[command(subcommand)]
        command: GuysCommands,
    },
}

#[derive(Subcommand)]
pub enum GuysCommands {
    #[command(about = "Create or load a guy from the store and apply to it if provided")]
    Apply {
        #[arg(
            short,
            long,
            default_value = "false",
            help = "Reset the guy's history first"
        )]
        reset: bool,
        #[arg(
            short,
            long,
            default_value = "false",
            help = "Show preview of the updated guy (does not persist changes)"
        )]
        dry_run: bool,
        template: Option<String>,
    },
    #[command(about = "Delete a guy from the store")]
    Delete {},
    #[command(about = "Edit a stord guy")]
    Edit {
        #[arg(long, help = "The text editor to use")]
        editor: Option<String>,
    },
    #[command(about = "List all available guys in the store")]
    List {},
    #[command(about = "Export guy's data")]
    Get {
        #[arg(short, long, help = "The output format")]
        output: Option<GuyGetOutputFormat>,
    },
    #[command(about = "Perform a chat completion with a guy`")]
    Ask {
        #[arg(
            short,
            long,
            default_value = "user",
            long_help = "The `role` of the message to be appended to the guy's history."
        )]
        //TODO: posible values
        role: GuyAskRole,
        #[arg(
            short,
            long,
            long_help = "The message to be appended to the guy's history.\nFilled with stdin (if any data avail) when not provided.",
            help = "The message to be appended to history"
        )]
        message: Option<String>,
        #[arg(
            short,
            long,
            default_value = "false",
            long_help = "Run in interactive mode.",
            help = "Interactive mode",
            conflicts_with = "completion"
        )]
        interactive: bool,
        #[arg(
            short,
            long,
            default_value = "true",
            help = "Perform a completion after consuming the provided message if any",
            conflicts_with = "interactive"
        )]
        completion: bool,
    },
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, clap::ValueEnum)]
pub enum GuyGetOutputFormat {
    Yaml,
    Json,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, clap::ValueEnum)]
pub enum GuyAskRole {
    System,
    User,
    Assistant,
    Function,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    dotenv::dotenv().ok();

    if let Err(e) = hanlde_command(cli).await {
        print_error!("{:?}", e);
    }
}

async fn hanlde_command(cli: Cli) -> Result<(), anyhow::Error> {
    let store_directory = cli.store;

    match &cli.command {
        Commands::Init {} => {
            if store_directory.exists() {
                return Err(IaError::Message(format!(
                    "The file already exists: `{:?}`",
                    store_directory
                )))?;
            }
            let _store = Store::create(&store_directory)?;
            println!("Store created in {:?}", store_directory);
        }
        Commands::Guy { name, command } => {
            let store = Store::open(&store_directory)?;
            let name = name.clone().unwrap_or_else(|| {
                std::env::var("DEFAULT_GUY").unwrap_or_else(|_| "guy".to_string())
            });
            match command {
                GuysCommands::Apply {
                    template,
                    reset,
                    dry_run,
                } => {
                    let handle = store.get_guy_handle(&name).await?;
                    commands::apply::apply(handle, *reset, template.as_deref(), *dry_run).await?;
                }
                GuysCommands::Delete {} => {
                    store.delete_guy(&name).await?;
                    println!("Guy `{}` deleted", name);
                }
                GuysCommands::List {} => {
                }
                GuysCommands::Edit { editor } => {
                    let handle = store.get_guy_handle(&name).await?;
                    let guy = handle.get_guy()?;
                    let serialized = serde_yaml::to_string(&guy)?;

                    let mut edited = ask_for_editing(serialized.clone(), editor.clone()).await?;
                    let mut edited_guy: Result<Guy, _> = serde_yaml::from_str(&edited);
                    while edited_guy.is_err() {
                        print_error!(
                            "Invalid yaml: {:?}",
                            edited_guy.as_ref().err().as_ref().unwrap()
                        );
                        if inquire::Confirm::new("Do you want to edit the file again ?").prompt()? {
                            edited = ask_for_editing(edited.clone(), editor.clone()).await?;
                            edited_guy = serde_yaml::from_str(&edited);
                        } else {
                            print_error!("Aborting");
                            return Ok(());
                        }
                    }

                    let edited_guy = edited_guy.unwrap();

                    if edited_guy == guy {
                        print_warning!("No changes detected");
                    } else {
                        handle.store_guy(edited_guy)?;
                        print_success!("Guy `{}` updated", name);
                    }
                }
                GuysCommands::Get { output } => {
                    let handle = store.get_guy_handle(&name).await?;
                    let guy = handle.get_guy()?;
                    match output {
                        None => {}
                        Some(GuyGetOutputFormat::Yaml) => {
                            println!("{}", serde_yaml::to_string(&guy)?);
                        }
                        Some(GuyGetOutputFormat::Json) => {
                            println!("{}", serde_json::to_string_pretty(&guy)?);
                        }
                    }
                }
                GuysCommands::Ask {
                    role,
                    message,
                    interactive,
                    completion,
                } => {
                    let handle = store.get_guy_handle(&name).await?;
                    let message = if let Some(message) = message {
                        Some(message.clone())
                    } else if !atty::is(atty::Stream::Stdin) {
                        let mut buffer = String::new();
                        if let Err(_e) = tokio::io::stdin().read_to_string(&mut buffer).await {
                            print_error!("Failed to read stdin: {:?}", _e);
                            None
                        } else {
                            Some(buffer)
                        }
                    } else {
                        None
                    };
                    commands::ask::ask(handle, (*role).into(), message, *interactive, *completion)
                        .await?;
                }
            }
        }
    }
    Ok(())
}

async fn ask_for_editing(init: String, editor: Option<String>) -> Result<String, anyhow::Error> {
    let tmp_dir = TempDir::new()?;
    let tmp_path = tmp_dir.path().join("guy.yaml");
    tokio::fs::write(&tmp_path, init.as_bytes()).await?;

    Command::new(editor.unwrap_or_else(|| "vim".to_string()))
        .arg(&tmp_path)
        .status()
        .await?;

    let mut edited_content = String::new();
    tokio::fs::File::open(tmp_path)
        .await?
        .read_to_string(&mut edited_content)
        .await?;

    let _ = tmp_dir.close();
    Ok(edited_content)
}

impl Into<ChatCompletionRole> for GuyAskRole {
    fn into(self) -> ChatCompletionRole {
        match self {
            Self::System => ChatCompletionRole::System,
            Self::User => ChatCompletionRole::User,
            Self::Assistant => ChatCompletionRole::Assistant,
            Self::Function => ChatCompletionRole::Function,
        }
    }
}
