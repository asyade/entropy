use crate::prelude::*;

pub async fn ask(
    handle: GuyHandle,
    role: ChatCompletionRole,
    message: Option<String>,
    interactive: bool,
    completion: bool,
) -> IaResult<()> {
    let keychain = KeyChain::from_env();
    let connector = OpenAIConnector::new(&keychain);

    if interactive {
        ask_interactive(connector, handle, role, message).await
    } else {
        ask_non_interactive(connector, handle, role, message, completion).await
    }
}

async fn ask_non_interactive(
    mut connector: OpenAIConnector,
    handle: GuyHandle,
    role: ChatCompletionRole,
    message: Option<String>,
    completion: bool,
) -> IaResult<()> {
    let mut guy = handle.get_guy()?;
    if let Some(message) = message {
        guy.push_message(message, role);
    }
    if completion && guy.history.len() > 0 {
        let response = guy.completion(&mut connector).await?;
        println!("{}", response.choices[0].message.content);
    } else {
        print_warning!("Nothing to complete")
    }
    let _ = handle.store_guy(guy.clone());
    if let Err(e) = handle.store_guy(guy.clone()) {
        print_error!("Failed to persist guy's changes: {:?}", e)
    }
    Ok(())
}

async fn ask_interactive(
    mut connector: OpenAIConnector,
    handle: GuyHandle,
    role: ChatCompletionRole,
    message: Option<String>,
) -> IaResult<()> {
    let mut guy = handle.get_guy()?;
    let mut request: Option<(String, ChatCompletionRole)> = if let Some(message) = message {
        Some((message, role))
    } else {
        None
    };

    for (idx, message) in guy.history.iter().enumerate() {
        print_message(&message, idx);
    }

    loop {
        if let Some((message, role)) = request.take() {
            guy.push_message(message, role);
            let response: ChatCompletionResponse = guy.completion(&mut connector).await?;
            print_message(&response.choices[0].message, guy.history.len() - 1);
            if let Err(e) = handle.store_guy(guy.clone()) {
                print_error!("Failed to persist guy's changes: {:?}", e)
            }
        }
        
        let input = Text::new("");
        let Ok(input) = input.prompt() else {
            return Ok(());
        };
        // let input = Text::new("");
        // let Ok(input) = input.prompt() else {
        //     return Ok(());
        // };
        match input.as_str() {
            n if n.starts_with("\\") => {
                let mut args = n.split(" ");
                match args.next() {
                    Some("\\help") => {
                        println!("
\\help - print this help
\\save \\s - save changes
\\exit \\e - exit
\\completion \\c - ask for completion
\\history \\h - print history
\\rm [<index>..] - remove message at index
                        ");
                    }
                    Some("\\save") | Some("\\s") => {
                        if let Err(e) = handle.store_guy(guy.clone()) {
                            print_error!("Failed to persist guy's changes: {:?}", e)
                        } else {
                            print_success!("Guy cahnges saved")
                        }            
                    }
                    Some("\\exit") | Some("\\e") => {
                        // TODO: ask for save
                        return Ok(());
                    }
                    Some("\\completion") | Some("\\c") => {
                        let response: ChatCompletionResponse = guy.completion(&mut connector).await?;
                        print_message(&response.choices[0].message, guy.history.len() - 1);
                    }
                    Some("\\history") | Some("\\h") => {
                        for (idx, message) in guy.history.iter().enumerate() {
                            print_message(&message, idx);
                        }
                    }
                    Some("\\rm") => {
                        while let Some(idx) = args.next().and_then(|e| e.parse::<usize>().ok()) {
                            if idx < guy.history.len() {
                                guy.history.remove(idx);
                                print_success!("Message removed: {}", idx);
                            } else {
                                print_error!("Index out of bounds: {}", idx);
                            }
                        }
                    }
                    _ => print_error!("Unknown command: {}", n),
                }
            }
            _ => {
                request = Some((input.to_string(), ChatCompletionRole::User));
            }
        }
    }
}

use colored::Colorize;

fn print_message(message: &ChatCompletionMessage, index: usize) {
    let role_colored = match message.role {
        ChatCompletionRole::User => format!("{:?}", message.role).green(),
        ChatCompletionRole::System => format!("{:?}", message.role).magenta(),
        ChatCompletionRole::Assistant => format!("{:?}", message.role).blue(),
        ChatCompletionRole::Function => format!("{:?}", message.role).yellow(),
    };
    println!(
        "{} ({})",
        role_colored,
        index,
    );
    termimad::print_text(&message.content);
    print!("\n");    
}
