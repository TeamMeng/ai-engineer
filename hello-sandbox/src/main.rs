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

    println!("Output: {}", output.stdout()?);

    sb.stop().await?;
    println!("Sandbox stop");

    println!("Second day");
    let sb = Sandbox::start("Hello").await?;

    let output = sb
        .exec("python", ["-c", "print('第二天数据处理完成')"])
        .await?;

    println!("Output: {}", output.stdout()?);
    println!("Task finish");

    sb.stop().await?;
    println!("Sandbox stop");

    Sandbox::remove("Hello").await?;
    println!("Sandbox remove");

    Ok(())
}
