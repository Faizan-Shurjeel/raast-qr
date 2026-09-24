use std::process::{Command, Output};

use raast_qr::{crc::compute_crc16, RaastQr};

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_raast-qr-cli"))
        .args(args)
        .output()
        .expect("CLI should launch")
}

fn multi_scheme_payload() -> String {
    let prefix = "00020101021126210005other01081234567828290008pk.raast0113+9233678658235204541153035865406100.005802PK5906Faizan6006Lahore6304";
    format!("{}{:04X}", prefix, compute_crc16(prefix.as_bytes()))
}

#[test]
fn generate_with_bank_code_round_trips() {
    let output = run(&[
        "generate",
        "--alias",
        "+923367865823",
        "--name",
        "Faizan",
        "--city",
        "Lahore",
        "--bank-code",
        "1234",
    ]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let payload = String::from_utf8(output.stdout).expect("payload is UTF-8");
    let qr = RaastQr::parse(payload.trim()).expect("generated payload should parse");
    assert_eq!(qr.bank_code, Some("1234"));
}

#[test]
fn generate_rejects_invalid_bank_code() {
    let output = run(&[
        "generate",
        "--alias",
        "+923367865823",
        "--name",
        "Faizan",
        "--city",
        "Lahore",
        "--bank-code",
        "",
    ]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Bank code"));
}

#[test]
fn decode_selects_requested_scheme_guid() {
    let payload = multi_scheme_payload();
    let output = run(&["decode", &payload, "--format", "json"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let default: serde_json::Value = serde_json::from_slice(&output.stdout).expect("valid JSON");
    assert_eq!(default["scheme_guid"], "pk.raast");
    assert_eq!(default["mai_tag"], "28");
    assert_eq!(default["raast_id"], "+923367865823");

    let output = run(&[
        "decode",
        &payload,
        "--scheme-guid",
        "other",
        "--format",
        "json",
    ]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let selected: serde_json::Value = serde_json::from_slice(&output.stdout).expect("valid JSON");
    assert_eq!(selected["scheme_guid"], "other");
    assert_eq!(selected["mai_tag"], "26");
    assert_eq!(selected["raast_id"], "12345678");
}

#[test]
fn verify_selects_requested_scheme_guid() {
    let payload = multi_scheme_payload();
    let output = run(&["verify", &payload, "--scheme-guid", "other"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("12345678"));

    let output = run(&["verify", &payload, "--scheme-guid", "missing"]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("no MAI template matches"));
}

#[test]
fn generate_and_decode_fee_modes() {
    for (args, expected) in [
        (vec!["--prompt-tip"], "PromptTip"),
        (vec!["--fixed-fee", "10.00"], "Fixed"),
        (vec!["--percentage-fee", "2.50"], "Percentage"),
    ] {
        let mut command = vec![
            "generate",
            "--alias",
            "+923367865823",
            "--name",
            "Faizan",
            "--city",
            "Lahore",
            "--dynamic",
            "--amount",
            "100.00",
        ];
        command.extend(args);
        let output = run(&command);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let payload = String::from_utf8(output.stdout).expect("payload is UTF-8");
        let output = run(&["decode", payload.trim(), "--format", "json"]);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let json: serde_json::Value = serde_json::from_slice(&output.stdout).expect("valid JSON");
        assert_eq!(json["amount"], "100.00");
        assert!(json["fee"].to_string().contains(expected), "{json}");

        let output = run(&["verify", payload.trim()]);
        assert!(output.status.success());
        assert!(String::from_utf8_lossy(&output.stdout).contains("Extra:"));
    }
}

#[test]
fn generate_rejects_invalid_fee_flags() {
    for args in [
        vec!["--prompt-tip", "--fixed-fee", "10"],
        vec!["--fixed-fee", "10", "--percentage-fee", "2"],
        vec!["--fixed-fee", "0"],
        vec!["--percentage-fee", "100"],
    ] {
        let mut command = vec![
            "generate",
            "--alias",
            "+923367865823",
            "--name",
            "Faizan",
            "--city",
            "Lahore",
        ];
        command.extend(args);
        assert!(!run(&command).status.success(), "accepted {command:?}");
    }
}
