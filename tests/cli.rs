use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;

fn fixture(path: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(path)
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_0_transactions_to_accounts_stdout_matches_expected_csv() {
        let transactions = fixture("test_0_transactions.csv");
        let accounts = fs::read_to_string(fixture("test_0_accounts.csv")).unwrap();

        let mut cmd = cargo_bin_cmd!(env!("CARGO_PKG_NAME"));

        cmd.arg(transactions).assert().success().stdout(accounts);
    }
}
