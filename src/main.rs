mod mods;
use mods::download_from_github;

#[tokio::main]
async fn main() {
    download_from_github().await.unwrap();
}
