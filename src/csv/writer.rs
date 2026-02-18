use csv::WriterBuilder;
use std::io::Write;

use crate::csv::Account as CsvAccount;
use payments_engine::models::Account;

pub fn write<'a, W, A>(writer: W, accounts: A) -> csv::Result<()>
where
    W: Write,
    A: IntoIterator<Item = &'a Account>,
{
    let mut csv_writer = WriterBuilder::new().has_headers(true).from_writer(writer);

    for account in accounts {
        let csv_account = CsvAccount::from(account);
        csv_writer.serialize(csv_account)?;
    }

    csv_writer.flush()?;
    Ok(())
}
