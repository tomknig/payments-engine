use std::io::Read;

use super::dto::Transaction;

pub fn read<R: Read>(reader: R) -> impl Iterator<Item = Result<Transaction, csv::Error>> {
    let csv_reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .trim(csv::Trim::All)
        .from_reader(reader);

    csv_reader.into_deserialize::<Transaction>()
}
