# Case 02 — 3.3 V rail passes at idle, downstream device resets under load

> Status: **method / framework demonstration. Synthetic scenario, no
> measured data.** No waveform is attached to this case on purpose:
> inventing a ripple plot and presenting it next to a measurement plan
> would blur exactly the line this repo keeps — synthetic material is
> generated only where it is labelled as a reading exercise (Case 01),
> and a rail-droop claim needs a real capture to mean anything.

Related tool output (synthetic): `samples/synthetic_test_run.log` line 4
shows the idle-side half of this pattern — `power_rail_3v3 PASS
measured=3.31V` — taken with no load step. This case is about why that
PASS does not close the question.

## 1. Phenomenon

- Symptom (synthetic scenario): board boots, idle rail checks pass, but
  a downstream device resets or drops off its bus when a load step
  happens (peripheral activity, transmit burst, motor / fan start —
  whatever the board's worst transient is).
- Failures correlate with *activity*, not with time since power-on and
  not with temperature in the scenario as framed.
- First wrong move to avoid: re-testing the rail the same idle way,
  getting the same PASS, and declaring the power tree healthy.

## 2. Measurement

1. **Define the load step first.** Write down the exact action that
   triggers the reset and make it repeatable on demand. No trigger,
   no debug — only anecdotes.
2. **Scope the rail at the load, not at the regulator.** Probe at the
   failing device's supply pins (short ground spring, not a long ground
   lead — the lead's inductance invents ringing that is not on the
   rail). Record: DC level before the step, minimum voltage during the
   step, recovery time, and whether the power-good / reset line moves
   with it.
3. **Compare two points on the same net:** regulator output vs. device
   pins during the same trigger. Droops at both → source / transient
   response problem. Droops only at the device → distribution problem
   (resistance in the path: connector, trace, via, solder joint).
4. **Current side:** if equipment allows, observe the load current step
   shape. A step far larger than the design estimate points back at the
   load (shorted or misbehaving device), not at the regulator.
5. **Multimeter is the wrong instrument for the main question** — it
   averages away exactly the transient that resets the device. It still
   earns its place for the static checks: continuity of the power path,
   resistance to ground compared against expectation.

## 3. Localization

- Droop depth at the device vs. at the regulator splits the case in
  two: *generation* (regulator cannot hold the rail through the step)
  vs. *distribution* (rail is fine at the source, lost on the way).
- Power-good / reset correlation: if the supervisor asserts reset
  exactly at the droop minimum, the reset is a *symptom* — do not
  debug the reset circuit.
- Swap discipline: change one thing per experiment (decoupling at the
  load, a known-good supply substituted at the input, the load device
  isolated). Record droop minimum after each change with the identical
  trigger, otherwise results cannot be compared.
- Schematic reading is part of localization here: trace the net from
  regulator to device, list every series element and every branch, and
  check the design's decoupling placement against where the droop is
  actually observed.

## 4. Fix

Branch-matched, as in Case 01:

- Distribution loss at a joint / connector → rework that joint; verify
  by the two-point measurement, not by "it boots now".
- Insufficient local decoupling for the real load step → add / correct
  capacitance at the load per the device datasheet's requirements.
- Regulator transient response out of spec for the step → component
  change is a design conversation (DFx feedback), not a silent bench
  substitution on a unit meant to represent the design.
- Load device drawing abnormal current → the power case closes and a
  device case opens; do not "fix" it with a bigger supply.

## 5. Verification

- Same trigger, repeated (tens of cycles, not one): droop minimum stays
  inside the device's supply specification with margin.
- The original failing test sequence passes end-to-end, plus a soak
  run under the activity pattern that used to reset the device.
- Idle checks still pass — a fix that trades idle behaviour for loaded
  behaviour is not a fix.

## Honest boundary

- Scenario and reasoning chain are written as a **debug framework**:
  they show the measurement order, the probe discipline, and the
  generation-vs-distribution split that this fault class turns on.
- **Not yet done:** real scope captures of a drooping rail, real droop
  numbers, and a real rework / verification record. Those need a board,
  a load, and an oscilloscope, and are future work. No number in this
  case is a measurement.
