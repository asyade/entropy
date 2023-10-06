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

    for message in guy.history.iter() {
        print_message(&message);
    }

    'rd: loop {
        if let Some((message, role)) = request.take() {
            guy.push_message(message, role);
            let response = guy.completion(&mut connector).await?;
            print_message(&response.choices[0].message);
            if let Err(e) = handle.store_guy(guy.clone()) {
                print_error!("Failed to persist guy's changes: {:?}", e)
            }
        }
        let input = Text::new("");
        let Ok(input) = input.prompt() else {
            return Ok(());
        };
        match input.as_str() {
            n if n.starts_with("\\") => match n {
                "\\exit" => {
                    break 'rd;
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
    Ok(())
}

fn print_message(message: &ChatCompletionMessage) {
    termimad::print_text(&message.content);
    termimad::print_text(&format!("(**{:?}**)\n\n", message.role));
}