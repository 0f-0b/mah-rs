use anyhow::bail;
use mah_core::adapter::Mah as _;
use mah_http_adapter::HttpAdapter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() != 2 {
        bail!("usage: {} <http-endpoint>", args[0]);
    }
    let endpoint = args[1].parse()?;
    let mah = HttpAdapter::new(endpoint, None);
    println!("Hello to mirai-api-http {}", mah.about().await?.version);
    Ok(())
}
