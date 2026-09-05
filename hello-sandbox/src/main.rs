use anyhow::Result;
use microsandbox::Sandbox;

#[tokio::main]
async fn main() -> Result<()> {
    let volume_name = "pipeline-cache";

    println!("=== 启动沙箱 A：生成数据 ===");
    let sb_a = Sandbox::builder("worker-a")
        .replace()
        .image("docker.m.daocloud.io/library/python:3.12-alpine")
        .volume("/data", |m| {
            m.named_with(volume_name, |v| v.ensure_exists())
        })
        .create()
        .await?;

    sb_a.exec(
        "python",
        [
            "-c",
            "open('/data/result.txt', 'w').write('这是沙箱A的计算结果')",
        ],
    )
    .await?;

    sb_a.destroy().await?;
    println!("=== 启动沙箱 B：生成数据 ===");
    let sb_b = Sandbox::builder("worker-b")
        .replace()
        .image("docker.m.daocloud.io/library/python:3.12-alpine")
        .volume("/input", |m| m.named(volume_name))
        .create()
        .await?;

    let output = sb_b
        .exec(
            "python",
            [
                "-c",
                "print('沙箱B读到:', open('/input/result.txt').read())",
            ],
        )
        .await?;

    println!("{}", output.stdout()?);
    sb_b.destroy().await?;

    Ok(())
}
