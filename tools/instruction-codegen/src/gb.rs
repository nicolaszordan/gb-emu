use std::collections::BTreeMap;
use std::error::Error;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Default)]
struct Operand {
    name: String,
    #[serde(default)]
    bytes: Option<u8>,
    #[serde(default)]
    increment: Option<bool>,
    #[serde(default)]
    decrement: Option<bool>,
    immediate: bool,
}

#[derive(Debug, Deserialize, Serialize, Default)]
struct Instruction {
    mnemonic: String,
    bytes: u8,
    cycles: Vec<u8>,
    operands: Vec<Operand>,
    immediate: bool,
    flags: Flags,
}

#[derive(Debug, Deserialize, Serialize, Default)]
struct Flags {
    #[serde(rename = "Z")]
    z: String,
    #[serde(rename = "N")]
    n: String,
    #[serde(rename = "H")]
    h: String,
    #[serde(rename = "C")]
    c: String,
}

type InstructionMap = BTreeMap<String, Instruction>;

/// Top-level structure for the input JSON file, containing both unprefixed and CB-prefixed instructions
#[derive(Debug, Deserialize, Serialize, Default)]
struct InstructionMaps {
    unprefixed: InstructionMap,
    cbprefixed: InstructionMap,
}

const TRAIT_NAME: &str = "InstructionHandler";
const UNPREFIXED_TABLE_NAME: &str = "UNPREFIXED_INSTRUCTIONS";
const CBPREFIXED_TABLE_NAME: &str = "CBPREFIXED_INSTRUCTIONS";

pub fn generate_code(input: &str) -> Result<String, Box<dyn Error>> {
    let instructions: InstructionMaps = serde_json::from_str(&input)?;

    let mut output = String::new();

    output.push_str("// Auto-generated file - DO NOT EDIT\n");
    output.push_str("// Use `tools/instruction-codegen` to edit\n");

    output.push_str("\n/// Auto-generated trait containing all GameBoy CPU instructions\n");
    output.push_str(&generate_trait(&instructions));
    output.push_str("\n/// Metadata for a single CPU instruction\n");
    output.push_str(&generate_instruction_metadata_struct());
    output.push_str(&generate_instruction_tables(&instructions));

    Ok(output)
}

fn generate_trait(instructions: &InstructionMaps) -> String {
    let mut output = String::new();

    output.push_str(&format!("pub trait {} {{\n", TRAIT_NAME));
    output.push_str(&generate_trait_instruction_map(&instructions.unprefixed));
    output.push_str(&generate_trait_instruction_map(&instructions.cbprefixed));
    output.push_str("}\n");

    output
}

fn generate_trait_instruction_map(instruction_map: &InstructionMap) -> String {
    let mut output = String::new();

    for i in 0..instruction_map.len() {
        if i % 16 == 0 {
            output.push_str(&format!("\n    // 0x{:02X}-0x{:02X}\n", i, i + 15));
        }

        let opcode_key = format!("0x{:02X}", i);

        if let Some(instr) = instruction_map.get(&opcode_key) {
            let method_name = generate_trait_method_name(instr);
            output.push_str(&format!("    fn {}(&mut self);\n", method_name));
        } else {
            panic!("Missing instruction for opcode 0x{:02X}", i);
        }
    }

    output
}

fn generate_trait_method_name(instr: &Instruction) -> String {
    let mut name = instr.mnemonic.to_lowercase();

    for operand in &instr.operands {
        name.push_str(&generate_trait_method_operand_name(operand));
    }

    name
}

fn generate_trait_method_operand_name(operand: &Operand) -> String {
    let mut operand_name = String::new();

    operand_name.push('_');

    // For memory operands, prefix with 'm' to avoid conflicts with register names
    if !operand.immediate {
        operand_name.push('m');
    }

    let name = operand.name.replace("$", "addr");

    operand_name.push_str(&name);

    // For HL+/HL- instructions, append inc/dec to the operand name
    if operand.increment.unwrap_or(false) {
        operand_name.push_str("inc");
    } else if operand.decrement.unwrap_or(false) {
        operand_name.push_str("dec");
    }

    operand_name
}

fn generate_instruction_tables(instructions: &InstructionMaps) -> String {
    let mut output = String::new();

    output.push_str(&generate_instruction_table(
        UNPREFIXED_TABLE_NAME,
        &instructions.unprefixed,
    ));

    output.push_str(&generate_instruction_table(
        CBPREFIXED_TABLE_NAME,
        &instructions.cbprefixed,
    ));

    output
}

fn generate_instruction_metadata_struct() -> String {
    let mut output = String::new();

    output.push_str("#[derive(Clone)]\n");
    output.push_str("pub struct InstructionMeta {\n");
    output.push_str("    pub mnemonic: &'static str,\n");
    output.push_str("    pub opcode: u8,\n");
    output.push_str("    pub bytes: u8,\n");
    output.push_str("    pub cycles: &'static [u8],\n");
    output.push_str(&format!("    pub execute: fn(&mut dyn {}),\n", TRAIT_NAME));
    output.push_str("}\n");

    output
}

fn generate_instruction_table(
    table_name: &str,
    instructions: &BTreeMap<String, Instruction>,
) -> String {
    let mut output = String::new();

    output.push_str("\n");
    output.push_str(&format!(
        "pub const {}: [InstructionMeta; {}] = [\n",
        table_name,
        instructions.len()
    ));

    for i in 0..instructions.len() {
        if i % 16 == 0 {
            output.push_str(&format!("\n    // 0x{:02X}-0x{:02X}\n", i, i + 15));
        }

        let opcode_key = format!("0x{:02X}", i);

        if let Some(instr) = instructions.get(&opcode_key) {
            output.push_str(&generate_instruction_table_entry(instr, i as u8));
        } else {
            panic!("Missing instruction for opcode 0x{:02X}", i);
        }
    }

    output.push_str("];\n");

    output
}

fn generate_instruction_table_entry(instr: &Instruction, opcode: u8) -> String {
    let method_name = generate_trait_method_name(instr);
    let cycles_str = instr
        .cycles
        .iter()
        .map(|c| c.to_string())
        .collect::<Vec<_>>()
        .join(", ");

    format!(
        "    InstructionMeta {{\n        mnemonic: \"{}\",\n        opcode: 0x{:02X},\n        bytes: {},\n        cycles: &[{}],\n        execute: |cpu| cpu.{}(),\n    }},\n",
        instr.mnemonic, opcode, instr.bytes, cycles_str, method_name
    )
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_generate_trait_method_operand_name() {
        let operand = Operand {
            name: "A".to_string(),
            bytes: None,
            increment: None,
            decrement: None,
            immediate: false,
        };
        assert_eq!(generate_trait_method_operand_name(&operand), "_mA");

        let operand = Operand {
            name: "n16".to_string(),
            bytes: Some(2),
            increment: None,
            decrement: None,
            immediate: true,
        };
        assert_eq!(generate_trait_method_operand_name(&operand), "_n16");

        let operand = Operand {
            name: "HL".to_string(),
            bytes: None,
            increment: Some(true),
            decrement: None,
            immediate: false,
        };
        assert_eq!(generate_trait_method_operand_name(&operand), "_mHLinc");

        let operand = Operand {
            name: "HL".to_string(),
            bytes: None,
            increment: None,
            decrement: Some(true),
            immediate: false,
        };
        assert_eq!(generate_trait_method_operand_name(&operand), "_mHLdec");

        let operand = Operand {
            name: "$FF00".to_string(),
            bytes: None,
            increment: None,
            decrement: None,
            immediate: true,
        };
        assert_eq!(generate_trait_method_operand_name(&operand), "_addrFF00");
    }

    #[test]
    fn test_generate_trait_method_name() {
        let instr = Instruction {
            mnemonic: "LD".to_string(),
            operands: vec![
                Operand {
                    name: "BC".to_string(),
                    bytes: None,
                    increment: None,
                    decrement: None,
                    immediate: false,
                },
                Operand {
                    name: "n16".to_string(),
                    bytes: Some(2),
                    increment: None,
                    decrement: None,
                    immediate: true,
                },
            ],
            ..Instruction::default()
        };

        let method_name = generate_trait_method_name(&instr);
        assert_eq!(method_name, "ld_mBC_n16");

        let instr = Instruction {
            mnemonic: "NOP".to_string(),
            operands: vec![],
            ..Instruction::default()
        };

        let method_name = generate_trait_method_name(&instr);
        assert_eq!(method_name, "nop");

        let instr = Instruction {
            mnemonic: "LD".to_string(),
            operands: vec![
                Operand {
                    name: "A".to_string(),
                    bytes: None,
                    increment: None,
                    decrement: None,
                    immediate: true,
                },
                Operand {
                    name: "HL".to_string(),
                    bytes: None,
                    increment: Some(true),
                    decrement: None,
                    immediate: false,
                },
            ],
            ..Instruction::default()
        };

        let method_name = generate_trait_method_name(&instr);
        assert_eq!(method_name, "ld_A_mHLinc");
    }

    #[test]
    fn test_generate_trait() {
        let mut instructions = InstructionMaps::default();
        instructions.unprefixed.insert(
            "0x00".to_string(),
            Instruction {
                mnemonic: "NOP".to_string(),
                operands: vec![],
                ..Instruction::default()
            },
        );
        instructions.unprefixed.insert(
            "0x01".to_string(),
            Instruction {
                mnemonic: "LD".to_string(),
                operands: vec![
                    Operand {
                        name: "BC".to_string(),
                        bytes: None,
                        increment: None,
                        decrement: None,
                        immediate: false,
                    },
                    Operand {
                        name: "n16".to_string(),
                        bytes: Some(2),
                        increment: None,
                        decrement: None,
                        immediate: true,
                    },
                ],
                ..Instruction::default()
            },
        );

        let trait_code = generate_trait(&instructions);

        let expected = r#"
pub trait InstructionHandler {

    // 0x00-0x0F
    fn nop(&mut self);
    fn ld_mBC_n16(&mut self);
}
"#;

        assert_eq!(&expected[1..], trait_code);
    }

    #[test]
    fn test_generate_trait_instruction_map() {}

    #[test]
    fn test_generate_instruction_table_entry() {
        let instr = Instruction {
            mnemonic: "LD".to_string(),
            bytes: 3,
            cycles: vec![12],
            operands: vec![
                Operand {
                    name: "BC".to_string(),
                    bytes: None,
                    increment: None,
                    decrement: None,
                    immediate: false,
                },
                Operand {
                    name: "n16".to_string(),
                    bytes: Some(2),
                    increment: None,
                    decrement: None,
                    immediate: true,
                },
            ],
            immediate: true,
            flags: Flags {
                z: "0".to_string(),
                n: "0".to_string(),
                h: "0".to_string(),
                c: "0".to_string(),
            },
        };

        let entry = generate_instruction_table_entry(&instr, 0x01);

        let expected = r#"
    InstructionMeta {
        mnemonic: "LD",
        opcode: 0x01,
        bytes: 3,
        cycles: &[12],
        execute: |cpu| cpu.ld_mBC_n16(),
    },
"#;

        assert_eq!(&expected[1..], entry);
    }

    #[test]
    fn test_generate_instruction_table() {
        let mut instructions = BTreeMap::new();
        instructions.insert(
            "0x00".to_string(),
            Instruction {
                mnemonic: "NOP".to_string(),
                bytes: 1,
                cycles: vec![4],
                operands: vec![],
                immediate: false,
                flags: Flags {
                    z: "0".to_string(),
                    n: "0".to_string(),
                    h: "0".to_string(),
                    c: "0".to_string(),
                },
            },
        );
        instructions.insert(
            "0x01".to_string(),
            Instruction {
                mnemonic: "LD".to_string(),
                bytes: 3,
                cycles: vec![12],
                operands: vec![
                    Operand {
                        name: "BC".to_string(),
                        bytes: None,
                        increment: None,
                        decrement: None,
                        immediate: false,
                    },
                    Operand {
                        name: "n16".to_string(),
                        bytes: Some(2),
                        increment: None,
                        decrement: None,
                        immediate: true,
                    },
                ],
                immediate: true,
                flags: Flags {
                    z: "0".to_string(),
                    n: "0".to_string(),
                    h: "0".to_string(),
                    c: "0".to_string(),
                },
            },
        );

        let table = generate_instruction_table("TEST_TABLE", &instructions);

        let expected = r#"
pub const TEST_TABLE: [InstructionMeta; 2] = [

    // 0x00-0x0F
    InstructionMeta {
        mnemonic: "NOP",
        opcode: 0x00,
        bytes: 1,
        cycles: &[4],
        execute: |cpu| cpu.nop(),
    },
    InstructionMeta {
        mnemonic: "LD",
        opcode: 0x01,
        bytes: 3,
        cycles: &[12],
        execute: |cpu| cpu.ld_mBC_n16(),
    },
];
"#;
        assert_eq!(expected, table);
    }
}
