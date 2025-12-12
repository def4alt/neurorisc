pub enum DecodedInstruction {
    R {
        op: u8,
        rd: u8,
        funct3: u8,
        rs1: u8,
        rs2: u8,
        funct7: u8,
    },
    I {
        op: u8,
        rd: u8,
        funct3: u8,
        rs1: u8,
        imm: i32,
    },
    S {
        op: u8,
        funct3: u8,
        rs1: u8,
        rs2: u8,
        imm: i32,
    },
    B {
        op: u8,
        funct3: u8,
        rs1: u8,
        rs2: u8,
        imm: i32,
    },
    U {
        op: u8,
        rd: u8,
        imm: i32,
    },
    J {
        op: u8,
        rd: u8,
        imm: i32,
    },
}

pub enum Instruction {
    Add { rd: u8, rs1: u8, rs2: u8 },
    Sub { rd: u8, rs1: u8, rs2: u8 },
    Sll { rd: u8, rs1: u8, rs2: u8 },
    Slt { rd: u8, rs1: u8, rs2: u8 },
    Sltu { rd: u8, rs1: u8, rs2: u8 },
    Xor { rd: u8, rs1: u8, rs2: u8 },
    Srl { rd: u8, rs1: u8, rs2: u8 },
    Sra { rd: u8, rs1: u8, rs2: u8 },
    Or { rd: u8, rs1: u8, rs2: u8 },
    And { rd: u8, rs1: u8, rs2: u8 },
}
