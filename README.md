# board-debug-lab

A small, honest proof-of-work lab for board-level debug *method* and one
Rust test-log tool — built as a gap proof artifact for a Hardware Test
Engineer application (Oxide Computer).

## What this repo is — and is not

**It is:**

- A runnable Rust tool that parses manufacturing / board test logs and
  serial-style logs into a structured failure summary (counts, failing
  tests, error-code aggregation), with unit and integration tests.
- Three board-fault case write-ups, each following the full chain
  **phenomenon → measurement → localization → fix → verification**,
  written to show debug reasoning order: cheapest measurement first,
  isolate before replacing, fix matched to the isolated cause, verify
  with the original failing trigger.
- One signal-analysis reading exercise (Case 01): a Python-generated
  **synthetic** I2C waveform (CSV + PNG) showing how an "SDA stuck low"
  trace would be read against an expected trace.

**It is not:**

- **Not** a record of real board repairs. Every log line, scenario, and
  waveform here is synthetic / invented for this lab and labelled so.
- **Not** evidence of hands-on oscilloscope or logic-analyzer work.
  The real instrument work — probing a physical board, capturing real
  waveforms, taking real voltage / timing measurements, reworking a
  real fault and verifying it — **is still to be done when equipment
  and a board are available.** This repo covers the method-and-tooling
  part only; board-level measurement depth is explicitly **not** proven
  by this repo.
- Clean-room throughout: no employer or customer material, logs, part
  numbers from real products, or measured data from anyone's hardware.

## Structure

```text
board-debug-lab/
  README.md                  this file
  VERIFICATION.md            toolchain versions + actual build/test/run records
  rust-log-parser/           the Rust tool (zero external dependencies)
    Cargo.toml
    src/lib.rs               parser library + unit tests
    src/main.rs              CLI: text / --json / --failures-only
    tests/cli_smoke.rs       integration test
  samples/
    synthetic_test_run.log   SYNTHETIC sample log the tool is demoed on
  cases/
    case-01-i2c-sda-stuck-low.md          I2C NACK / SDA stuck low (+ waveform exercise)
    case-02-power-rail-droop-under-load.md 3.3 V rail droop, generation vs distribution
    case-03-spi-flash-no-response.md      SPI flash ID read funnel, signal-by-signal table
  tools/
    generate_waveforms.py    generates the synthetic Case 01 waveform
  data/waveforms/
    case01_i2c_sda_stuck_low.csv   SYNTHETIC data (regenerable)
    case01_i2c_waveform.png        SYNTHETIC plot (regenerable)
```

## How to run

Rust tool (needs a Rust toolchain, e.g. installed via rustup):

```bash
cd rust-log-parser
cargo build
cargo test

# text summary (exit code 1 means the log contains FAIL/ERROR results)
./target/debug/board-log-parser ../samples/synthetic_test_run.log

# structured JSON summary
./target/debug/board-log-parser ../samples/synthetic_test_run.log --json

# only the failing lines
./target/debug/board-log-parser ../samples/synthetic_test_run.log --failures-only
```

Log format the parser understands (one result per line):

```text
[timestamp] TEST <test_name> <PASS|FAIL|ERROR|SKIP> [code=ERR_... | code=0x....] [key=value ...]
```

Lines without a `TEST` token and a verdict are counted as unparsed and
are never silently treated as passes.

Synthetic waveform (Python 3 + matplotlib):

```bash
python3 tools/generate_waveforms.py
# rewrites data/waveforms/case01_i2c_sda_stuck_low.csv and .png
```

## Honest boundary (read this before citing the repo)

- All data is synthetic and labelled synthetic: the sample log, the
  case scenarios, and the I2C waveform.
- The cases demonstrate a debug *method and writing discipline*. They
  do not demonstrate instrument proficiency, and they must not be
  presented — in a resume, cover letter, or interview — as performed
  board-level measurements or repairs.
- What would close that gap: the same three cases executed on a real
  (own / sacrificial) board with a real oscilloscope and logic
  analyzer, real captures replacing the synthetic plot, and real
  measured values recorded in each case. That is the stated next step,
  not a completed one.
