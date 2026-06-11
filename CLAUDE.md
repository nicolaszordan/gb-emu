# CLAUDE.md

Guidance for working in this repository.

## Overview

`gb-emu` is a Game Boy emulator written in Rust, structured as a Cargo workspace.
The emulator core is platform-agnostic; frontends for Web, Mobile, and Desktop are
planned. Today the desktop (`pc`) frontend exists and the bulk of the work is the
Sharp SM83 CPU and motherboard in the `gb` crate.

## Workspace layout

`Cargo.toml` is a workspace (resolver "2"); every crate is edition **2024**.

| Crate | Path | Purpose | Dependencies |
|-------|------|---------|--------------|
| `emu` | `emulators/emu` | Core emulation primitives shared across platforms: the `MemoryBus` trait (`src/mem.rs`), `bit_index` helpers, and a `test-utilities` feature exposing `MockMemoryBus`. | none |
| `gb` | `emulators/gb` | Game Boy core: CPU, motherboard, interrupts, instruction decode/execute. | `emu` (path), `bitflags = "2.11"` |
| `pc` | `frontends/pc` | Desktop frontend. | `gb` (path) |
| `instruction-codegen` | `tools/instruction-codegen` | CLI that generates instruction metadata tables from a JSON opcode spec. | `serde`, `serde_json`, `clap 4.5` |

Dependency graph:

```
emu ──> gb ──> pc
instruction-codegen   (standalone)
```

There is no MSRV pin, `rustfmt.toml`, or Makefile. CI uses the latest **stable**
toolchain.

## Commands

CI (`.github/workflows/rust-ci.yml`) runs these exact checks, with
`RUSTFLAGS: -Dwarnings` — **warnings are errors**, so clippy and fmt must be clean:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets
cargo test --workspace
```

Common dev variants:

```bash
cargo build -p gb              # build just the GB core
cargo test  -p gb             # run the GB core tests
cargo run   -p pc             # run the desktop frontend
cargo fmt --all               # actually format (no --check)
```

Regenerate the instruction metadata table (see below — only when opcode metadata
changes):

```bash
cargo run -p instruction-codegen -- \
  --input files/instructions/gb_instructions.json \
  --output emulators/gb/src/cpu/instructions/meta.rs
```

## Architecture

### Top level (`emulators/gb/src/lib.rs`)

```rust
pub struct GameBoy { cpu: CPU, mb: MotherBoard }
```

`GameBoy::step()` runs one CPU instruction, gets the cycle count back, then advances
the motherboard's sub-components by that many cycles:

```rust
let cycles = self.cpu.step(&mut self.mb);
self.mb.step(cycles);
```

### Bus traits

The CPU never touches concrete hardware — it works through two traits, both
implemented by `MotherBoard`:

- **`MemoryBus`** (`emulators/emu/src/mem.rs`): `read`/`write` plus little-endian
  `read_word`/`write_word` and `read_range`. CPU methods are generic over
  `M: MemoryBus`.
- **`InterruptLine`** (`emulators/gb/src/interrupts.rs`): `pending_interrupt` /
  `acknowledge_interrupt`. The IE/IF registers live in `motherboard/interrupts.rs`.

`MotherBoard` (`motherboard.rs`) implements both, so `&mut MotherBoard` satisfies the
`M: MemoryBus + InterruptLine` bound that `CPU::step` requires. Use the same generic
bounds (rather than a concrete bus) so code stays testable against `MockMemoryBus`.

### CPU (`emulators/gb/src/cpu.rs` + `cpu/`)

```rust
pub struct CPU { registers: Registers, sp: u16, pc: u16, ime: IME, state: CPUState }
```

`CPU::step` ordering matters:

1. Try to dispatch a pending interrupt (`try_dispatch_pending_interrupt`); if one
   fires, return its cycle count and stop.
2. `ime.commit_pending()` — applies the one-instruction delay of `EI`.
3. Fetch the opcode and call `execute_instruction`.

Submodules under `cpu/`:

- `registers.rs` — eight 8-bit registers plus `af`/`bc`/`de`/`hl` 16-bit pair
  getters/setters; `Flags { z, n, h, c }` with `From<u8>`/`Into<u8>` (low nibble
  always 0).
- `alu.rs` — free functions returning `(result, Flags)`; instructions apply the
  returned flags selectively.
- `stack.rs` — `StackController` push/pop.
- `interrupts.rs` — `IME` state machine: `PendingEnable` → `Enabled` → `Disabled`.
- `state.rs` — `CPUState` (`Running`, `Halted`).

### Instruction subsystem (`cpu/instructions/`)

The decode/execute pipeline is layered:

- `dispatch.rs` — **hand-written** `execute_instruction(opcode)` `match`. Maps opcode
  bytes to `instr_*` calls, decoding parameters from opcode bits.
- `execute.rs` — the `instr_*` methods. Each returns the **additional** cycle count
  beyond the base timing (often `0`).
- `operand/operand8.rs`, `operand/operand16.rs` — `Operand8`/`Operand16` enums with
  `read`/`write`; they encapsulate immediate fetches (advancing `pc`) and side effects
  like `HL` increment/decrement.
- `parameter.rs` — `*Param` enums that decode register/addressing info from opcode bits
  via `From<u8>`.
- `condition.rs` — jump/call condition checks.
- `meta.rs` — **auto-generated, do not edit** (`UNPREFIXED_INSTRUCTIONS:
  [InstructionMeta; 256]`). Regenerate via `instruction-codegen`.

## Adding a CPU instruction

1. **Parameter decoding** — if the instruction needs an addressing mode not already
   covered, add a variant to the relevant `*Param` enum in
   `cpu/instructions/parameter.rs` and implement/extend its `From<u8>` bit decode.
2. **Operands** — if a new operand source/destination is needed, add a variant to
   `Operand8`/`Operand16` in `cpu/instructions/operand/`, and implement its `read` /
   `write` (and any `From<…Param>` conversion).
3. **Execute** — add `pub(crate) fn instr_<name><M: MemoryBus>(&mut self, bus: &mut M,
   …) -> u32` in `cpu/instructions/execute.rs`. Read/write through `Operand*`, use
   `alu::*` for arithmetic, apply the returned `Flags` selectively, and return the
   *additional* cycles. (`instr_ld8` is a clean reference; `NOP` returns `0`.)
4. **Dispatch** — add the opcode `match` arm(s) in `cpu/instructions/dispatch.rs`,
   decoding params from the opcode and calling your `instr_*`.
5. **Tests** — colocate `#[cfg(test)] mod tests { … }` tests in the same file, driving
   the CPU against `MockMemoryBus` (or a local mock implementing both bus traits).
6. **Metadata** — if you changed opcode metadata (mnemonic/bytes/cycles), update
   `files/instructions/gb_instructions.json` and regenerate `meta.rs` with the
   `instruction-codegen` command above. Never hand-edit `meta.rs`.

## Conventions

- Edition 2024; workspace clippy lints are `pedantic` and `nursery` = `warn`, and CI
  promotes warnings to errors. Keep both clippy and `rustfmt` clean.
- `#[allow(clippy::upper_case_acronyms)]` (and `upper_case_acronyms`) is used
  deliberately to keep names (`CPU`, `IME`, `ALU`) aligned with the Pan Docs.
- 16-bit memory access is little-endian; addresses/`pc` use `wrapping_add`; high memory
  / I/O registers are based at `0xFF00`.
- Tests are colocated in `#[cfg(test)]` modules, often with nested topic submodules.
- Prefer generic `M: MemoryBus (+ InterruptLine)` bounds over concrete bus types so code
  can be tested with mocks.

## Git / contribution workflow

- **Commits**: Conventional Commits with a scope, e.g. `feat(gb/cpu): …`,
  `feat(gb/mb): …`, `ci: …`, `doc(gb): …`.
- **Branches**: `<issue#>-<type>-<short-desc>`, e.g.
  `17-feat-gb-cpu-implement-instruction-halt`.
- **PRs**: one feature per PR into `main`; the squashed commit references the PR number
  (`… (#NN)`). CI must pass (`fmt --check`, `clippy --all-targets`, `test`, all with
  warnings-as-errors) before merge.

## Reference material

- Pan Docs, PyBoy, and TCAGBD PDFs in `files/docs/` are tracked via **git LFS**
  (`.gitattributes`: `*.pdf filter=lfs`). Run `git lfs install` / `git lfs pull` to
  fetch them.
- Opcode definitions: `files/instructions/gb_instructions.json`, following the
  [gbdev.io Opcodes.json](https://gbdev.io/gb-opcodes/Opcodes.json) format.
