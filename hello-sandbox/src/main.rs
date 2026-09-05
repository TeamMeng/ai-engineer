use anyhow::Result;
use microsandbox::{Sandbox, Volume};

#[tokio::main]
async fn main() -> Result<()> {
    let vol = Volume::builder("task-store").create().await?;

    vol.fs()
        .write(
            "/input.json",
            br#"{"task": "summarize", "data": [1, 2, 3]}"#,
        )
        .await?;

    println!("已在宿主机向卷预置 input.json");

    let sb = Sandbox::builder("processor")
        .replace()
        .image("docker.m.daocloud.io/library/python:3.12-alpine")
        .volume("/workspace", |m| m.named("task-store"))
        .create()
        .await?;

    sb.exec(
        "python",
        [
            "-c",
            r#"
import json
data = json.load(open('/workspace/input.json'))
result = {"sum": sum(data['data'])}
json.dump(result, open('/workspace/output.json', 'w'))
"#,
        ],
    )
    .await?;

    sb.destroy().await?;

    let outopt_str = vol.fs().read_to_string("/output.json").await?;
    println!("宿主机免开机直读产物: {}", outopt_str);

    Volume::remove(vol.name()).await?;
    println!("卷已清理完毕");

    Ok(())
}
