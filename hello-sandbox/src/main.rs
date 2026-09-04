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

    sb.exec(
        "python",
        ["-c", "import time; time.sleep(3); print('批处理完成')"],
    )
    .await?;

    sb.request_drain().await?;

    let result = sb.wait_until_stopped().await?;

    println!("Sandbox is stopping and status: {:?}", result);

    Ok(())
}
