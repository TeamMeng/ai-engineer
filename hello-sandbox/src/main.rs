use anyhow::Result;
use microsandbox::Sandbox;

#[tokio::main]
async fn main() -> Result<()> {
    let sb = Sandbox::builder("Hello")
        .replace()
        .image("docker.m.daocloud.io/library/python:3.12-alpine")
        .create()
        .await?;

    sb.shell("dd if=/dev/urandom of=/tmp/result.bin bs=1024 count=4")
        .await?;

    let bytes = sb.fs().read("/tmp/result.bin").await?;
    println!("read {} bytes", bytes.len());

    sb.destroy().await?;

    Ok(())
}
