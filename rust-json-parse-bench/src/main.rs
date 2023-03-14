use std::io::Read;

use anyhow::Result;
use clap::Parser;
use rust_json_parse::JsonDocument;

#[cfg(not(target_arch = "wasm32"))]
#[global_allocator]
static ALLOC: snmalloc_rs::SnMalloc = snmalloc_rs::SnMalloc;

#[cfg(any(target_feature = "sse4.2", target_feature = "neon"))]
#[derive(clap::Parser)]
struct Args {
    #[arg(short, long)]
    print: bool,
    /// Set to `-` for stdin
    #[arg(default_value = "-")]
    file: String,
    #[arg(short, long)]
    serde: bool,
    #[arg(long)]
    simd: bool,
}

#[cfg(not(any(target_feature = "sse4.2", target_feature = "neon")))]
#[derive(clap::Parser)]
struct Args {
    #[arg(short, long)]
    print: bool,
    /// Set to `-` for stdin
    #[arg(default_value = "-")]
    file: String,
    #[arg(short, long)]
    serde: bool,
}

#[allow(unused_mut)]
fn main() -> Result<()> {
    let args = Args::parse();
    let mut json = if &args.file == "-" {
        let mut buf = Vec::new();
        std::io::stdin().read_to_end(&mut buf)?;
        buf
    } else {
        std::fs::read(args.file)?
    };
    #[cfg(any(target_feature = "sse4.2", target_feature = "neon"))]
    if args.simd {
        let _ = simd_json::from_slice::<serde_json::Value>(&mut json)?;
        return Ok(());
    }
    if args.serde {
        let _ = serde_json::from_slice::<serde_json::Value>(&json)?;
    } else {
        let _ = JsonDocument::parse_create(&json)?;
    }
    Ok(())
}
