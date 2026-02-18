use std::{env, fs, io};

use payments_engine::models::Ledger;

mod adapters;
mod csv;

fn main() {
    let args: Vec<String> = env::args().collect();

    let transactions_file_path = args
        .get(1)
        .expect("Expected a path to a file containing transactions as the first argument");

    let mut ledger = Ledger::new();

    let file = fs::File::open(transactions_file_path).expect("Failed to open transactions file");
    for result in csv::read(file) {
        let row = result.expect("Failed to deserialize CSV row");

        let result = ledger.process_transaction(
            row.try_into()
                .expect("Failed to convert CSV row to transaction"),
        );

        if let Err(err) = result {
            eprintln!("Error processing transaction: {}", err);
        }
    }

    let stdout = io::stdout();
    let stdout_handle = stdout.lock();
    let result = csv::write(stdout_handle, ledger.accounts());

    if let Err(err) = result {
        eprintln!("Error writing accounts to stdout: {}", err);
    }
}
