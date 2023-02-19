use std::io::Read;

use anyhow::Result;
use clap::Parser;
use rs_json::parse;

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

fn main() -> Result<()> {
    let args = Args::parse();
    let json = if &args.file == "-" {
        let mut buf = Vec::new();
        std::io::stdin().read_to_end(&mut buf)?;
        buf
    } else {
        std::fs::read(args.file)?
    };
    if args.serde {
        let _ = serde_json::from_slice::<serde_json::Value>(&json)?;
    } else {
        let _ = parse(&json)?;
    }
    Ok(())
}
