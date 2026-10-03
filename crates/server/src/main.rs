use std::{net::IpAddr, net::SocketAddr, path::PathBuf, sync::Arc};

use anyhow::Context;
use clap::Parser;
use web_note_core::Notes;

#[derive(Parser)]
#[command(version)]
struct Args {
    /// The directory to store notes in.
    #[arg(long, env = "NOTES_DIR", default_value = "Notes")]
    notes_dir: PathBuf,
    #[arg(long, env = "BIND", default_value = "127.0.0.1")]
    bind: IpAddr,
    #[arg(long, env = "PORT", default_value_t = 5393)]
    port: u16,
    /// The maximum size of an asset in kibibyte (1024 bytes).
    #[arg(long, env = "MAX_ASSET_KIB", default_value_t = 10 * 1024)]
    max_asset_kib: u64,
    /// Print the OpenAPI spec and exit.
    #[arg(long)]
    openapi: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    if args.openapi {
        println!("{}", web_note_server::openapi_json());
        return Ok(());
    }

    let notes = Notes::open(&args.notes_dir, args.max_asset_kib * 1024)
        .context("failed to open notes directory")?;
    let notes = Arc::new(notes);

    #[cfg(feature = "swagger")]
    let app = web_note_server::app_with_swagger(notes);
    #[cfg(not(feature = "swagger"))]
    let app = web_note_server::app(notes);

    let addr = SocketAddr::new(args.bind, args.port);
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("failed to bind {addr}"))?;
    println!("listening on {addr}");
    axum::serve(listener, app).await?;
    Ok(())
}
