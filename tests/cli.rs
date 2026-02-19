#[cfg(test)]
mod integration_tests {
    use assert_cmd::cargo::cargo_bin_cmd;
    use std::fs;

    fn fixture(path: &str) -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join(path)
    }

    fn assert_csv_eq_unordered(actual: &[u8], expected: &str) {
        let mut actual_reader = csv::Reader::from_reader(actual);
        let mut expected_reader = csv::Reader::from_reader(expected.as_bytes());

        let actual_headers = actual_reader.headers().unwrap().clone();
        let expected_headers = expected_reader.headers().unwrap().clone();
        assert_eq!(actual_headers, expected_headers);

        let mut actual_rows: Vec<Vec<String>> = actual_reader
            .records()
            .map(|r| r.unwrap().iter().map(|s| s.to_string()).collect())
            .collect();

        let mut expected_rows: Vec<Vec<String>> = expected_reader
            .records()
            .map(|r| r.unwrap().iter().map(|s| s.to_string()).collect())
            .collect();

        actual_rows.sort();
        expected_rows.sort();

        assert_eq!(actual_rows, expected_rows);
    }

    #[test]
    fn test_fixtures() {
        let cases = [
            ("test_0_transactions.csv", "test_0_accounts.csv"),
            (
                "test_1_locked_account_transactions.csv",
                "test_1_locked_account_accounts.csv",
            ),
            (
                "test_2_open_dispute_transactions.csv",
                "test_2_open_dispute_accounts.csv",
            ),
            (
                "test_3_double_chargeback_transactions.csv",
                "test_3_double_chargeback_accounts.csv",
            ),
            (
                "test_4_double_dispute_transactions.csv",
                "test_4_double_dispute_accounts.csv",
            ),
            (
                "test_5_double_resolve_transactions.csv",
                "test_5_double_resolve_accounts.csv",
            ),
            (
                "test_6_chargeback_resolved_transactions.csv",
                "test_6_chargeback_resolved_accounts.csv",
            ),
            (
                "test_7_resolve_chargedback_transactions.csv",
                "test_7_resolve_chargedback_accounts.csv",
            ),
            (
                "test_8_successful_resolution_transactions.csv",
                "test_8_successful_resolution_accounts.csv",
            ),
            (
                "test_9_mismatching_client_id_transactions.csv",
                "test_9_mismatching_client_id_accounts.csv",
            ),
            (
                "test_10_malformed_client_id_transactions.csv",
                "test_10_malformed_client_id_accounts.csv",
            ),
            (
                "test_11_malformed_tx_id_transactions.csv",
                "test_11_malformed_tx_id_accounts.csv",
            ),
            (
                "test_12_malformed_amount_transactions.csv",
                "test_12_malformed_amount_accounts.csv",
            ),
            (
                "test_13_malformed_file_transactions.csv",
                "test_13_malformed_file_accounts.csv",
            ),
            (
                "test_14_missing_amount_transactions.csv",
                "test_14_missing_amount_accounts.csv",
            ),
            (
                "test_15_negative_amount_transactions.csv",
                "test_15_negative_amount_accounts.csv",
            ),
            (
                "test_16_duplicated_tx_id_transactions.csv",
                "test_16_duplicated_tx_id_accounts.csv",
            ),
            (
                "test_17_balance_too_low_for_dispute_transactions.csv",
                "test_17_balance_too_low_for_dispute_accounts.csv",
            ),
            (
                "test_18_unordered_ids_transactions.csv",
                "test_18_unordered_ids_accounts.csv",
            ),
            (
                "test_19_invalid_transaction_type_transactions.csv",
                "test_19_invalid_transaction_type_accounts.csv",
            ),
        ];

        for (transactions, accounts) in cases {
            let transactions_path = fixture(transactions);
            let expected_accounts = fs::read_to_string(fixture(accounts)).unwrap();

            let mut cmd = cargo_bin_cmd!(env!("CARGO_PKG_NAME"));

            let output = cmd
                .arg(transactions_path)
                .assert()
                .success()
                .get_output()
                .stdout
                .clone();

            let result =
                std::panic::catch_unwind(|| assert_csv_eq_unordered(&output, &expected_accounts));

            if let Err(_err) = result {
                panic!(
                    "Run failed for fixtures {:?} / {:?}",
                    transactions, accounts
                );
            }
        }
    }
}
