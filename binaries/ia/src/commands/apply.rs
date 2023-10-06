use crate::prelude::*;

pub async fn apply(handle: GuyHandle, reset: bool, template: Option<&str>, dry_run: bool) -> IaResult<()> {
    let mut guy = handle.get_guy()?;

    if reset {
        guy.history.clear();
    }

    if let Some(template) = template {
        print_success!("Template applyed: {}", template);
        guy.load_template(GuyTemplate::from_yaml_file(&template)?)
            .await?;
    }

    if dry_run {
        println!("{}", serde_yaml::to_string(&guy)?)
    } else {
        handle.store_guy(guy)?;
        print_success!("Guy upserted");
    }
    Ok(())
}