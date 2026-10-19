# VERIFICATION — build / test / run records

All records below are from actual runs on the build machine on
2026-10-19. Nothing here is estimated or copied from documentation.

## Toolchain — how it got here, and versions

Rust was **not** preinstalled on this machine (`cargo` / `rustc`: command
not found). It was installed per-user with rustup, minimal profile,
stable channel, into `~/.cargo`:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs -o /tmp/rustup-init.sh
sh /tmp/rustup-init.sh -y --profile minimal --default-toolchain stable
```

Measured versions after install:

```text
rustup 1.29.1 (d95a37b6a 2026-08-13)
rustc  1.99.1 (58cf8f9c3 2026-10-13)
cargo  1.99.1 (5f94df478 2026-08-27)
```

Python side: no installation was needed — Python 3.12.3 and matplotlib
3.6.3 were already present on the machine and were used as-is.

```text
Python 3.12.3
matplotlib 3.6.3
OS: Linux 7.0.0-39-generic x86_64
```

The Rust tool has **zero external dependencies** (empty `[dependencies]`
in `Cargo.toml`), so the build needs no crates.io downloads and is
reproducible offline once a toolchain exists.

## cargo build — actual output (summary)

```text
$ cd rust-log-parser && cargo build
   Compiling board-log-parser v0.1.0
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.64s
```
Exit code 0. Binary: `target/debug/board-log-parser` (~5.5 MB).

## cargo test — actual output (summary)

```text
running 7 tests                       (src/lib.rs unit tests)
test tests::aggregates_error_code_counts ... ok
test tests::counts_statuses_correctly ... ok
test tests::extracts_error_codes ... ok
test tests::empty_input_is_zeroed ... ok
test tests::non_test_lines_are_not_passes ... ok
test tests::parses_timestamp_and_line_numbers ... ok
test tests::json_output_is_structured ... ok
test result: ok. 7 passed; 0 failed

running 1 test                        (tests/cli_smoke.rs integration test)
test smoke_two_failures_one_error_code_repeated ... ok
test result: ok. 1 passed; 0 failed

doc-tests: 0 run
```
Total: **8 tests passed, 0 failed.** Exit code 0.

## Running the tool on the synthetic sample log — actual output

Input: `samples/synthetic_test_run.log` (synthetic, 15 lines).

Text mode:

```text
Test log summary
----------------
total lines:     15
parsed results:  11
unparsed lines:  4
passed:          6
failed:          3
errors:          1
skipped:         1

error codes:
  0xE104 x1
  ERR_I2C_NACK x2
  ERR_SPI_NO_RESPONSE x1

failures:
  line 7:  i2c_scan FAIL codes=[ERR_I2C_NACK] ...
  line 8:  i2c_scan FAIL codes=[ERR_I2C_NACK] ...
  line 9:  uart_loopback ERROR codes=[0xE104] ...
  line 10: spi_flash_id FAIL codes=[ERR_SPI_NO_RESPONSE] ...
```

Exit code in all three modes (`default`, `--json`, `--failures-only`)
was **1**, which is the tool's documented behaviour when a log contains
FAIL/ERROR results — it is how a CI step would gate on the summary, not
a tool failure. The `--json` output was visually checked to contain the
same counts and the three distinct error codes with correct tallies.

## Synthetic waveform — generation record

```text
$ python3 tools/generate_waveforms.py
wrote data/waveforms/case01_i2c_sda_stuck_low.csv (241 rows)
wrote data/waveforms/case01_i2c_waveform.png
```

- CSV: 245 lines total (3 synthetic-labelling comment lines + header +
  241 data rows), ~2.9 KB. Columns:
  `time_us,scl,expected_sda,observed_sda_stuck_low`.
- PNG: 1760 x 832, ~62 KB. Visually inspected after generation: SCL
  shows 9 clock pulses, expected SDA shows the data + ACK pattern, the
  observed SDA trace stays low after the START edge, and the title and
  annotation both say SYNTHETIC / not measured data.
- The data is a deterministic programmatic model (no random component);
  re-running the script regenerates the same files.

## What was NOT verified / not done

- No physical board, oscilloscope, or logic analyzer was involved at
  any point. No real waveform, voltage, or timing was measured.
- The cases in `cases/` are method frameworks over synthetic scenarios,
  verified only for internal consistency (chain completeness, labelling,
  and that every claim in them is framed as a plan or a reading
  exercise, not as a performed measurement).
- The repo is local only; it has not been pushed anywhere.
