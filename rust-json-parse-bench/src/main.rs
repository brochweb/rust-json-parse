use std::io::Read;

use anyhow::Result;
use clap::Parser;
use rust_json_parse::JsonDocument;

#[cfg(not(target_arch = "wasm32"))]
#[global_allocator]
static ALLOC: rsbmalloc::BinnedAlloc = rsbmalloc::BinnedAlloc::new();

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

#[cfg(test)]
mod tests {
    use std::thread;

    #[test]
    fn test_global_allocator() {
        const THREADS: usize = 8;
        const ITERATIONS: usize = 10000;

        let mut threads = Vec::with_capacity(THREADS);

        for i in 0..THREADS {
            threads.push(thread::spawn(move || {
                for _ in 0..ITERATIONS {
                    let vec = vec![i; 256];
                    for word in &vec {
                        assert_eq!(*word, i);
                    }
                    drop(vec);
                }
            }));
        }

        for thread in threads {
            thread.join().unwrap();
        }
    }
}
