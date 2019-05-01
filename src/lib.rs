const OPCODE_LOAD: u32 =     0b000_0011; 
const OPCODE_MISC_MEM: u32 = 0b000_1111;
const OPCODE_OP_IMM: u32 =   0b001_0011; 
const OPCODE_AUIPC: u32 =    0b001_0111; 
const OPCODE_STORE: u32 =    0b010_0011; 
const OPCODE_OP: u32 =       0b011_0011; 
const OPCODE_LUI: u32 =      0b011_0111; 
const OPCODE_BRANCH: u32 =   0b110_0011; 
const OPCODE_JALR: u32 =     0b110_0111; 
const OPCODE_JAL: u32 =      0b110_1111;
const OPCODE_SYSTEM: u32 =   0b111_0011; 

const OPCODE_C0: u16 =  0b00;
const OPCODE_C1: u16 =  0b01;
const OPCODE_C2: u16 =  0b10;

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

const FUNCT3_SYSTEM_PRIV: u32 = 0b000;
const FUNCT3_SYSTEM_CSRRW: u32 = 0b001;
const FUNCT3_SYSTEM_CSRRS: u32 = 0b010;
const FUNCT3_SYSTEM_CSRRC: u32 = 0b011;
const FUNCT3_SYSTEM_CSRRWI: u32 = 0b101;
const FUNCT3_SYSTEM_CSRRSI: u32 = 0b110;
const FUNCT3_SYSTEM_CSRRCI: u32 = 0b111;

pub fn dump<I: Input>(input: &mut I) -> std::io::Result<()> {
    let mut ptr = 0;
    loop {
        match dump0(input, &mut ptr) {
            Ok(()) => continue,
            Err(ref e) if e.kind() == ErrorKind::UnexpectedEof => return Ok(()),
            Err(e) => return Err(e),
        };
    }
}

fn dump0<I: Input>(input: &mut I, ptr: &mut usize) -> std::io::Result<()> {
    let ins = input.read_u16()?;
    if ins & 0b11 != 0b11 {
        dump_u16(ins, *ptr);
        *ptr += 2;
        return Ok(());
    }
    if ins & 0b11100 != 0b11100 {
        let ins = (ins as u32) + ((input.read_u16()? as u32) << 16);
        dump_u32(ins, *ptr);
        *ptr += 4;
        return Ok(());
    }
    if ins & 0b100000 == 0 {
        let ins = (ins as u64) 
            + ((input.read_u16()? as u64) << 16)
            + ((input.read_u16()? as u64) << 32);
        dump_u48(ins, *ptr);
        *ptr += 6;
        return Ok(());
    }
    if ins & 0b1000000 == 0 {
        let ins = (ins as u64) 
            + ((input.read_u16()? as u64) << 16) 
            + ((input.read_u32()? as u64) << 32);
        dump_u64(ins, *ptr);
        *ptr += 8;
        return Ok(());
    }
    let bits = (ins & 0b01110000_00000000) >> 12;
    if bits != 0b0111 {
        let mut buf = Vec::new();
        buf.push(ins);
        for _ in 0..(4 + 2 * bits) {
            buf.push(input.read_u16()?)
        }
        dump_u80_u176(&buf, *ptr);
        *ptr += ((4 + 2 * bits) * 2) as usize;
        return Ok(());
    }
    dump_u192_reserved();
    Ok(())
}

fn dump_u16(src: u16, ptr: usize) {
    print!("{:x}:\t{:04x}\t\t", ptr, src);
    let opcode = src & 0b11;
    let inst1513 = (src >> 13) & 0b111;
    let nzimm540 = {
        let res = (src >> 2) as i16 & 0b11111;
        if (src >> 12) & 0b1 > 0 { -(res + 1) } else { res }
    };
    let nzimm946875 = {
        let res = (
            (((src >> 6) & 0b1) << 4) | (((src >> 5) & 0b1) << 6) |
            (((src >> 3) & 0b11) << 7) | (((src >> 2) & 0b1) << 5)
        ) as i16;
        if (src >> 12) & 0b1 > 0 { -(res + 1) } else { res }
    };
    let nzuimm171612 = {
        let src = src as u32;
        (((src >> 12) & 0b1) << 17) | (((src >> 2) & 0b11111) << 12)
    };
    let nzuimm540 = ((src >> 2) & 0b11111) | (((src >> 12) & 0b1) << 5);
    let imm540 = ((src >> 2) & 0b11111) | (((src >> 12) & 0b1) << 5);
    let imm114981067315 = 
        (((src >> 3) & 0b11) << 1) | (((src >> 11) & 0b1) << 3) | 
        (((src >> 2) & 0b1) << 4) | (((src >> 7) & 0b1) << 5) | 
        (((src >> 6) & 0b1) << 6) | (((src >> 9) & 0b11) << 8) | 
        (((src >> 8) & 0b1) << 9) | (((src >> 11) & 0b1) << 10);
    let imm84376215 = (((src >> 12) & 0b1) << 8) | (((src >> 10) & 0b11) << 3) |
        (((src >> 5) & 0b11) << 6) | (((src >> 3) & 0b11) << 1) |
        (((src >> 2) & 0b11) << 5);
    let rd_c = format!("x{}", 8 + ((src >> 2) & 0b111)); 
    let rs1_c = format!("x{}", 8 + ((src >> 7) & 0b111)); 
    let rs2_c = format!("x{}", 8 + ((src >> 2) & 0b111)); 
    let rd = format!("x{}", (src >> 7) & 0b1_1111);
    let rs1 = format!("x{}", (src >> 7) & 0b1_1111);
    let rs2 = format!("x{}", (src >> 2) & 0b1_1111);
    let uimm5276 = (((src >> 9) & 0b1111) << 2) | (((src >> 7) & 0b11) << 6);
    let uimm5376 = (((src >> 10) & 0b111) << 3) | (((src >> 5) & 0b11) << 6);
    let uimm5326 = (((src >> 11) & 0b111) << 3) | (((src >> 5) & 0b1) << 6) | 
        (((src >> 6) & 0b1) << 2);
    let uimm54386 = (((src >> 12) & 0b1) << 5) | (((src >> 5) & 0b11) << 3) |
        (((src >> 2) & 0b111) << 6);
    let uimm54276 = (((src >> 12) & 0b1) << 5) | (((src >> 4) & 0b111) << 2) | 
        (((src >> 2) & 0b111) << 6);
    let uimm5386 = (((src >> 10) & 0b111) << 3) | (((src >> 7) & 0b111) << 6);
    let nzuimm549623 = (((src >> 11) & 0b11) << 4) | (((src >> 7) & 0b1111) << 6) |
        (((src >> 6) & 0b1) << 2) | (((src >> 5) & 0b1) << 3);
    // print!(" {:02b}  {:03b}  ", opcode, inst1513);
    match (opcode, inst1513) {
        (OPCODE_C0, 0b000) if nzuimm549623 != 0 => {
            println!("c.addi4spn {}, #0x{:03X} ; {}", rd_c, nzuimm549623, nzuimm549623);
        },
        (OPCODE_C0, 0b000) => println!("!! illegal"),
        (OPCODE_C0, 0b001) => {
            println!("c.fld {}, {}, #0x{:02X} ; {}", rd_c, rs1_c, uimm5376, uimm5376);
        },
        (OPCODE_C0, 0b010) => {
            println!("c.lw {}, {}, #0x{:02X} ; {}", rd_c, rs1_c, uimm5326, uimm5326);
        },
        (OPCODE_C0, 0b011) => {
            println!("c.flw {}, {}, #0x{:02X} ; {}", rd_c, rs1_c, uimm5326, uimm5326);
        },
        (OPCODE_C0, 0b100) => println!("!! reserved"),
        (OPCODE_C0, 0b101) => {
            println!("c.fsd {}, {}, #0x{:02X} ; {}", rd_c, rs1_c, uimm5376, uimm5376);
        },
        (OPCODE_C0, 0b110) => {
            println!("c.sw {}, {}, #0x{:02X} ; {}", rd_c, rs1_c, uimm5326, uimm5326);
        },
        (OPCODE_C0, 0b111) => {
            println!("c.fsw {}, {}, #0x{:02X} ; {}", rd_c, rs1_c, uimm5326, uimm5326);
        },
        (OPCODE_C0, _) => unreachable!(),
        (OPCODE_C1, 0b000) => {
            println!("c.addi {}, #0x{:02X}; {}", rd, nzimm540, nzimm540);
        },
        (OPCODE_C1, 0b001) => {
            println!("c.jal #0x{:03X} ; {}", imm114981067315, imm114981067315);
        },
        (OPCODE_C1, 0b010) => {
            println!("c.li {}, #0x{:02X} ; {}", rd, imm540, imm540);
        },
        (OPCODE_C1, 0b011) if ((src >> 7) & 0b1_1111) == 2 => {
            println!("c.addi16sp #0x{:03X} ; {}", nzimm946875, nzimm946875);
        },
        (OPCODE_C1, 0b011) => {
            println!("c.lui {}, #0x{:05X} ; {}", rd, nzuimm171612, nzuimm171612);
        },
        (OPCODE_C1, 0b100) => {
            let inst1110 = (src >> 10) & 0b11;
            let inst65 = (src >> 5) & 0b11;
            match inst1110 {
                0b10 => println!("c.andi {}, #0x{:03X} ; {}", rd_c, imm540, imm540),
                0b11 => {
                    let name = match inst65 {
                        0b00 => "c.sub",
                        0b01 => "c.xor",
                        0b10 => "c.or",
                        0b11 => "c.and",
                        _ => unreachable!()
                    };
                    println!("{} {}, {}", name, rd_c, rs2_c);
                },
                _ => {},
            };
        },
        (OPCODE_C1, 0b101) => {
            println!("c.j #0x{:03X} ; {}", imm114981067315, imm114981067315);
        },
        (OPCODE_C1, 0b110) => {
            println!("c.beqz {}, #0x{:03X} ; {}", rs1_c, imm84376215, imm84376215);
        },
        (OPCODE_C1, 0b111) => {
            println!("c.bnez {}, #0x{:03X} ; {}", rs1_c, imm84376215, imm84376215);
        },
        (OPCODE_C1, _) => unreachable!(),
        (OPCODE_C2, 0b000) if nzuimm540 == 0 => {
            println!("c.slli64 {}", rd);
        },
        (OPCODE_C2, 0b000) => {
            println!("c.slli {}, #0x{:02X} ; {}", rd, nzuimm540, nzuimm540);
        },
        (OPCODE_C2, 0b001) => {
            println!("c.fldsp {}, #0x{:02X} ; {}", rd, uimm54386, uimm54386);
        },
        (OPCODE_C2, 0b010) => {
            println!("c.lwsp {}, #0x{:02X} ; {}", rd, uimm54276, uimm54276);
        },
        (OPCODE_C2, 0b011) => {
            println!("c.flwsp {}, #0x{:02X} ; {}", rd, uimm54276, uimm54276);
        },
        (OPCODE_C2, 0b100) => {
            let b12 = (src >> 12) & 0b1;
            match (b12, (src >> 7) & 0b1_1111, (src >> 2) & 0b1_1111) {
                (0, _, 0) => println!("c.jr {}", rs1),
                (0, _, _) => println!("c.mv {}, {}", rd, rs2),
                (1, 0, 0) => println!("c.ebreak"),
                (1, _, 0) => println!("c.jalr {}", rs1),
                (1, _, _) => println!("c.add {}, {}", rd, rs2),
                (_, _, _) => unreachable!()
            } 
        },
        (OPCODE_C2, 0b101) => {
            println!("c.fsdsp {}, #0x{:03X} ; {}", rd, uimm5386, uimm5386);
        },
        (OPCODE_C2, 0b110) => {
            println!("c.swsp {}, #0x{:02X}; {}", rs2, uimm5276, uimm5276);
        },
        (OPCODE_C2, 0b111) => {
            println!("c.fswsp {}, #0x{:03X}; {}", rs2, uimm5386, uimm5386);
        },
        (OPCODE_C2, _) => {println!()},
        _ => unreachable!()
    }
}

fn dump_u32(src: u32, ptr: usize) {
    print!("{:x}:\t{:08x}\t", ptr, src);
    let opcode = src & 0b111_1111;
    let rd = format!("x{}", (src >> 7) & 0b1_1111);
    let rs1 = format!("x{}", (src >> 15) & 0b1_1111);
    let rs2 = format!("x{}", (src >> 20) & 0b1_1111);
    let funct3 = (src >> 12) & 0b111;
    let shamt = (src >> 20) & 0b1_1111;
    let imm110 = (src >> 20) & 0b1111_1111_1111;
    let funct7 = (src >> 25) & 0b111_1111;
    let imm3112 = ((src >> 12) & 0b1111_1111_1111_1111_1111) << 12;
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
            println!("lui {}, #0x{:08X} ; {}", rd, imm3112, imm3112);
        },
        OPCODE_AUIPC => {
            println!("auipc {}, #0x{:08X}; {}", rd, imm3112, imm3112);
        },
        OPCODE_JAL => {
            println!("jal {}, #0x{:08X} ; {}", rd, imm20101111912, imm20101111912);
        },
        OPCODE_JALR => {
            println!("jalr {}, {}, #0x{:03X} ; {}", rd, rs1, imm110, imm110);
        },
        OPCODE_SYSTEM => {
            let name = match funct3 {
                FUNCT3_SYSTEM_CSRRW => "csrrw",
                FUNCT3_SYSTEM_CSRRS => "csrrs",
                FUNCT3_SYSTEM_CSRRC => "csrrc",
                FUNCT3_SYSTEM_CSRRWI => "csrrwi",
                FUNCT3_SYSTEM_CSRRSI => "csrrsi",
                FUNCT3_SYSTEM_CSRRCI => "csrrci",
                FUNCT3_SYSTEM_PRIV => match imm110 { 
                    0 => "ecall",
                    1 => "ebreak",
                    0b0000000_00010 => "uret",
                    0b0001000_00010 => "sret",
                    0b0011000_00010 => "mret",
                    0b0001000_00101 => "wfi",
                    _ => unreachable!(),
                }
                _ => unreachable!(),
            };
            if funct3 == FUNCT3_SYSTEM_PRIV {
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
                _ => { " " }, 
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
                    println!("{} {}, {}, #0x{:03X} ; {}", name, rd, rs1, imm110, imm110);
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
        },
        OPCODE_MISC_MEM => {
            match funct3 {
                0 => {
                    print!("fence ");
                    if src & (1 << 20) > 0 { print!("sw, ") };
                    if src & (1 << 21) > 0 { print!("sr, ") };
                    if src & (1 << 22) > 0 { print!("so, ") };
                    if src & (1 << 23) > 0 { print!("si, ") };
                    if src & (1 << 24) > 0 { print!("pw, ") };
                    if src & (1 << 25) > 0 { print!("pr, ") };
                    if src & (1 << 26) > 0 { print!("po, ") };
                    if src & (1 << 27) > 0 { print!("pi, ") };
                    println!()
                },
                1 => {
                    println!("fence.i");
                },
                _ => unreachable!()
            }
        },
        _ => println!()
    }
}

fn dump_u48(src: u64, ptr: usize) {
    println!("{:x}:\t{:016x}\t", ptr, src);
}

fn dump_u64(src: u64, ptr: usize) {
    println!("{:x}:\t{:032x}\t", ptr, src);
}

fn dump_u80_u176(src: &[u16], ptr: usize) {
    print!("{:x}:\t0x", ptr);
    for word in src {
        print!("{:02x}", word);
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
