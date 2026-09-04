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
        .exec(
            "python",
            [
                "-c",
                r#"import sys
print('这是标准输出')
print('这是错误输出', file=sys.stderr)
sys.exit(0)"#,
            ],
        )
        .await?;

    let stdout_text = output.stdout()?;
    println!("stdout: {}", stdout_text);

    let stderr_text = output.stderr()?;
    println!("stderr: {}", stderr_text);

    let exit_code = output.status().code;
    let is_success = output.status().success;

    println!("exit code: {}, is success: {}", exit_code, is_success);

    sb.destroy().await?;

    Ok(())
}
