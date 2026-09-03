use anyhow::Result;
use microsandbox::Sandbox;

#[tokio::main]
async fn main() -> Result<()> {
    let sb = Sandbox::builder("Hello")
        .replace()
        .image("docker.m.daocloud.io/library/python:3.12-alpine")
        .memory(512)
        .create()
        .await?;

    println!("Sandbox already begin....");

    let output = sb
        .exec("python", ["-c", "print('你好，来自 microVM!')"])
        .await?;

    println!("Output: {}", output.stdout()?);

    println!("Exit code: {}", output.status().code);

    sb.destroy().await?;

    print!("Sandbox Stop");

    Ok(())
}
