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
        let row = result?;
        let transaction = row.try_into()?;
        let result = ledger.process_transaction(transaction);

        if let Err(err) = result {
            eprintln!("Error processing transaction: {}", err);
        }
    }

    let stdout = io::stdout();
    let stdout_handle = stdout.lock();
    csv::write(stdout_handle, ledger.accounts())?;

    Ok(())
}
