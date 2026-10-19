# Case 03 — SPI flash does not answer: ID read returns nothing

> Status: **method / framework demonstration. Synthetic scenario, no
> measured data.** The probing table below is a plan for what each
> signal would prove if captured on real hardware — it is not a capture.

Related tool output (synthetic): `samples/synthetic_test_run.log`
line 10 — `spi_flash_id FAIL code=ERR_SPI_NO_RESPONSE`. An all-zeros or
all-ones ID read is one of the most misread symptoms in bring-up: it
usually says "the bus conversation never happened", and only rarely
"the flash is dead".

## 1. Phenomenon

- Symptom (synthetic scenario): firmware issues a Read-ID command; the
  returned bytes are all `0xFF` (bus floating / device never drives) —
  the all-`0x00` variant (line held low) is the sibling symptom and is
  noted where the reasoning differs.
- The rest of the board is alive: clocks run, other test steps pass, so
  this is a one-peripheral conversation failure, not a dead board.
- First wrong move to avoid: replacing the flash. It is the *last*
  step of this case, not the first.

## 2. Measurement

Probe the four SPI signals plus power at the flash pins — logic
analyzer or 4-channel scope — triggered on chip-select falling:

| Signal | Healthy expectation | What each abnormal result proves |
|---|---|---|
| CS# | Falls before the clock burst, rises after it | Never falls → controller / pin-mux / firmware never selected the device; stop debugging the flash |
| SCLK | Burst of clocks for command + data (e.g. 8 + 24 for a 3-byte ID) | No clocks with CS# low → clock path or controller config; clocks with wrong count → command framing problem |
| MOSI | Command byte (Read-ID opcode) then don't-care / address bytes | Wrong opcode on the wire → firmware / driver issue, proven at the pins, no guessing |
| MISO | High-impedance until the data phase, then the ID bytes | Stays high the whole frame → device never drove: power, CS# reaching the device, HOLD# / WP# state, or the device itself |
| VCC at device | In-spec, stable through the frame | Absent / low → power distribution, same discipline as Case 02 |

Static checks alongside (power off / idle): HOLD# and WP# pin states
against the schematic (a held HOLD# makes a healthy flash mute), CS#
continuity from controller to device, and solder inspection of the
flash pins under magnification — one unsoldered pin explains
everything and is cheap to rule out early.

## 3. Localization

Read the capture as a funnel; each answer halves the suspect set:

1. **Is there a conversation at all?** No CS#, no clocks → the fault is
   on the controller side: pin mux, clock / power domain of the SPI
   controller, or firmware never reaching the read. The flash is not
   yet a suspect.
2. **Conversation at the controller, not at the device?** Signals good
   at the controller pins, absent at the flash pins → the path:
   continuity, series elements, solder joints.
3. **Full conversation at the device, MISO silent?** Now the device
   neighbourhood is the suspect set: its power, HOLD# / WP#, its
   solder, and only then the device itself.
4. **MISO answers but the ID is wrong / garbled?** Different case
   entirely: mode (CPOL/CPHA) mismatch, clock too fast for the setup,
   or signal integrity — the framing and timing questions, not the
   presence questions.
5. Cross-check with a second method before any rework: e.g. read the
   same flash with an external programmer / clip if the design allows,
   or probe a second, known-good unit with the identical setup.

## 4. Fix

- Controller-side (mux / config / firmware) → fix the configuration;
  the "repair" is a commit, and the case record should say so plainly.
- Path / solder → rework the named joint or pin, identified by the
  funnel step that isolated it.
- HOLD# / WP# strapping wrong → correct the strap; note it as a design
  / assembly finding (DFx feedback), not a component failure.
- Device itself → replace only after steps 1–3 and the cross-check
  have all pointed at it, and keep the removed part: a replaced part
  that was never proven dead is an open case wearing a closed costume.

## 5. Verification

- Read-ID returns the expected ID consistently across cold boots and
  repeated reads (hundreds, not once).
- Full-path test: erase / write / read-back a test pattern in a
  scratch area if the firmware supports it — ID alone does not prove
  the data path.
- The original failing step (`spi_flash_id`) passes inside the full
  test sequence, not only standalone.

## Honest boundary

- This case demonstrates a **localization method**: a signal-by-signal
  expectation table and a funnel that keeps the flash itself as the
  last suspect. The scenario and log line are synthetic.
- **Not yet done:** a real 4-channel capture, real continuity /
  resistance readings, and a real fix record on physical hardware.
  Those require a board and instruments and are future work. Nothing
  here is evidence of hands-on SPI probing experience.
