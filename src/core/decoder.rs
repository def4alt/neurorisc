use crate::core::instructions::{DecodedInstruction, Instruction};
use anyhow::anyhow;

pub fn decode(instruction: u32) -> anyhow::Result<DecodedInstruction> {
    let op = (instruction & 0x7f) as u8;

    match op {
        0b0110011 => {
            let rd = ((instruction >> 7) & 0x1f) as u8;
            let funct3 = ((instruction >> 12) & 0x07) as u8;
            let rs1 = ((instruction >> 15) & 0x1f) as u8;
            let rs2 = ((instruction >> 20) & 0x1f) as u8;
            let funct7 = ((instruction >> 25) & 0x7f) as u8;

            Ok(DecodedInstruction::R {
                op,
                rd,
                funct3,
                rs1,
                rs2,
                funct7,
            })
        }
        _ => Err(anyhow!("Unsupported instruction")),
    }
}

pub fn resolve(decoded: DecodedInstruction) -> anyhow::Result<Instruction> {
    match decoded {
        DecodedInstruction::R {
            rd,
            funct3,
            rs1,
            rs2,
            funct7,
            ..
        } => match (funct3, funct7) {
            (0b000, 0b0000000) => Ok(Instruction::Add { rd, rs1, rs2 }),
            (0b000, 0b0100000) => Ok(Instruction::Sub { rd, rs1, rs2 }),
            (0b001, 0b0000000) => Ok(Instruction::Sll { rd, rs1, rs2 }),
            (0b010, 0b0000000) => Ok(Instruction::Slt { rd, rs1, rs2 }),
            (0b011, 0b0000000) => Ok(Instruction::Sltu { rd, rs1, rs2 }),
            (0b100, 0b0000000) => Ok(Instruction::Xor { rd, rs1, rs2 }),
            (0b101, 0b0000000) => Ok(Instruction::Srl { rd, rs1, rs2 }),
            (0b101, 0b0100000) => Ok(Instruction::Sra { rd, rs1, rs2 }),
            (0b110, 0b0000000) => Ok(Instruction::Or { rd, rs1, rs2 }),
            (0b111, 0b0000000) => Ok(Instruction::And { rd, rs1, rs2 }),
            _ => Err(anyhow!("Unsupported instruction")),
        },
        _ => Err(anyhow!("Unsupported instruction")),
    }
}
