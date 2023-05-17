use std::io::Write;
use std::path::Path;
use std::pin::pin;

use anyhow::Context;
use futures::StreamExt;
use tokio_openai::ChatRequest;
use users::{get_current_uid, get_user_by_uid};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let input = std::env::args().skip(1).collect::<Vec<_>>().join(" ");

    let user = get_user_by_uid(get_current_uid()).context("no user with current UID")?;
    let user = user.name().to_str().unwrap();

    let current_working_directory = std::env::current_dir()?;

    let files = direct_children(&current_working_directory)?.join(" ");

    let email = tokio::process::Command::new("git")
        .arg("config")
        .arg("user.email")
        .output()
        .await?;

    let email = String::from_utf8(email.stdout)?;

    let os = std::env::consts::OS;

    let sys = format!("Output idiomatic single-line bash command to accomplish task.\nOS: {os}\nUsername: {user}\nEmail {email}\nFiles: {files}");

    let request = ChatRequest::new().sys_msg(sys).user_msg(input);

    let client = tokio_openai::Client::simple()?;

    let result = client.stream_chat(request).await?;

    let mut result = pin!(result);

    while let Some(res) = result.next().await {
        let res = res?;
        print!("{res}");
        std::io::stdout().flush()?;
    }

    println!();

    Ok(())
}

/// get list of relative paths to all direct children (files and dirs) in directory
fn direct_children(dir: &Path) -> anyhow::Result<Vec<String>> {
    let mut children = vec![];
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let path = path.strip_prefix(dir)?;
        let path = path.to_str().unwrap();
        children.push(path.to_string());
    }
    Ok(children)
}
