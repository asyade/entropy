#![feature(async_closure)]
use api_connector::{
    keyring::KeyChain,
    openai::{ChatCompletionRole, OpenAIConnector},
};
use clap::{Parser, Subcommand};
use guy::prelude::*;
use inquire::Text;
use tokio::{io::AsyncReadExt, process::Command};

use crate::prelude::*;

pub mod error;
pub mod prelude;
pub mod store;
pub mod utils;

macro_rules! print_error {
    ($($arg:tt)*) => {{
        eprintln!("❌ {}", format!( $($arg)* ));
    }}
}

macro_rules! print_warning {
    ($($arg:tt)*) => {{
        eprintln!("⚡ {}", format!( $($arg)* ));
    }}
}

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
        #[arg(short, long, default_value = "false", help = "Template file to apply")]
        template: Option<String>,
    },
    #[command(about = "Delete a guy from the store")]
    Delete {},
    #[command(about = "Edit a stord guy as yaml file")]
    Edit {
        #[arg(long, help = "The text editor to use")]
        editor: Option<String>,
    },
    #[command(about = "List all available guys in the store")]
    List {},
    #[command(about = "Show guy's data")]
    Get {
        #[arg(short, long, help = "The output format")]
        output: Option<GuyGetOutputFormat>,
    },
    #[command(about = "Perform a chat completion with a guy")]
    Ask {
        #[arg(short, long, default_value = "user", help = "The message's role")]
        //TODO: posible values
        role: String,
        #[arg(
            short,
            long,
            help = "The message to send to the guy, stdin if not provided"
        )]
        message: Option<String>,
        #[arg(short, long, default_value = "false", help = "Interactive mode")]
        interactive: bool,
    },
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, clap::ValueEnum)]
pub enum GuyGetOutputFormat {
    Yaml,
    Json,
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
                    let mut guy = handle.get_guy()?;

                    if *reset {
                        guy.history.clear();
                    }

                    if let Some(template) = template.clone() {
                        println!("Template applyed: {}", template);
                        guy.load_template(GuyTemplate::from_yaml_file(&template)?)
                            .await?;
                    }

                    if *dry_run {
                        println!("{}", serde_yaml::to_string(&guy)?)
                    } else {
                        handle.store_guy(guy)?;
                        print_success!("Guy `{}` upserted", name);
                    }
                }
                GuysCommands::Delete {} => {
                    // store.drop_tree(name)?;
                    // println!("Guy `{}` deleted", name);
                }
                GuysCommands::List {} => {
                    // for tree in store.tree_names().into_iter().map(|e| String::from_utf8(e.to_vec()).unwrap()).filter(|e| !e.starts_with("__")) {
                    // println!("{}", tree);
                    // }
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
                } => {
                    let role = serde_yaml::from_str(&format!("\"{}\"", role))?;

                    let keychain = KeyChain::from_env();
                    let mut connector = OpenAIConnector::new(&keychain);

                    let handle = store.get_guy_handle(&name).await?;
                    let mut guy = handle.get_guy()?;
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
                    dbg!(&message);
                    if *interactive {
                        let mut request: Option<(String, ChatCompletionRole)> =
                            if let Some(message) = message {
                                Some((message, role))
                            } else {
                                None
                            };
                        loop {
                            if let Some((message, role)) = request.take() {
                                guy.push_message(message, role);
                                let response = guy.completion(&mut connector).await?;
                                if let Err(e) = handle.store_guy(guy.clone()) {
                                    print_error!("Failed to persist guy's changes: {:?}", e)
                                }
                                termimad::print_text(&response.choices[0].message.content);
                            }
                            let input = Text::new("").prompt()?;
                            match input.as_str() {
                                n if n.starts_with("\\") => match n {
                                    "\\exit" => {
                                        break;
                                    }
                                    "\\last" => {
                                        let _ = guy
                                            .history
                                            .last()
                                            .map(|e| println!("{:?}: {}", e.role, e.content));
                                    }
                                    _ => {
                                        print_error!("Unknown command: {}", n);
                                    }
                                },
                                _ => {
                                    request = Some((input.to_string(), ChatCompletionRole::User));
                                }
                            }
                        }
                    } else {
                        if let Some(message) = message {
                            guy.push_message(message, role);
                        }
                        let response = guy.completion(&mut connector).await?;
                        let _ = handle.store_guy(guy.clone());
                        if let Err(e) = handle.store_guy(guy.clone()) {
                            print_error!("Failed to persist guy's changes: {:?}", e)
                        } else {
                            println!("{}", response.choices[0].message.content);
                        }
                    }
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
