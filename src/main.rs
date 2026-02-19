use anyhow::{Context, Result};
use std::{env, fs::File, io};

use payments_engine::models::Ledger;

mod adapters;
mod csv;

fn main() -> Result<()> {
    let mut args = env::args();

    let _program = args.next();
    let transactions_file_path = args
        .next()
        .context("Expected the path to a file containing transactions as the first argument")?;

    let mut ledger = Ledger::new();

    let file = File::open(transactions_file_path)?;
    for result in csv::read(file) {
        let parse_result = result;

        if let Err(err) = parse_result {
            eprintln!("Skipping malformed transaction: {}", err);
            continue;
        }

        let transaction_result = parse_result.unwrap().try_into();

        if let Err(err) = transaction_result {
            eprintln!("Skipping non convertible transaction: {}", err);
            continue;
        }

        let result = ledger.process_transaction(transaction_result.unwrap());

        if let Err(err) = result {
            eprintln!("Error processing transaction: {}", err);
        }
    }

    let stdout = io::stdout();
    let stdout_handle = stdout.lock();
    csv::write(stdout_handle, ledger.accounts())?;

    Ok(())
}
