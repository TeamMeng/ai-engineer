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

    let handle = Sandbox::get("Hello").await?;

    // 1. 查看沙箱名称
    println!("沙箱名称: {}", handle.name());
    // 2. 查看沙箱状态
    // - Handle 上提供同步快照方法：status_snapshot()
    println!("沙箱状态(快照): {:?}", handle.status_snapshot());
    // - live 沙箱对象上提供异步实时查询：sb.status().await?
    println!("沙箱状态(实时): {:?}", sb.status().await?);

    // 3. 查看内存配置
    // - handle.config() 返回 Result<SandboxConfig>，需加 `?` 解包
    // - memory_mib 位于 spec.resources 结构体中
    println!(
        "内存配置: {} MiB",
        handle.config()?.spec.resources.memory_mib
    );

    sb.destroy().await?;

    Ok(())
}
