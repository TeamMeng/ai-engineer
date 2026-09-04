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

    let output = sb
        .exec("python", ["-c", "print('第一天数据处理完成')"])
        .await?;
    println!("{}", output.stdout()?);

    sb.request_drain().await?;
    println!("request drain");

    println!("All Task finish...");

    Ok(())
}
