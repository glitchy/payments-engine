use std::env;
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};

use crate::{
    engine::PaymentsEngine, 
    error::Result
};

mod account;
mod engine;
mod error;
mod transaction;

fn main() -> Result<()> {
    let mut engine = PaymentsEngine::new();

    let fpath = env::args().nth(1).expect("Usage: cargo run -- {file_path}");
    let file = File::open(fpath)?;
    let reader = BufReader::new(file);
    let mut rdr = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_reader(reader);

    for result in rdr.deserialize() {
        // make sure csv row is a valid transaciton, ignore if not
        match result {
            Ok(tx) => {
                // if processing fails, log error to stderr and continue processing txs
                if let Err(e) = engine.process_tx(&tx) {
                    eprintln!("failed transaction: {}", e);
                    continue;
                }
            }
            Err(e) => {
                eprintln!("skipping invalid transaction row: {}", e);
                continue;
            }
        }
    }

    let mut stdout = BufWriter::new(std::io::stdout());

    // write the account balances/state to stdout in csv format
    writeln!(stdout, "client,available,held,total,locked")?;
    for (id, account) in &engine.accounts {
        writeln!(
            stdout,
            "{},{:.4},{:.4},{:.4},{}",
            id, account.available, account.held, account.total, account.locked
        )?;
    }

    Ok(())
}
