//! board-log-parser library: parse board / manufacturing test logs.
//!
//! Clean-room implementation, no external crates.
//! Input is line-oriented text such as (all synthetic, invented for this lab):
//!
//! ```text
//! [2026-01-15T09:12:03Z] TEST power_rail_3v3 PASS measured=3.31V
//! [2026-01-15T09:12:07Z] TEST i2c_scan FAIL code=ERR_I2C_NACK addr=0x50
//! [2026-01-15T09:12:09Z] TEST uart_loopback ERROR code=0xE104 timeout_ms=500
//! ```
//!
//! A line is a test result when it contains the token `TEST` followed by a
//! test name, plus one of PASS / PASSED / FAIL / FAILED / ERROR / SKIP /
//! SKIPPED. Other lines (boot banners, free-form serial output) are counted
//! as unparsed and never silently treated as passes.

use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Pass,
    Fail,
    Error,
    Skip,
}

impl Status {
    pub fn as_str(&self) -> &'static str {
        match self {
            Status::Pass => "PASS",
            Status::Fail => "FAIL",
            Status::Error => "ERROR",
            Status::Skip => "SKIP",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestResult {
    pub line_no: usize,
    pub timestamp: Option<String>,
    pub test_name: String,
    pub status: Status,
    pub error_codes: Vec<String>,
    pub detail: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Summary {
    pub total_lines: usize,
    pub parsed_results: usize,
    pub unparsed_lines: usize,
    pub passed: usize,
    pub failed: usize,
    pub errors: usize,
    pub skipped: usize,
    /// error code -> occurrence count, sorted by code (BTreeMap)
    pub error_code_counts: BTreeMap<String, usize>,
    /// test name -> occurrence count for FAIL / ERROR results only
    pub failing_tests: BTreeMap<String, usize>,
}

impl Summary {
    pub fn failure_results<'a>(&self, results: &'a [TestResult]) -> Vec<&'a TestResult> {
        results
            .iter()
            .filter(|r| matches!(r.status, Status::Fail | Status::Error))
            .collect()
    }

    pub fn to_json(&self, results: &[TestResult]) -> String {
        let mut out = String::new();
        out.push_str("{\n");
        out.push_str(&format!("  \"total_lines\": {},\n", self.total_lines));
        out.push_str(&format!("  \"parsed_results\": {},\n", self.parsed_results));
        out.push_str(&format!("  \"unparsed_lines\": {},\n", self.unparsed_lines));
        out.push_str(&format!("  \"passed\": {},\n", self.passed));
        out.push_str(&format!("  \"failed\": {},\n", self.failed));
        out.push_str(&format!("  \"errors\": {},\n", self.errors));
        out.push_str(&format!("  \"skipped\": {},\n", self.skipped));
        out.push_str("  \"error_code_counts\": {");
        let codes: Vec<String> = self
            .error_code_counts
            .iter()
            .map(|(k, v)| format!("\"{}\": {}", json_escape(k), v))
            .collect();
        out.push_str(&codes.join(", "));
        out.push_str("},\n");
        out.push_str("  \"failures\": [");
        let fails = self.failure_results(results);
        let items: Vec<String> = fails
            .iter()
            .map(|r| {
                let codes: Vec<String> = r
                    .error_codes
                    .iter()
                    .map(|c| format!("\"{}\"", json_escape(c)))
                    .collect();
                format!(
                    "{{\"line\": {}, \"test\": \"{}\", \"status\": \"{}\", \"error_codes\": [{}]}}",
                    r.line_no,
                    json_escape(&r.test_name),
                    r.status.as_str(),
                    codes.join(", ")
                )
            })
            .collect();
        out.push_str(&items.join(", "));
        out.push_str("]\n}\n");
        out
    }

    pub fn to_text(&self, results: &[TestResult]) -> String {
        let mut out = String::new();
        out.push_str("Test log summary\n");
        out.push_str("----------------\n");
        out.push_str(&format!("total lines:     {}\n", self.total_lines));
        out.push_str(&format!("parsed results:  {}\n", self.parsed_results));
        out.push_str(&format!("unparsed lines:  {}\n", self.unparsed_lines));
        out.push_str(&format!("passed:          {}\n", self.passed));
        out.push_str(&format!("failed:          {}\n", self.failed));
        out.push_str(&format!("errors:          {}\n", self.errors));
        out.push_str(&format!("skipped:         {}\n", self.skipped));
        if !self.error_code_counts.is_empty() {
            out.push_str("\nerror codes:\n");
            for (code, n) in &self.error_code_counts {
                out.push_str(&format!("  {} x{}\n", code, n));
            }
        }
        let fails = self.failure_results(results);
        if !fails.is_empty() {
            out.push_str("\nfailures:\n");
            for r in fails {
                out.push_str(&format!(
                    "  line {}: {} {} codes=[{}] {}\n",
                    r.line_no,
                    r.test_name,
                    r.status.as_str(),
                    r.error_codes.join(", "),
                    r.detail
                ));
            }
        }
        out
    }
}

pub fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

/// Extract error codes from a line.
/// Recognised shapes:
/// - `code=ERR_I2C_NACK`, `code=0xE104`
/// - bare tokens `ERR_<UPPER/DIGITS/_>`, `E_<DIGITS>` is NOT matched (too noisy),
///   bare hex tokens starting with `0x` followed by hex digits.
pub fn extract_error_codes(line: &str) -> Vec<String> {
    let mut codes: Vec<String> = Vec::new();
    for raw in line.split(|c: char| c.is_whitespace() || c == ',' || c == ';') {
        let tok = raw.trim_matches(|c| c == '[' || c == ']' || c == '(' || c == ')');
        if tok.is_empty() {
            continue;
        }
        let candidate: Option<&str> = if let Some(rest) = tok.strip_prefix("code=") {
            Some(rest)
        } else if tok.starts_with("ERR_") {
            Some(tok)
        } else if tok.starts_with("0x") && tok.len() > 2 && tok[2..].chars().all(|c| c.is_ascii_hexdigit()) {
            Some(tok)
        } else {
            None
        };
        if let Some(c) = candidate {
            let c = c.trim_end_matches(|x| x == '.' || x == ':');
            if !c.is_empty() && !codes.iter().any(|x| x == c) {
                codes.push(c.to_string());
            }
        }
    }
    codes
}

fn parse_status(tokens: &[&str]) -> Option<Status> {
    // The last status-like token wins, so a detail such as
    // "previous=PASS" does not override the real verdict.
    let mut found = None;
    for t in tokens {
        let up = t.to_ascii_uppercase();
        match up.as_str() {
            "PASS" | "PASSED" => found = Some(Status::Pass),
            "FAIL" | "FAILED" => found = Some(Status::Fail),
            "ERROR" => found = Some(Status::Error),
            "SKIP" | "SKIPPED" => found = Some(Status::Skip),
            _ => {}
        }
    }
    found
}

pub fn parse_line(line_no: usize, line: &str) -> Option<TestResult> {
    let tokens: Vec<&str> = line.split_whitespace().collect();
    let test_idx = tokens.iter().position(|t| *t == "TEST")?;
    let test_name = tokens.get(test_idx + 1)?.to_string();
    let status = parse_status(&tokens)?;
    let timestamp = if line.starts_with('[') {
        line.find(']').map(|end| line[1..end].to_string())
    } else {
        None
    };
    Some(TestResult {
        line_no,
        timestamp,
        test_name,
        status,
        error_codes: extract_error_codes(line),
        detail: line.trim().to_string(),
    })
}

pub fn parse_log(text: &str) -> (Vec<TestResult>, Summary) {
    let mut results = Vec::new();
    let mut summary = Summary::default();
    for (idx, line) in text.lines().enumerate() {
        summary.total_lines += 1;
        if line.trim().is_empty() {
            summary.unparsed_lines += 1;
            continue;
        }
        match parse_line(idx + 1, line) {
            Some(r) => {
                match r.status {
                    Status::Pass => summary.passed += 1,
                    Status::Fail => summary.failed += 1,
                    Status::Error => summary.errors += 1,
                    Status::Skip => summary.skipped += 1,
                }
                if matches!(r.status, Status::Fail | Status::Error) {
                    *summary.failing_tests.entry(r.test_name.clone()).or_insert(0) += 1;
                    for c in &r.error_codes {
                        *summary.error_code_counts.entry(c.clone()).or_insert(0) += 1;
                    }
                }
                summary.parsed_results += 1;
                results.push(r);
            }
            None => summary.unparsed_lines += 1,
        }
    }
    (results, summary)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "\
[2026-01-15T09:12:03Z] TEST power_rail_3v3 PASS measured=3.31V
[2026-01-15T09:12:05Z] TEST power_rail_5v PASS measured=5.02V
[2026-01-15T09:12:07Z] TEST i2c_scan FAIL code=ERR_I2C_NACK addr=0x50
[2026-01-15T09:12:09Z] TEST uart_loopback ERROR code=0xE104 timeout_ms=500
[2026-01-15T09:12:11Z] TEST spi_flash_id FAIL code=ERR_SPI_NO_RESPONSE
[2026-01-15T09:12:13Z] TEST fan_tach SKIP reason=no_fixture
boot banner: synthetic board rev A, not a test line
";

    #[test]
    fn counts_statuses_correctly() {
        let (_r, s) = parse_log(SAMPLE);
        assert_eq!(s.total_lines, 7);
        assert_eq!(s.parsed_results, 6);
        assert_eq!(s.unparsed_lines, 1);
        assert_eq!(s.passed, 2);
        assert_eq!(s.failed, 2);
        assert_eq!(s.errors, 1);
        assert_eq!(s.skipped, 1);
    }

    #[test]
    fn extracts_error_codes() {
        assert_eq!(extract_error_codes("x code=ERR_I2C_NACK y"), vec!["ERR_I2C_NACK"]);
        assert_eq!(extract_error_codes("code=0xE104 timeout"), vec!["0xE104"]);
        assert_eq!(
            extract_error_codes("ERR_SPI_NO_RESPONSE and ERR_SPI_NO_RESPONSE"),
            vec!["ERR_SPI_NO_RESPONSE"]
        );
        assert!(extract_error_codes("plain serial chatter 1234").is_empty());
    }

    #[test]
    fn aggregates_error_code_counts() {
        let (_r, s) = parse_log(SAMPLE);
        assert_eq!(s.error_code_counts.get("ERR_I2C_NACK"), Some(&1));
        assert_eq!(s.error_code_counts.get("0xE104"), Some(&1));
        assert_eq!(s.error_code_counts.get("ERR_SPI_NO_RESPONSE"), Some(&1));
        assert_eq!(s.failing_tests.get("i2c_scan"), Some(&1));
    }

    #[test]
    fn parses_timestamp_and_line_numbers() {
        let (r, _s) = parse_log(SAMPLE);
        assert_eq!(r[0].timestamp.as_deref(), Some("2026-01-15T09:12:03Z"));
        assert_eq!(r[2].line_no, 3);
        assert_eq!(r[2].test_name, "i2c_scan");
    }

    #[test]
    fn non_test_lines_are_not_passes() {
        let (_r, s) = parse_log("hello PASS world\nrandom FAIL chatter\n");
        assert_eq!(s.parsed_results, 0);
        assert_eq!(s.passed, 0);
        assert_eq!(s.failed, 0);
    }

    #[test]
    fn json_output_is_structured() {
        let (r, s) = parse_log(SAMPLE);
        let j = s.to_json(&r);
        assert!(j.contains("\"passed\": 2"));
        assert!(j.contains("\"test\": \"i2c_scan\""));
        assert!(j.contains("\"ERR_I2C_NACK\": 1"));
    }

    #[test]
    fn empty_input_is_zeroed() {
        let (_r, s) = parse_log("");
        assert_eq!(s.total_lines, 0);
        assert_eq!(s.parsed_results, 0);
    }
}
