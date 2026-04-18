// Integration tests for TCPing Rust implementation

use assert_cmd::Command;
use predicates::prelude::*;
use std::process;

#[test]
fn test_cli_help() {
    let mut cmd = Command::cargo_bin("tcping").unwrap();
    cmd.arg("--help");
    cmd.assert().success().stdout(predicate::str::contains("TCP connectivity testing tool"));
}

#[test]
fn test_cli_version() {
    let mut cmd = Command::cargo_bin("tcping").unwrap();
    cmd.arg("--version");
    cmd.assert().success().stdout(predicate::str::contains("tcping version 0.1.0"));
}

#[test]
fn test_invalid_host() {
    let mut cmd = Command::cargo_bin("tcping").unwrap();
    cmd.arg("").arg("80");
    cmd.assert().failure().stderr(predicate::str::contains("Host cannot be empty"));
}

#[test]
fn test_invalid_port() {
    let mut cmd = Command::cargo_bin("tcping").unwrap();
    cmd.arg("example.com").arg("0");
    cmd.assert().failure().stderr(predicate::str::contains("Port cannot be zero"));
}

#[test]
fn test_both_ipv4_and_ipv6_flags() {
    let mut cmd = Command::cargo_bin("tcping").unwrap();
    cmd.arg("-4").arg("-6").arg("example.com").arg("80");
    cmd.assert().failure().stderr(predicate::str::contains("Cannot use both -4 and -6 flags"));
}

#[test]
fn test_json_output_flag() {
    let mut cmd = Command::cargo_bin("tcping").unwrap();
    cmd.arg("-j").arg("example.com").arg("80").arg("-c").arg("1");
    cmd.assert().success().stdout(predicate::str::contains("target"));
}

#[test]
fn test_csv_output_flag() {
    let mut cmd = Command::cargo_bin("tcping").unwrap();
    cmd.arg("--csv").arg("example.com").arg("80").arg("-c").arg("1");
    cmd.assert().success().stdout(predicate::str::contains("CSV results written to"));
}

#[test]
fn test_no_color_flag() {
    let mut cmd = Command::cargo_bin("tcping").unwrap();
    cmd.arg("--no-color").arg("example.com").arg("80").arg("-c").arg("1");
    cmd.assert().success();
}

#[test]
fn test_show_source_address_flag() {
    let mut cmd = Command::cargo_bin("tcping").unwrap();
    cmd.arg("--show-source-address").arg("example.com").arg("80").arg("-c").arg("1");
    cmd.assert().success();
}

#[test]
fn test_retry_flag() {
    let mut cmd = Command::cargo_bin("tcping").unwrap();
    cmd.arg("-r").arg("3").arg("example.com").arg("80").arg("-c").arg("1");
    cmd.assert().success();
}

#[test]
fn test_count_flag() {
    let mut cmd = Command::cargo_bin("tcping").unwrap();
    cmd.arg("-c").arg("5").arg("example.com").arg("80");
    cmd.assert().success();
}

#[test]
fn test_interval_flag() {
    let mut cmd = Command::cargo_bin("tcping").unwrap();
    cmd.arg("-i").arg("2.5").arg("example.com").arg("80").arg("-c").arg("1");
    cmd.assert().success();
}

#[test]
fn test_timeout_flag() {
    let mut cmd = Command::cargo_bin("tcping").unwrap();
    cmd.arg("-t").arg("10.0").arg("example.com").arg("80").arg("-c").arg("1");
    cmd.assert().success();
}

#[test]
fn test_verbose_flag() {
    let mut cmd = Command::cargo_bin("tcping").unwrap();
    cmd.arg("-v").arg("example.com").arg("80").arg("-c").arg("1");
    cmd.assert().success();
}

#[test]
fn test_debug_flag() {
    let mut cmd = Command::cargo_bin("tcping").unwrap();
    cmd.arg("-D").arg("example.com").arg("80").arg("-c").arg("1");
    cmd.assert().success();
}