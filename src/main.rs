use std::io::Read;

use anyhow::Result;
use clap::Parser;
use rust_json_parse::JsonDocument;

#[global_allocator]
static ALLOC: snmalloc_rs::SnMalloc = snmalloc_rs::SnMalloc;

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

fn main() -> Result<()> {
    let args = Args::parse();
    let mut json = if &args.file == "-" {
        let mut buf = Vec::new();
        std::io::stdin().read_to_end(&mut buf)?;
        buf
    } else {
        std::fs::read(args.file)?
    };
    if args.serde {
        let _ = serde_json::from_slice::<serde_json::Value>(&json)?;
    } else if args.simd {
        let _ = simd_json::from_slice::<serde_json::Value>(&mut json)?;
    } else {
        let mut doc = JsonDocument::init();
        doc.parse_slice(&json)?;
    }
    Ok(())
}
