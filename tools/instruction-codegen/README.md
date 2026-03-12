# Instruction-Codegen

This tool generates the instruction tables and trait for the emulator.

## Usage

To run the tool, use the following command:

```bash
cargo run -p instruction-codegen -- --input <INPUT_FILE> --output <OUTPUT_FILE>
```

### Arguments

- `--input <INPUT_FILE>`: The path to the input file containing the instruction definitions.
- `--output <OUTPUT_FILE>`: The path to the output file where the generated code will be generated.

## Input File Format

The input file should be a JSON file containing an array of instruction definitions following the [gbdev.io](https://gbdev.io/gb-opcodes/Opcodes.json) format.
