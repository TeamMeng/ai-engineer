use anyhow::Result;
use microsandbox::Sandbox;

#[tokio::main]
async fn main() -> Result<()> {
    let sb = Sandbox::list().await?;

    if sb.sandboxes.is_empty() {
        println!("Not found sandbox");
        return Ok(());
    }

    println!("There have {} sandboxes", sb.sandboxes.len());

    for handle in &sb.sandboxes {
        println!(
            "name: {}, status: {:?}",
            handle.name(),
            handle.status_snapshot()
        );
    }

    Ok(())
}
