//! Parse-throughput benchmark (zero-dep, deliberately simple).
//! Run with: cargo bench -p dotcli-parser
//!
//! Methodology: the shipped examples are concatenated into a ~450 KB corpus
//! and parsed repeatedly; we report time per corpus pass, per statement, and
//! MB/s. Numbers are machine-dependent — compare trends, not absolutes.

use std::time::Instant;

fn main() {
    let publish = include_str!("../../../examples/publish.cli");
    let cleanup = include_str!("../../../examples/cleanup.cli");

    let mut corpus = String::new();
    for _ in 0..500 {
        corpus.push_str(publish);
        corpus.push_str(cleanup);
    }
    let bytes = corpus.len();

    for _ in 0..10 {
        dotcli_parser::parse(&corpus).expect("corpus must parse");
    }

    let iters = 100u32;
    let start = Instant::now();
    let mut statements = 0;
    for _ in 0..iters {
        statements = dotcli_parser::parse(&corpus)
            .expect("corpus must parse")
            .len();
    }
    let elapsed = start.elapsed();

    let per_pass = elapsed / iters;
    let per_stmt = elapsed.as_nanos() as f64 / (iters as f64 * statements as f64);
    let mbps = (bytes as f64 * iters as f64) / elapsed.as_secs_f64() / 1e6;

    println!("corpus: {} KB, {statements} statements", bytes / 1024);
    println!("parse:  {per_pass:?}/pass  |  {per_stmt:.0} ns/statement  |  {mbps:.0} MB/s");
}
