use board_log_parser::parse_log;

#[test]
fn smoke_two_failures_one_error_code_repeated() {
    let log = "\
TEST a PASS
TEST b FAIL code=ERR_X
TEST c FAIL code=ERR_X
TEST d ERROR code=ERR_Y
";
    let (_r, s) = parse_log(log);
    assert_eq!(s.passed, 1);
    assert_eq!(s.failed, 2);
    assert_eq!(s.errors, 1);
    assert_eq!(s.error_code_counts.get("ERR_X"), Some(&2));
    assert_eq!(s.error_code_counts.get("ERR_Y"), Some(&1));
}
