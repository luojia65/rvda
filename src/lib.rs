#![allow(unused)]

const OPCODE_LOAD: u32 =     0b000_0011; //
const OPCODE_MISC_MEM: u32 = 0b000_1111;
const OPCODE_OP_IMM: u32 =   0b001_0011; //
const OPCODE_AUIPC: u32 =    0b001_0111; //
const OPCODE_STORE: u32 =    0b010_0011; // 
const OPCODE_OP: u32 =       0b011_0011; //
const OPCODE_LUI: u32 =      0b011_0111; //
const OPCODE_BRANCH: u32 =   0b110_0011; //
const OPCODE_JALR: u32 =     0b110_0111; //
const OPCODE_JAL: u32 =      0b110_1111; //
const OPCODE_SYSTEM: u32 =   0b111_0011;

const FUNCT3_OP_ADD_SUB: u32 = 0b000;
const FUNCT3_OP_SLL: u32   = 0b001;
const FUNCT3_OP_SLT: u32   = 0b010;
const FUNCT3_OP_SLTU: u32  = 0b011;
const FUNCT3_OP_XOR: u32   = 0b100;
const FUNCT3_OP_SRL_SRA: u32 = 0b101;
const FUNCT3_OP_OR: u32    = 0b110;
const FUNCT3_OP_AND: u32   = 0b111;

const FUNCT3_BRANCH_BEQ: u32 = 0b000;
const FUNCT3_BRANCH_BNE: u32 = 0b001;
const FUNCT3_BRANCH_BLT: u32 = 0b100;
const FUNCT3_BRANCH_BGE: u32 = 0b101;
const FUNCT3_BRANCH_BLTU: u32 = 0b110;
const FUNCT3_BRANCH_BGEU: u32 = 0b111;

const FUNCT3_STORE_SB: u32 = 0b000;
const FUNCT3_STORE_SH: u32 = 0b001;
const FUNCT3_STORE_SW: u32 = 0b010;

const FUNCT3_LOAD_LB: u32 = 0b000;
const FUNCT3_LOAD_LH: u32 = 0b001;
const FUNCT3_LOAD_LW: u32 = 0b010;
const FUNCT3_LOAD_LBU: u32 = 0b100;
const FUNCT3_LOAD_LHU: u32 = 0b101;

const FUNCT3_SYSTEM_CSRRW: u32 = 0b001;
const FUNCT3_SYSTEM_CSRRS: u32 = 0b010;
const FUNCT3_SYSTEM_CSRRC: u32 = 0b011;
const FUNCT3_SYSTEM_CSRRWI: u32 = 0b101;
const FUNCT3_SYSTEM_CSRRSI: u32 = 0b110;
const FUNCT3_SYSTEM_CSRRCI: u32 = 0b111;
const FUNCT3_SYSTEM_ECALL_EBREAK: u32 = 0b000;

pub fn dump<I: Input>(mut input: I) -> std::io::Result<()> {
    loop {
        let ins = match input.read_u16() {
            Ok(ins) => ins,
            Err(_err) => {/*println!("{:?}", err);*/ return Ok(())},
        };
        if ins & 0b11 != 0b11 {
            dump_u16(ins);
            continue;
        }
        if ins & 0b11100 != 0b11100 {
            let ins = (ins as u32) + ((input.read_u16()? as u32) << 16);
            dump_u32(ins);
            continue;
        }
        if ins & 0b100000 == 0 {
            let ins = (ins as u64) 
                + ((input.read_u16()? as u64) << 16)
                + ((input.read_u16()? as u64) << 32);
            dump_u48(ins);
            continue;
        }
        if ins & 0b1000000 == 0 {
            let ins = (ins as u64) 
                + ((input.read_u16()? as u64) << 16) 
                + ((input.read_u32()? as u64) << 32);
            dump_u64(ins);
            continue;
        }
        let bits = (ins & 0b01110000_00000000) >> 12;
        if bits != 0b0111 {
            let mut buf = Vec::new();
            buf.push(ins);
            for _ in 0..(4 + 2 * bits) {
                buf.push(input.read_u16()?)
            }
            dump_u80_u176(&buf);
            continue;
        }
        dump_u192_reserved()
    }
}

fn dump_u16(src: u16) {
    println!("u16: 0x{:04X}", src);
}

fn dump_u32(src: u32) {
    print!("u32: 0x{:08X}\t", src);
    let opcode = src & 0b111_1111;
    let rd = format!("x{}", (src >> 7) & 0b1_1111);
    let rs1 = format!("x{}", (src >> 15) & 0b1_1111);
    let rs2 = format!("x{}", (src >> 20) & 0b1_1111);
    let funct3 = (src >> 12) & 0b111;
    let shamt = (src >> 20) & 0b1_1111;
    let imm110 = (src >> 20) & 0b111_1111_1111;
    let funct7 = (src >> 25) & 0b111_1111;
    let imm3112 = (src >> 12) & 0b1111_1111_1111_1111_1111;
    let imm11540 = ((src >> 7) & 0b11111) | ((src >> 25) & 0b1111111);
    let imm20101111912 = {
        let val = (((src >> 21) & 0b11_1111_1111) | 
        (src >> 20) & 0b1 | (src >> 12) & 0b1111_1111) as i32;
        2 * if (src >> 30) > 0 { -(val + 1) } else { val }
    };
    let imm121054111 = {
        let val = (((src >> 8) & 0b1111) | ((src >> 25) & 0b111111) |
        ((src >> 7) & 0b1)) as i32;
        2 * if (src >> 30) > 0 { -(val + 1) } else { val }
    };
    match opcode {
        OPCODE_LUI => {
            println!("lui {}, #{}", rd, imm3112);
        },
        OPCODE_AUIPC => {
            println!("auipc {}, #{}", rd, imm3112);
        },
        OPCODE_JAL => {
            println!("jal {}, #{}", rd, imm20101111912);
        },
        OPCODE_JALR => {
            println!("jalr {}, {}, #{}", rd, rs1, imm110);
        },
        OPCODE_SYSTEM => {
            let name = match funct3 {
                FUNCT3_SYSTEM_CSRRW => "csrrw",
                FUNCT3_SYSTEM_CSRRS => "csrrs",
                FUNCT3_SYSTEM_CSRRC => "csrrc",
                FUNCT3_SYSTEM_CSRRWI => "csrrwi",
                FUNCT3_SYSTEM_CSRRSI => "csrrsi",
                FUNCT3_SYSTEM_CSRRCI => "csrrci",
                FUNCT3_SYSTEM_ECALL_EBREAK => match imm110 { 
                    0 => "ecall",
                    1 => "ebreak",
                    _ => {""} ,
                }
                _ => unreachable!(),
            };
            if funct3 == FUNCT3_SYSTEM_ECALL_EBREAK {
                println!("{}", name);
            } else {
                println!("{} {}, {}, {}", name, rd, rs1, imm110);
            }
        }
        OPCODE_STORE => {
            let name = match funct3 {
                FUNCT3_STORE_SB => "sb",
                FUNCT3_STORE_SH => "sh",
                FUNCT3_STORE_SW => "sw",
                _ => unreachable!(),
            };
            println!("{} {}, {}, #{}", name, rs1, rs2, imm11540);
        },
        OPCODE_LOAD => {
            let name = match funct3 {
                FUNCT3_LOAD_LB => "lb",
                FUNCT3_LOAD_LH => "lh",
                FUNCT3_LOAD_LW => "lw",
                FUNCT3_LOAD_LBU => "lbu",
                FUNCT3_LOAD_LHU => "lhu",
                _ => unreachable!(),
            };
            println!("{} {}, {}, #{}", name, rd, rs1, imm110);
        },
        OPCODE_BRANCH => {
            let name = match funct3 {
                FUNCT3_BRANCH_BEQ => "beq", 
                FUNCT3_BRANCH_BNE => "bne", 
                FUNCT3_BRANCH_BLT => "blt", 
                FUNCT3_BRANCH_BGE => "bge", 
                FUNCT3_BRANCH_BLTU => "bltu", 
                FUNCT3_BRANCH_BGEU => "bgeu",
                _ => unreachable!(), 
            };
            println!("{} {}, {}, #{}", name, rs1, rs2, imm121054111);
        },
        OPCODE_OP_IMM => {
            let name = match funct3 {
                FUNCT3_OP_ADD_SUB => "addi",
                FUNCT3_OP_SLT => "slti",
                FUNCT3_OP_SLTU => "sltiu",
                FUNCT3_OP_XOR => "xori",
                FUNCT3_OP_OR => "ori",
                FUNCT3_OP_AND => "andi",
                _ => match (funct3, funct7) {
                    (FUNCT3_OP_SLL, 0) => "slli",
                    (FUNCT3_OP_SRL_SRA, 0) => "srli",
                    (FUNCT3_OP_SRL_SRA, 0b100000) => "srai",
                    _ => unreachable!() // malformed
                }
            };
            match funct3 {
                FUNCT3_OP_ADD_SUB | FUNCT3_OP_SLT | FUNCT3_OP_SLTU | 
                FUNCT3_OP_XOR | FUNCT3_OP_OR | FUNCT3_OP_AND => {
                    println!("{} {}, {}, #{}", name, rd, rs1, imm110);
                },
                _ => {
                    println!("{} {}, {}, {}", name, rd, rs1, shamt);
                }
            }
        },
        OPCODE_OP => {
            let name = match funct3 {
                FUNCT3_OP_SLT => "slt",
                FUNCT3_OP_SLTU => "sltu",
                FUNCT3_OP_XOR => "xor",
                FUNCT3_OP_OR => "or",
                FUNCT3_OP_AND => "and",
                _ => match (funct3, funct7) {
                    (FUNCT3_OP_ADD_SUB, 0) => "add",
                    (FUNCT3_OP_ADD_SUB, 0b100000) => "sub",
                    (FUNCT3_OP_SLL, 0) => "sll",
                    (FUNCT3_OP_SRL_SRA, 0) => "srl",
                    (FUNCT3_OP_SRL_SRA, 0b100000) => "sra",
                    _ => unreachable!() // malformed
                }
            };
            println!("{} {}, {}, {}", name, rd, rs1, rs2);
        }
        _ => println!()
    }
}

fn dump_u48(src: u64) {
    println!("u48: 0x{:016X}", src);
}

fn dump_u64(src: u64) {
    println!("u64: 0x{:032X}", src);
}

fn dump_u80_u176(src: &[u16]) {
    print!("u{}: 0x", src.len()*16);
    for word in src {
        print!("{:02X}", word);
    }
    println!();
}

fn dump_u192_reserved() {
    println!("Reserved!")
}

pub trait Input {

    fn read_u8(&mut self) -> std::io::Result<u8>;

    fn read_u16(&mut self) -> std::io::Result<u16>;

    fn read_u32(&mut self) -> std::io::Result<u32>;
}

use std::io::Read;
use std::io::{Error, ErrorKind};
impl<T> Input for std::io::Cursor<T>
where T: AsRef<[u8]> {
    fn read_u8(&mut self) -> std::io::Result<u8> {
        let mut buf = [0u8];
        let size = self.read(&mut buf)?;
        if size < 1 {
            return Err(Error::from(ErrorKind::UnexpectedEof));
        }
        Ok(buf[0])
    }

    fn read_u16(&mut self) -> std::io::Result<u16> {
        let mut buf = [0u8, 0];
        let size = self.read(&mut buf)?;
        if size < 2 {
            return Err(Error::from(ErrorKind::UnexpectedEof));
        }
        Ok(u16::from_ne_bytes(buf))
    }

    fn read_u32(&mut self) -> std::io::Result<u32> {
        let mut buf = [0u8, 0, 0, 0];
        let size = self.read(&mut buf)?;
        if size < 4 {
            return Err(Error::from(ErrorKind::UnexpectedEof));
        }
        Ok(u32::from_ne_bytes(buf))
    }
}
