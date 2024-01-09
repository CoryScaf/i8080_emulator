use std::env;
use std::fs;
use std::io;

#[derive(Debug)]
enum RegisterSymbols {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    SP,
    PSW,
    MEMORY,
}

struct I8080Flags {
    zero: bool,
    sign: bool,
    parity: bool,
    carry: bool,
    auxiliary_carry: bool,
}

struct I8080State {
    reg_a: u8,
    reg_b: u8,
    reg_c: u8,
    reg_d: u8,
    reg_e: u8,
    reg_h: u8,
    reg_l: u8,
    stack_pointer: u16,
    program_counter: u16,
    flags: I8080Flags,
    memory: Vec<u8>,
}

fn disassemble8080_op(code_buffer: &[u8], program_counter: usize) -> usize {
    let mut op_bytes = 1;
    print!("{:04x} ", program_counter);
    match code_buffer[program_counter] {
        0x00 | 0x08 | 0x10 | 0x18 | 0x20 | 0x28 | 0x30 | 0x38 | 0xcb | 0xd9 | 0xdd | 0xed
        | 0xfd => print!("NOP"),
        0x01 => {
            print!(
                "LXI    B,#${:02x}{:02x}",
                code_buffer[program_counter + 2],
                code_buffer[program_counter + 1]
            );
            op_bytes = 3
        }
        0x02 => print!("STAX   B"),
        0x03 => print!("INX    B"),
        0x04 => print!("INR    B"),
        0x05 => print!("DCR    B"),
        0x06 => {
            print!("MVI    B,#${:02x}", code_buffer[program_counter + 1]);
            op_bytes = 2
        }
        0x07 => print!("RLC"),
        0x09 => print!("DAD    B"),
        0x0a => print!("LDAX   B"),
        0x0b => print!("DCX    B"),
        0x0c => print!("INR    C"),
        0x0d => print!("DCR    C"),
        0x0e => {
            print!("MVI    C,#${:02x}", code_buffer[program_counter + 1]);
            op_bytes = 2
        }
        0x0f => print!("RRC"),
        0x11 => {
            print!(
                "LXI    D,#${:02x}{:02x}",
                code_buffer[program_counter + 2],
                code_buffer[program_counter + 1]
            );
            op_bytes = 3
        }
        0x12 => print!("STAX   D"),
        0x13 => print!("INX    D"),
        0x14 => print!("INR    D"),
        0x15 => print!("DCR    D"),
        0x16 => {
            print!("MVI    D,#${:02x}", code_buffer[program_counter + 1]);
            op_bytes = 2
        }
        0x17 => print!("RAL"),
        0x19 => print!("DAD    D"),
        0x1a => print!("LDAX   D"),
        0x1b => print!("DCX    D"),
        0x1c => print!("INR    E"),
        0x1d => print!("DCR    E"),
        0x1e => {
            print!("MVI    E,#${:02x}", code_buffer[program_counter + 1]);
            op_bytes = 2
        }
        0x1f => print!("RAR"),
        0x21 => {
            print!(
                "LXI    H,#${:02x}{:02x}",
                code_buffer[program_counter + 2],
                code_buffer[program_counter + 1]
            );
            op_bytes = 3
        }
        0x22 => {
            print!(
                "SHLD   ${:02x}{:02x}",
                code_buffer[program_counter + 2],
                code_buffer[program_counter + 1]
            );
            op_bytes = 3
        }
        0x23 => print!("INX    H"),
        0x24 => print!("INR    H"),
        0x25 => print!("DCR    H"),
        0x26 => {
            print!("MVI    H,#${:02x}", code_buffer[program_counter + 1]);
            op_bytes = 2
        }
        0x27 => print!("DAA"),
        0x29 => print!("DAD    H"),
        0x2a => {
            print!(
                "LHLD   ${:02x}{:02x}",
                code_buffer[program_counter + 2],
                code_buffer[program_counter + 1]
            );
            op_bytes = 3
        }
        0x2b => print!("DCX    H"),
        0x2c => print!("INR    L"),
        0x2d => print!("DCR    L"),
        0x2e => {
            print!("MVI    L,#${:02x}", code_buffer[program_counter + 1]);
            op_bytes = 2
        }
        0x2f => print!("CMA"),
        0x31 => {
            print!(
                "LXI    SP,#${:02x}{:02x}",
                code_buffer[program_counter + 2],
                code_buffer[program_counter + 1]
            );
            op_bytes = 3
        }
        0x32 => {
            print!(
                "STA    ${:02x}{:02x}",
                code_buffer[program_counter + 2],
                code_buffer[program_counter + 1]
            );
            op_bytes = 3
        }
        0x33 => print!("INX    SP"),
        0x34 => print!("INR    M"),
        0x35 => print!("DCR    M"),
        0x36 => {
            print!("MVI    M,#${:02x}", code_buffer[program_counter + 1]);
            op_bytes = 2
        }
        0x37 => print!("STC"),
        0x39 => print!("DAD    SP"),
        0x3a => {
            print!(
                "LDA    ${:02x}{:02x}",
                code_buffer[program_counter + 2],
                code_buffer[program_counter + 1]
            );
            op_bytes = 3
        }
        0x3b => print!("DCX    SP"),
        0x3c => print!("INR    A"),
        0x3d => print!("DCR    A"),
        0x3e => {
            print!("MVI    A,#${:02x}", code_buffer[program_counter + 1]);
            op_bytes = 2
        }
        0x3f => print!("CMC"),
        0x40 => print!("MOV    B,B"),
        0x41 => print!("MOV    B,C"),
        0x42 => print!("MOV    B,D"),
        0x43 => print!("MOV    B,E"),
        0x44 => print!("MOV    B,H"),
        0x45 => print!("MOV    B,L"),
        0x46 => print!("MOV    B,M"),
        0x47 => print!("MOV    B,A"),
        0x48 => print!("MOV    C,B"),
        0x49 => print!("MOV    C,C"),
        0x4a => print!("MOV    C,D"),
        0x4b => print!("MOV    C,E"),
        0x4c => print!("MOV    C,H"),
        0x4d => print!("MOV    C,L"),
        0x4e => print!("MOV    C,M"),
        0x4f => print!("MOV    C,A"),
        0x50 => print!("MOV    D,B"),
        0x51 => print!("MOV    D,C"),
        0x52 => print!("MOV    D,D"),
        0x53 => print!("MOV    D,E"),
        0x54 => print!("MOV    D,H"),
        0x55 => print!("MOV    D,L"),
        0x56 => print!("MOV    D,M"),
        0x57 => print!("MOV    D,A"),
        0x58 => print!("MOV    E,B"),
        0x59 => print!("MOV    E,C"),
        0x5a => print!("MOV    E,D"),
        0x5b => print!("MOV    E,E"),
        0x5c => print!("MOV    E,H"),
        0x5d => print!("MOV    E,L"),
        0x5e => print!("MOV    E,M"),
        0x5f => print!("MOV    E,A"),
        0x60 => print!("MOV    H,B"),
        0x61 => print!("MOV    H,C"),
        0x62 => print!("MOV    H,D"),
        0x63 => print!("MOV    H,E"),
        0x64 => print!("MOV    H,H"),
        0x65 => print!("MOV    H,L"),
        0x66 => print!("MOV    H,M"),
        0x67 => print!("MOV    H,A"),
        0x68 => print!("MOV    L,B"),
        0x69 => print!("MOV    L,C"),
        0x6a => print!("MOV    L,D"),
        0x6b => print!("MOV    L,E"),
        0x6c => print!("MOV    L,H"),
        0x6d => print!("MOV    L,L"),
        0x6e => print!("MOV    L,M"),
        0x6f => print!("MOV    L,A"),
        0x70 => print!("MOV    M,B"),
        0x71 => print!("MOV    M,C"),
        0x72 => print!("MOV    M,D"),
        0x73 => print!("MOV    M,E"),
        0x74 => print!("MOV    M,H"),
        0x75 => print!("MOV    M,L"),
        0x76 => print!("HLT"),
        0x77 => print!("MOV    M,A"),
        0x78 => print!("MOV    A,B"),
        0x79 => print!("MOV    A,C"),
        0x7a => print!("MOV    A,D"),
        0x7b => print!("MOV    A,E"),
        0x7c => print!("MOV    A,H"),
        0x7d => print!("MOV    A,L"),
        0x7e => print!("MOV    A,M"),
        0x7f => print!("MOV    A,A"),
        0x80 => print!("ADD    B"),
        0x81 => print!("ADD    C"),
        0x82 => print!("ADD    D"),
        0x83 => print!("ADD    E"),
        0x84 => print!("ADD    H"),
        0x85 => print!("ADD    L"),
        0x86 => print!("ADD    M"),
        0x87 => print!("ADD    A"),
        0x88 => print!("ADC    B"),
        0x89 => print!("ADC    C"),
        0x8a => print!("ADC    D"),
        0x8b => print!("ADC    E"),
        0x8c => print!("ADC    H"),
        0x8d => print!("ADC    L"),
        0x8e => print!("ADC    M"),
        0x8f => print!("ADC    A"),
        0x90 => print!("SUB    B"),
        0x91 => print!("SUB    C"),
        0x92 => print!("SUB    D"),
        0x93 => print!("SUB    E"),
        0x94 => print!("SUB    H"),
        0x95 => print!("SUB    L"),
        0x96 => print!("SUB    M"),
        0x97 => print!("SUB    A"),
        0x98 => print!("SBB    B"),
        0x99 => print!("SBB    C"),
        0x9a => print!("SBB    D"),
        0x9b => print!("SBB    E"),
        0x9c => print!("SBB    H"),
        0x9d => print!("SBB    L"),
        0x9e => print!("SBB    M"),
        0x9f => print!("SBB    A"),
        0xa0 => print!("ANA    B"),
        0xa1 => print!("ANA    C"),
        0xa2 => print!("ANA    D"),
        0xa3 => print!("ANA    E"),
        0xa4 => print!("ANA    H"),
        0xa5 => print!("ANA    L"),
        0xa6 => print!("ANA    M"),
        0xa7 => print!("ANA    A"),
        0xa8 => print!("XRA    B"),
        0xa9 => print!("XRA    C"),
        0xaa => print!("XRA    D"),
        0xab => print!("XRA    E"),
        0xac => print!("XRA    H"),
        0xad => print!("XRA    L"),
        0xae => print!("XRA    M"),
        0xaf => print!("XRA    A"),
        0xb0 => print!("ORA    B"),
        0xb1 => print!("ORA    C"),
        0xb2 => print!("ORA    D"),
        0xb3 => print!("ORA    E"),
        0xb4 => print!("ORA    H"),
        0xb5 => print!("ORA    L"),
        0xb6 => print!("ORA    M"),
        0xb7 => print!("ORA    A"),
        0xb8 => print!("CMP    B"),
        0xb9 => print!("CMP    C"),
        0xba => print!("CMP    D"),
        0xbb => print!("CMP    E"),
        0xbc => print!("CMP    H"),
        0xbd => print!("CMP    L"),
        0xbe => print!("CMP    M"),
        0xbf => print!("CMP    A"),
        0xc0 => print!("RNZ"),
        0xc1 => print!("POP    B"),
        0xc2 => {
            print!(
                "JNZ    ${:02x}{:02x}",
                code_buffer[program_counter + 2],
                code_buffer[program_counter + 1]
            );
            op_bytes = 3
        }
        0xc3 => {
            print!(
                "JMP    ${:02x}{:02x}",
                code_buffer[program_counter + 2],
                code_buffer[program_counter + 1]
            );
            op_bytes = 3
        }
        0xc4 => {
            print!(
                "CNZ    ${:02x}{:02x}",
                code_buffer[program_counter + 2],
                code_buffer[program_counter + 1]
            );
            op_bytes = 3
        }
        0xc5 => print!("PUSH   B"),
        0xc6 => {
            print!("ADI    #${:02x}", code_buffer[program_counter + 1]);
            op_bytes = 2
        }
        0xc7 => print!("RST    0"),
        0xc8 => print!("RZ"),
        0xc9 => print!("RET"),
        0xca => {
            print!(
                "JZ     ${:02x}{:02x}",
                code_buffer[program_counter + 2],
                code_buffer[program_counter + 1]
            );
            op_bytes = 3
        }
        0xcc => {
            print!(
                "CZ     ${:02x}{:02x}",
                code_buffer[program_counter + 2],
                code_buffer[program_counter + 1]
            );
            op_bytes = 3
        }
        0xcd => {
            print!(
                "CALL   ${:02x}{:02x}",
                code_buffer[program_counter + 2],
                code_buffer[program_counter + 1]
            );
            op_bytes = 3
        }
        0xce => {
            print!("ACI    #${:02x}", code_buffer[program_counter + 1]);
            op_bytes = 2
        }
        0xcf => print!("RST    1"),
        0xd0 => print!("RNC"),
        0xd1 => print!("POP    D"),
        0xd2 => {
            print!(
                "JNC    ${:02x}{:02x}",
                code_buffer[program_counter + 2],
                code_buffer[program_counter + 1]
            );
            op_bytes = 3
        }
        0xd3 => {
            print!("OUT    #${:02x}", code_buffer[program_counter + 1]);
            op_bytes = 2
        }
        0xd4 => {
            print!(
                "CNC    ${:02x}{:02x}",
                code_buffer[program_counter + 2],
                code_buffer[program_counter + 1]
            );
            op_bytes = 3
        }
        0xd5 => print!("PUSH   D"),
        0xd6 => {
            print!("SUI    #${:02x}", code_buffer[program_counter + 1]);
            op_bytes = 2
        }
        0xd7 => print!("RST    2"),
        0xd8 => print!("RC"),
        0xda => {
            print!(
                "JA     ${:02x}{:02x}",
                code_buffer[program_counter + 2],
                code_buffer[program_counter + 1]
            );
            op_bytes = 3
        }
        0xdb => {
            print!("IN     #${:02x}", code_buffer[program_counter + 1]);
            op_bytes = 2
        }
        0xdc => {
            print!(
                "CC     ${:02x}{:02x}",
                code_buffer[program_counter + 2],
                code_buffer[program_counter + 1]
            );
            op_bytes = 3
        }
        0xde => {
            print!("SBI    #${:02x}", code_buffer[program_counter + 1]);
            op_bytes = 2
        }
        0xdf => print!("RST    3"),
        0xe0 => print!("RPO"),
        0xe1 => print!("POP    H"),
        0xe2 => {
            print!(
                "JPO    ${:02x}{:02x}",
                code_buffer[program_counter + 2],
                code_buffer[program_counter + 1]
            );
            op_bytes = 3
        }
        0xe3 => print!("XTHL"),
        0xe4 => {
            print!(
                "CPO    ${:02x}{:02x}",
                code_buffer[program_counter + 2],
                code_buffer[program_counter + 1]
            );
            op_bytes = 3
        }
        0xe5 => print!("PUSH   H"),
        0xe6 => {
            print!("ANI    #${:02x}", code_buffer[program_counter + 1]);
            op_bytes = 2
        }
        0xe7 => print!("RST    4"),
        0xe8 => print!("RPE"),
        0xe9 => print!("PCHL"),
        0xea => {
            print!(
                "JPE    ${:02x}{:02x}",
                code_buffer[program_counter + 2],
                code_buffer[program_counter + 1]
            );
            op_bytes = 3
        }
        0xeb => print!("XCHG"),
        0xec => {
            print!(
                "CPE    ${:02x}{:02x}",
                code_buffer[program_counter + 2],
                code_buffer[program_counter + 1]
            );
            op_bytes = 3
        }
        0xee => {
            print!("XRI    #${:02x}", code_buffer[program_counter + 1]);
            op_bytes = 2
        }
        0xef => print!("RST    5"),
        0xf0 => print!("RP"),
        0xf1 => print!("POP    PSW"),
        0xf2 => {
            print!(
                "JP     ${:02x}{:02x}",
                code_buffer[program_counter + 2],
                code_buffer[program_counter + 1]
            );
            op_bytes = 3
        }
        0xf3 => print!("DI"),
        0xf4 => {
            print!(
                "CP     ${:02x}{:02x}",
                code_buffer[program_counter + 2],
                code_buffer[program_counter + 1]
            );
            op_bytes = 3
        }
        0xf5 => print!("PUSH   PSW"),
        0xf6 => {
            print!("ORI    #${:02x}", code_buffer[program_counter + 1]);
            op_bytes = 2
        }
        0xf7 => print!("RST    6"),
        0xf8 => print!("RM"),
        0xf9 => print!("SPHL"),
        0xfa => {
            print!(
                "JM     ${:02x}{:02x}",
                code_buffer[program_counter + 2],
                code_buffer[program_counter + 1]
            );
            op_bytes = 3
        }
        0xfb => print!("EI"),
        0xfc => {
            print!(
                "CM     ${:02x}{:02x}",
                code_buffer[program_counter + 2],
                code_buffer[program_counter + 1]
            );
            op_bytes = 3
        }
        0xfe => {
            print!("CPI    #${:02x}", code_buffer[program_counter + 1]);
            op_bytes = 2
        }
        0xff => print!("RST    7"),
    }

    println!();

    return op_bytes;
}

fn unimplemented_instruction(state: &mut I8080State) {
    println!("Error: Unimplimented Instruction");
    disassemble8080_op(&state.memory, state.program_counter as usize);
    panic!(
        "SP: {:04x}\nInstruction: {:02x}",
        state.stack_pointer, state.memory[state.program_counter as usize]
    );
}

// LXI  register,d16
// load u16 into pair of u8 registers (stored in little endian style so ffee => B=ee, C=ff)
fn lxi_load_register_pair_immediate(state: &mut I8080State, register: RegisterSymbols) {
    let reg_1 = state.memory[(state.program_counter + 1) as usize];
    let reg_2 = state.memory[(state.program_counter + 2) as usize];
    match register {
        RegisterSymbols::B => {
            state.reg_c = reg_1;
            state.reg_b = reg_2;
        }
        RegisterSymbols::D => {
            state.reg_e = reg_1;
            state.reg_d = reg_2;
        }
        RegisterSymbols::H => {
            state.reg_l = reg_1;
            state.reg_h = reg_2;
        }
        RegisterSymbols::SP => {
            state.stack_pointer = ((reg_2 as u16) << 8) | (reg_1 as u16);
        }
        _ => panic!("Register for LXI given is undefined"),
    }
    state.program_counter += 3;
}

fn flags_to_u8(flags: &I8080Flags) -> u8 {
    let mut result: u8 = 0x02;
    if flags.sign {
        result &= 0x80;
    }
    if flags.zero {
        result &= 0x40;
    }
    if flags.auxiliary_carry {
        result &= 0x10;
    }
    if flags.parity {
        result &= 0x04;
    }
    if flags.carry {
        result &= 0x01;
    }
    return result;
}

fn u8_to_flags(flags: &mut I8080Flags, value: u8) {
    flags.sign = value & 0x80 != 0;
    flags.zero = value & 0x40 != 0;
    flags.auxiliary_carry = value & 0x10 != 0;
    flags.parity = value & 0x04 != 0;
    flags.carry = value & 0x01 != 0;
}

fn check_parity(value: u8) -> bool {
    let mut result = 0;
    for i in 0..8 {
        result += (value >> i) & 0x1;
    }
    return (result % 2) == 0;
}

fn add_register_add(state: &mut I8080State, register: RegisterSymbols) {
    println!("{:04x}: ADD    {:?}", state.program_counter, register);
    let mut answer = state.reg_a as u16;
    match register {
        RegisterSymbols::B => answer += state.reg_b as u16,
        RegisterSymbols::C => answer += state.reg_c as u16,
        RegisterSymbols::D => answer += state.reg_d as u16,
        RegisterSymbols::E => answer += state.reg_e as u16,
        RegisterSymbols::H => answer += state.reg_h as u16,
        RegisterSymbols::L => answer += state.reg_l as u16,
        RegisterSymbols::A => answer += state.reg_a as u16,
        RegisterSymbols::MEMORY => {
            let mem_loc = ((state.reg_l as u16) << 8) | (state.reg_h as u16);
            answer += state.memory[mem_loc as usize] as u16
        }
        _ => panic!("Register for ADD given is undefined"),
    }
    state.flags.zero = (answer & 0xff) == 0;
    state.flags.sign = (answer & 0x80) != 0;
    state.flags.carry = answer > 0xff;
    state.flags.parity = check_parity((answer & 0xff) as u8);
    state.flags.auxiliary_carry = (answer & 0x10) != ((state.reg_a as u16) & 0x10);
    state.reg_a = (answer & 0xff) as u8;

    state.program_counter += 1;
}

fn adi_immediate_add(state: &mut I8080State) {
    println!(
        "{:04x}: ADI    {:02x}",
        state.program_counter,
        state.memory[(state.program_counter + 1) as usize]
    );
    let answer = (state.reg_a as u16) + (state.memory[(state.program_counter + 1) as usize] as u16);
    state.flags.zero = (answer & 0xff) == 0;
    state.flags.sign = (answer & 0x80) != 0;
    state.flags.carry = answer > 0xff;
    state.flags.parity = check_parity((answer & 0xff) as u8);
    state.flags.auxiliary_carry = (answer & 0x10) != ((state.reg_a as u16) & 0x10);
    state.reg_a = (answer & 0xff) as u8;

    println!(
        "ADI    {:02x}",
        state.memory[(state.program_counter + 1) as usize]
    );
    state.program_counter += 2;
}

// DAD register
// Adds register pair to register pair HL, stores in HL
fn dad_double_add(state: &mut I8080State, register: RegisterSymbols) {
    let hl_val = ((state.reg_h as u16) << 8) | (state.reg_l as u16);
    let rp_val = match register {
        RegisterSymbols::B => ((state.reg_b as u16) << 8) | (state.reg_c as u16),
        RegisterSymbols::D => ((state.reg_d as u16) << 8) | (state.reg_e as u16),
        RegisterSymbols::H => hl_val,
        RegisterSymbols::SP => state.stack_pointer,
        _ => panic!("Register for DAD given is undefined"),
    };

    let result = hl_val.wrapping_add(rp_val);
    state.reg_h = ((result >> 8) & 0xff) as u8;
    state.reg_l = (result & 0xff) as u8;

    state.program_counter += 1;
}

fn inr_increment_register(state: &mut I8080State, register: RegisterSymbols) {
    println!("{:04x}: INR    {:?}", state.program_counter, register);
    let result = match register {
        RegisterSymbols::B => {
            state.reg_b = state.reg_b.wrapping_add(1);
            state.reg_b
        }
        RegisterSymbols::C => {
            state.reg_c = state.reg_c.wrapping_add(1);
            state.reg_c
        }
        RegisterSymbols::D => {
            state.reg_d = state.reg_d.wrapping_add(1);
            state.reg_d
        }
        RegisterSymbols::E => {
            state.reg_e = state.reg_d.wrapping_add(1);
            state.reg_e
        }
        RegisterSymbols::H => {
            state.reg_h = state.reg_h.wrapping_add(1);
            state.reg_h
        }
        RegisterSymbols::L => {
            state.reg_l = state.reg_l.wrapping_add(1);
            state.reg_l
        }
        RegisterSymbols::A => {
            state.reg_a = state.reg_a.wrapping_add(1);
            state.reg_a
        }
        RegisterSymbols::MEMORY => {
            let mem_loc = ((state.reg_l as u16) << 8) | (state.reg_h as u16);
            state.memory[mem_loc as usize] = state.memory[mem_loc as usize].wrapping_add(1);
            state.memory[mem_loc as usize]
        }
        _ => panic!("Register for INC given is undefined"),
    };
    state.flags.sign = (result & 0x80) != 0;
    state.flags.zero = result == 0;
    state.flags.parity = check_parity(result & 0xff);
    state.flags.auxiliary_carry = (result & 0x10) != ((result.wrapping_sub(1)) & 0x10);

    state.program_counter += 1;
}

fn inx_increment_register_pair(state: &mut I8080State, register: RegisterSymbols) {
    match register {
        RegisterSymbols::B => {
            let val = (((state.reg_b as u16) << 8) | (state.reg_c as u16)).wrapping_add(1);
            state.reg_b = ((val >> 8) & 0xff) as u8;
            state.reg_c = (val & 0xff) as u8;
        }
        RegisterSymbols::D => {
            let val = (((state.reg_d as u16) << 8) | (state.reg_e as u16)).wrapping_add(1);
            state.reg_d = ((val >> 8) & 0xff) as u8;
            state.reg_e = (val & 0xff) as u8;
        }
        RegisterSymbols::H => {
            let val = (((state.reg_h as u16) << 8) | (state.reg_l as u16)).wrapping_add(1);
            state.reg_h = ((val >> 8) & 0xff) as u8;
            state.reg_l = (val & 0xff) as u8;
        }
        RegisterSymbols::SP => {
            state.stack_pointer = state.stack_pointer.wrapping_add(1);
        }
        _ => panic!("Register for INX given is undefined"),
    }

    state.program_counter += 1;
}

fn dcr_decrement_register(state: &mut I8080State, register: RegisterSymbols) {
    let result = match register {
        RegisterSymbols::B => {
            state.reg_b = state.reg_b.wrapping_sub(1);
            state.reg_b
        }
        RegisterSymbols::C => {
            state.reg_c = state.reg_c.wrapping_sub(1);
            state.reg_c
        }
        RegisterSymbols::D => {
            state.reg_d = state.reg_d.wrapping_sub(1);
            state.reg_d
        }
        RegisterSymbols::E => {
            state.reg_e = state.reg_e.wrapping_sub(1);
            state.reg_e
        }
        RegisterSymbols::H => {
            state.reg_h = state.reg_h.wrapping_sub(1);
            state.reg_h
        }
        RegisterSymbols::L => {
            state.reg_l = state.reg_l.wrapping_sub(1);
            state.reg_l
        }
        RegisterSymbols::A => {
            state.reg_a = state.reg_a.wrapping_sub(1);
            state.reg_a
        }
        RegisterSymbols::MEMORY => {
            let mem_loc = ((state.reg_h as u16) << 8) | (state.reg_l as u16);
            state.memory[mem_loc as usize] = state.memory[mem_loc as usize].wrapping_sub(1);
            state.memory[mem_loc as usize]
        }
        _ => panic!("Register for DCR given is undefined"),
    };
    state.flags.sign = (result & 0x80) != 0;
    state.flags.zero = result == 0;
    state.flags.parity = check_parity(result & 0xff);
    state.flags.auxiliary_carry = (result & 0x10) != ((result.wrapping_add(1)) & 0x10);

    state.program_counter += 1;
}

fn dcx_decrement_register_pair(state: &mut I8080State, register: RegisterSymbols) {
    println!("{:04x}: DCX    {:?}", state.program_counter, register);
    match register {
        RegisterSymbols::B => {
            let val = (((state.reg_c as u16) << 8) | (state.reg_b as u16)).wrapping_sub(1);
            state.reg_c = ((val >> 8) & 0xff) as u8;
            state.reg_b = (val & 0xff) as u8;
        }
        RegisterSymbols::D => {
            let val = (((state.reg_e as u16) << 8) | (state.reg_d as u16)).wrapping_sub(1);
            state.reg_e = ((val >> 8) & 0xff) as u8;
            state.reg_d = (val & 0xff) as u8;
        }
        RegisterSymbols::H => {
            let val = (((state.reg_l as u16) << 8) | (state.reg_h as u16)).wrapping_sub(1);
            state.reg_l = ((val >> 8) & 0xff) as u8;
            state.reg_h = (val & 0xff) as u8;
        }
        RegisterSymbols::SP => {
            state.stack_pointer = state.stack_pointer.wrapping_sub(1);
        }
        _ => panic!("Register for DCX given is undefined"),
    }

    state.program_counter += 1;
}

// JMP  addr
// jumps to point in address
fn jmp_jump(state: &mut I8080State) {
    // intel 8080 is little endian => val2 is large side
    let val1 = state.memory[(state.program_counter + 1) as usize] as u16;
    let val2 = state.memory[(state.program_counter + 2) as usize] as u16;
    state.program_counter = (val2 << 8) | val1;
}

fn jnz_jump_if_not_zero(state: &mut I8080State) {
    let val1 = state.memory[(state.program_counter + 1) as usize] as u16;
    let val2 = state.memory[(state.program_counter + 2) as usize] as u16;
    if !state.flags.zero {
        state.program_counter = (val2 << 8) | val1;
    } else {
        state.program_counter += 3;
    }
}

fn call_function_call(state: &mut I8080State) {
    let ret = state.program_counter + 3;
    state.memory[(state.stack_pointer - 1) as usize] = ((ret >> 8) & 0xff) as u8;
    state.memory[(state.stack_pointer - 2) as usize] = (ret & 0xff) as u8;
    state.stack_pointer -= 2;
    let call_to = (state.memory[(state.program_counter + 2) as usize] as u16) << 8
        | (state.memory[(state.program_counter + 1) as usize] as u16);
    state.program_counter = call_to;
}

fn ret_function_return(state: &mut I8080State) {
    state.program_counter = (state.memory[state.stack_pointer as usize] as u16)
        | ((state.memory[(state.stack_pointer + 1) as usize] as u16) << 8);
    state.stack_pointer += 2;
}

fn mov_register_move(
    state: &mut I8080State,
    register_to: RegisterSymbols,
    register_from: RegisterSymbols,
) {
    let value = match register_from {
        RegisterSymbols::B => state.reg_b,
        RegisterSymbols::C => state.reg_c,
        RegisterSymbols::D => state.reg_d,
        RegisterSymbols::E => state.reg_e,
        RegisterSymbols::H => state.reg_h,
        RegisterSymbols::L => state.reg_l,
        RegisterSymbols::A => state.reg_a,
        RegisterSymbols::MEMORY => {
            let mem_loc = (((state.reg_h as u16) << 8) | (state.reg_l as u16)) as usize;
            state.memory[mem_loc]
        }
        _ => panic!("Register for ADD given is undefined"),
    };

    match register_to {
        RegisterSymbols::B => state.reg_b = value,
        RegisterSymbols::C => state.reg_c = value,
        RegisterSymbols::D => state.reg_d = value,
        RegisterSymbols::E => state.reg_e = value,
        RegisterSymbols::H => state.reg_h = value,
        RegisterSymbols::L => state.reg_l = value,
        RegisterSymbols::A => state.reg_a = value,
        RegisterSymbols::MEMORY => {
            let mem_loc = (((state.reg_h as u16) << 8) | (state.reg_l as u16)) as usize;
            state.memory[mem_loc] = value
        }
        _ => panic!("Register for ADD given is undefined"),
    }

    state.program_counter += 1;
}

fn mvi_immediate_move(state: &mut I8080State, register: RegisterSymbols) {
    let value = state.memory[(state.program_counter + 1) as usize];
    match register {
        RegisterSymbols::B => state.reg_b = value,
        RegisterSymbols::C => state.reg_c = value,
        RegisterSymbols::D => state.reg_d = value,
        RegisterSymbols::E => state.reg_e = value,
        RegisterSymbols::H => state.reg_h = value,
        RegisterSymbols::L => state.reg_l = value,
        RegisterSymbols::A => state.reg_a = value,
        RegisterSymbols::MEMORY => {
            let mem_loc = (((state.reg_h as u16) << 8) | (state.reg_l as u16)) as usize;
            state.memory[mem_loc] = value
        }
        _ => panic!("Register for ADD given is undefined"),
    }

    state.program_counter += 2;
}

// XCHG
// Exchange register pairs H,L and D,E
fn xchg_exchange_registers(state: &mut I8080State) {
    let tmp_d = state.reg_d;
    let tmp_e = state.reg_e;
    state.reg_d = state.reg_h;
    state.reg_e = state.reg_l;
    state.reg_h = tmp_d;
    state.reg_e = tmp_e;

    state.program_counter += 1;
}

// LDAX register
// load the accumulator register (A) with the address in the register pair provided
fn ldax_load_accumulator_indirect(state: &mut I8080State, register: RegisterSymbols) {
    let address = match register {
        RegisterSymbols::B => ((state.reg_b as u16) << 8) | (state.reg_c as u16),
        RegisterSymbols::D => ((state.reg_d as u16) << 8) | (state.reg_e as u16),
        _ => panic!("Register for LDAX given undefined"),
    };
    state.reg_a = state.memory[address as usize];

    state.program_counter += 1;
}

fn sta_store_accumulator(state: &mut I8080State) {
    println!(
        "{:04x}: STA    {:02x}{:02x}",
        state.program_counter,
        state.memory[(state.program_counter + 2) as usize],
        state.memory[(state.program_counter + 1) as usize]
    );
    let add1 = state.memory[(state.program_counter + 1) as usize] as u16;
    let add2 = state.memory[(state.program_counter + 2) as usize] as u16;
    let address = (add2 << 8) | add1;
    state.memory[address as usize] = state.reg_a;
    state.program_counter += 3;
}

fn cpi_compare_immediate_to_accumulator(state: &mut I8080State) {
    let immediate = state.memory[(state.program_counter + 1) as usize];
    let result = state.reg_a.wrapping_sub(immediate);
    state.flags.zero = result == 0;
    state.flags.carry = (result & 0x80) >= (state.reg_a & 0x80);
    state.flags.sign = (result & 0x80) == 0x80;
    state.flags.parity = check_parity(result);
    state.flags.auxiliary_carry = (result & 0x10) >= (state.reg_a & 0x10);

    state.program_counter += 2
}

fn push_add_to_stack(state: &mut I8080State, register: RegisterSymbols) {
    match register {
        RegisterSymbols::B => {
            state.memory[(state.stack_pointer - 1) as usize] = state.reg_c;
            state.memory[(state.stack_pointer - 2) as usize] = state.reg_b;
        }
        RegisterSymbols::D => {
            state.memory[(state.stack_pointer - 1) as usize] = state.reg_e;
            state.memory[(state.stack_pointer - 2) as usize] = state.reg_d;
        }
        RegisterSymbols::H => {
            state.memory[(state.stack_pointer - 1) as usize] = state.reg_l;
            state.memory[(state.stack_pointer - 2) as usize] = state.reg_h;
        }
        RegisterSymbols::PSW => {
            state.memory[(state.stack_pointer - 1) as usize] = flags_to_u8(&state.flags);
            state.memory[(state.stack_pointer - 2) as usize] = state.reg_a;
        }
        _ => panic!("Register for DCX given is undefined"),
    }
    state.stack_pointer -= 2;

    state.program_counter += 1;
}

fn pop_remove_from_stack(state: &mut I8080State, register: RegisterSymbols) {
    match register {
        RegisterSymbols::B => {
            state.reg_c = state.memory[(state.stack_pointer + 1) as usize];
            state.reg_b = state.memory[state.stack_pointer as usize];
        }
        RegisterSymbols::D => {
            state.reg_e = state.memory[(state.stack_pointer + 1) as usize];
            state.reg_d = state.memory[state.stack_pointer as usize];
        }
        RegisterSymbols::H => {
            state.reg_l = state.memory[(state.stack_pointer + 1) as usize];
            state.reg_h = state.memory[state.stack_pointer as usize];
        }
        RegisterSymbols::PSW => {
            u8_to_flags(
                &mut state.flags,
                state.memory[(state.stack_pointer - 1) as usize],
            );
            state.reg_a = state.memory[state.stack_pointer as usize];
        }
        _ => panic!("Register for POP given is undefined"),
    }
}

fn emulate8080_op(state: &mut I8080State) {
    match state.memory[state.program_counter as usize] {
        0x00 | 0x08 | 0x10 | 0x18 | 0x20 | 0x28 | 0x30 | 0x38 | 0xcb | 0xd9 | 0xdd | 0xed
        | 0xfd => state.program_counter += 1,
        0x01 => lxi_load_register_pair_immediate(state, RegisterSymbols::B),
        0x03 => inx_increment_register_pair(state, RegisterSymbols::B),
        0x04 => inr_increment_register(state, RegisterSymbols::B),
        0x05 => dcr_decrement_register(state, RegisterSymbols::B),
        0x06 => mvi_immediate_move(state, RegisterSymbols::B),
        0x09 => dad_double_add(state, RegisterSymbols::B),
        0x0a => ldax_load_accumulator_indirect(state, RegisterSymbols::B),
        0x0b => dcx_decrement_register_pair(state, RegisterSymbols::B),
        0x0c => inr_increment_register(state, RegisterSymbols::C),
        0x0d => dcr_decrement_register(state, RegisterSymbols::C),
        0x0e => mvi_immediate_move(state, RegisterSymbols::C),
        0x11 => lxi_load_register_pair_immediate(state, RegisterSymbols::D),
        0x13 => inx_increment_register_pair(state, RegisterSymbols::D),
        0x14 => inr_increment_register(state, RegisterSymbols::D),
        0x15 => dcr_decrement_register(state, RegisterSymbols::D),
        0x16 => mvi_immediate_move(state, RegisterSymbols::D),
        0x19 => dad_double_add(state, RegisterSymbols::D),
        0x1a => ldax_load_accumulator_indirect(state, RegisterSymbols::D),
        0x1b => dcx_decrement_register_pair(state, RegisterSymbols::D),
        0x1c => inr_increment_register(state, RegisterSymbols::D),
        0x1d => dcr_decrement_register(state, RegisterSymbols::D),
        0x1e => mvi_immediate_move(state, RegisterSymbols::E),
        0x21 => lxi_load_register_pair_immediate(state, RegisterSymbols::H),
        0x23 => inx_increment_register_pair(state, RegisterSymbols::H),
        0x24 => inr_increment_register(state, RegisterSymbols::H),
        0x25 => dcr_decrement_register(state, RegisterSymbols::H),
        0x26 => mvi_immediate_move(state, RegisterSymbols::H),
        0x29 => dad_double_add(state, RegisterSymbols::H),
        0x2b => dcx_decrement_register_pair(state, RegisterSymbols::H),
        0x2c => inr_increment_register(state, RegisterSymbols::L),
        0x2d => dcr_decrement_register(state, RegisterSymbols::L),
        0x2e => mvi_immediate_move(state, RegisterSymbols::L),
        0x31 => lxi_load_register_pair_immediate(state, RegisterSymbols::SP),
        0x32 => sta_store_accumulator(state),
        0x33 => inx_increment_register_pair(state, RegisterSymbols::SP),
        0x34 => inr_increment_register(state, RegisterSymbols::MEMORY),
        0x35 => dcr_decrement_register(state, RegisterSymbols::MEMORY),
        0x36 => mvi_immediate_move(state, RegisterSymbols::MEMORY),
        0x39 => dad_double_add(state, RegisterSymbols::SP),
        0x3b => dcx_decrement_register_pair(state, RegisterSymbols::SP),
        0x3c => inr_increment_register(state, RegisterSymbols::A),
        0x3d => dcr_decrement_register(state, RegisterSymbols::A),
        0x3e => mvi_immediate_move(state, RegisterSymbols::A),
        0x40 => mov_register_move(state, RegisterSymbols::B, RegisterSymbols::B),
        0x41 => mov_register_move(state, RegisterSymbols::B, RegisterSymbols::C),
        0x42 => mov_register_move(state, RegisterSymbols::B, RegisterSymbols::D),
        0x43 => mov_register_move(state, RegisterSymbols::B, RegisterSymbols::E),
        0x44 => mov_register_move(state, RegisterSymbols::B, RegisterSymbols::H),
        0x45 => mov_register_move(state, RegisterSymbols::B, RegisterSymbols::L),
        0x46 => mov_register_move(state, RegisterSymbols::B, RegisterSymbols::MEMORY),
        0x47 => mov_register_move(state, RegisterSymbols::B, RegisterSymbols::A),
        0x48 => mov_register_move(state, RegisterSymbols::C, RegisterSymbols::B),
        0x49 => mov_register_move(state, RegisterSymbols::C, RegisterSymbols::C),
        0x4a => mov_register_move(state, RegisterSymbols::C, RegisterSymbols::D),
        0x4b => mov_register_move(state, RegisterSymbols::C, RegisterSymbols::E),
        0x4c => mov_register_move(state, RegisterSymbols::C, RegisterSymbols::H),
        0x4d => mov_register_move(state, RegisterSymbols::C, RegisterSymbols::L),
        0x4e => mov_register_move(state, RegisterSymbols::C, RegisterSymbols::MEMORY),
        0x4f => mov_register_move(state, RegisterSymbols::C, RegisterSymbols::A),
        0x50 => mov_register_move(state, RegisterSymbols::E, RegisterSymbols::B),
        0x51 => mov_register_move(state, RegisterSymbols::E, RegisterSymbols::C),
        0x52 => mov_register_move(state, RegisterSymbols::E, RegisterSymbols::D),
        0x53 => mov_register_move(state, RegisterSymbols::E, RegisterSymbols::E),
        0x54 => mov_register_move(state, RegisterSymbols::E, RegisterSymbols::H),
        0x55 => mov_register_move(state, RegisterSymbols::E, RegisterSymbols::L),
        0x56 => mov_register_move(state, RegisterSymbols::E, RegisterSymbols::MEMORY),
        0x57 => mov_register_move(state, RegisterSymbols::E, RegisterSymbols::A),
        0x58 => mov_register_move(state, RegisterSymbols::D, RegisterSymbols::B),
        0x59 => mov_register_move(state, RegisterSymbols::D, RegisterSymbols::C),
        0x5a => mov_register_move(state, RegisterSymbols::D, RegisterSymbols::D),
        0x5b => mov_register_move(state, RegisterSymbols::D, RegisterSymbols::E),
        0x5c => mov_register_move(state, RegisterSymbols::D, RegisterSymbols::H),
        0x5d => mov_register_move(state, RegisterSymbols::D, RegisterSymbols::L),
        0x5e => mov_register_move(state, RegisterSymbols::D, RegisterSymbols::MEMORY),
        0x5f => mov_register_move(state, RegisterSymbols::D, RegisterSymbols::A),
        0x60 => mov_register_move(state, RegisterSymbols::H, RegisterSymbols::B),
        0x61 => mov_register_move(state, RegisterSymbols::H, RegisterSymbols::C),
        0x62 => mov_register_move(state, RegisterSymbols::H, RegisterSymbols::D),
        0x63 => mov_register_move(state, RegisterSymbols::H, RegisterSymbols::E),
        0x64 => mov_register_move(state, RegisterSymbols::H, RegisterSymbols::H),
        0x65 => mov_register_move(state, RegisterSymbols::H, RegisterSymbols::L),
        0x66 => mov_register_move(state, RegisterSymbols::H, RegisterSymbols::MEMORY),
        0x67 => mov_register_move(state, RegisterSymbols::H, RegisterSymbols::A),
        0x68 => mov_register_move(state, RegisterSymbols::L, RegisterSymbols::B),
        0x69 => mov_register_move(state, RegisterSymbols::L, RegisterSymbols::C),
        0x6a => mov_register_move(state, RegisterSymbols::L, RegisterSymbols::D),
        0x6b => mov_register_move(state, RegisterSymbols::L, RegisterSymbols::E),
        0x6c => mov_register_move(state, RegisterSymbols::L, RegisterSymbols::H),
        0x6d => mov_register_move(state, RegisterSymbols::L, RegisterSymbols::L),
        0x6e => mov_register_move(state, RegisterSymbols::L, RegisterSymbols::MEMORY),
        0x6f => mov_register_move(state, RegisterSymbols::L, RegisterSymbols::A),
        0x70 => mov_register_move(state, RegisterSymbols::MEMORY, RegisterSymbols::B),
        0x71 => mov_register_move(state, RegisterSymbols::MEMORY, RegisterSymbols::C),
        0x72 => mov_register_move(state, RegisterSymbols::MEMORY, RegisterSymbols::D),
        0x73 => mov_register_move(state, RegisterSymbols::MEMORY, RegisterSymbols::E),
        0x74 => mov_register_move(state, RegisterSymbols::MEMORY, RegisterSymbols::H),
        0x75 => mov_register_move(state, RegisterSymbols::MEMORY, RegisterSymbols::L),

        0x77 => mov_register_move(state, RegisterSymbols::MEMORY, RegisterSymbols::A),
        0x78 => mov_register_move(state, RegisterSymbols::A, RegisterSymbols::B),
        0x79 => mov_register_move(state, RegisterSymbols::A, RegisterSymbols::C),
        0x7a => mov_register_move(state, RegisterSymbols::A, RegisterSymbols::D),
        0x7b => mov_register_move(state, RegisterSymbols::A, RegisterSymbols::E),
        0x7c => mov_register_move(state, RegisterSymbols::A, RegisterSymbols::H),
        0x7d => mov_register_move(state, RegisterSymbols::A, RegisterSymbols::L),
        0x7e => mov_register_move(state, RegisterSymbols::A, RegisterSymbols::MEMORY),
        0x7f => mov_register_move(state, RegisterSymbols::A, RegisterSymbols::A),
        0x80 => add_register_add(state, RegisterSymbols::B),
        0x81 => add_register_add(state, RegisterSymbols::C),
        0x82 => add_register_add(state, RegisterSymbols::D),
        0x83 => add_register_add(state, RegisterSymbols::E),
        0x84 => add_register_add(state, RegisterSymbols::H),
        0x85 => add_register_add(state, RegisterSymbols::L),
        0x86 => add_register_add(state, RegisterSymbols::MEMORY),
        0x87 => add_register_add(state, RegisterSymbols::A),
        0xc1 => pop_remove_from_stack(state, RegisterSymbols::B),
        0xc2 => jnz_jump_if_not_zero(state),
        0xc3 => jmp_jump(state),
        0xc5 => push_add_to_stack(state, RegisterSymbols::B),
        0xc6 => adi_immediate_add(state),
        0xc9 => ret_function_return(state),
        0xcd => call_function_call(state),
        0xd1 => pop_remove_from_stack(state, RegisterSymbols::D),
        0xd5 => push_add_to_stack(state, RegisterSymbols::D),
        0xe1 => pop_remove_from_stack(state, RegisterSymbols::D),
        0xe5 => push_add_to_stack(state, RegisterSymbols::H),
        0xeb => xchg_exchange_registers(state),
        0xf1 => pop_remove_from_stack(state, RegisterSymbols::SP),
        0xf5 => push_add_to_stack(state, RegisterSymbols::PSW),
        0xfe => cpi_compare_immediate_to_accumulator(state),
        _ => unimplemented_instruction(state),
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut do_dissassemble = false;
    let mut filename = String::new();
    let mut arg_iterator = 1;
    while arg_iterator < args.len() {
        match args[arg_iterator].as_str() {
            "-d" => do_dissassemble = true,
            "-f" => {
                arg_iterator += 1;
                filename = args[arg_iterator].clone();
            }
            _ => panic!("Unknown flag given {}", args[arg_iterator]),
        }
        arg_iterator += 1;
    }
    if filename == "" {
        println!("Enter ROM filename:");
        io::stdin()
            .read_line(&mut filename)
            .expect("Filename not entered.");
        filename = filename.trim().to_string();
    }
    let mut buffer: Vec<u8> = match fs::read(filename.clone()) {
        Ok(res) => res,
        Err(why) => panic!("Failed to open file {}: {}", filename, why),
    };

    buffer.resize(16384, 0);

    if do_dissassemble {
        let mut program_counter = 0;
        while program_counter < buffer.len() {
            program_counter += disassemble8080_op(&buffer, program_counter);
        }
        return;
    }

    let mut state = I8080State {
        reg_a: 0,
        reg_b: 0,
        reg_c: 0,
        reg_d: 0,
        reg_e: 0,
        reg_h: 0,
        reg_l: 0,
        stack_pointer: 0,
        program_counter: 0,
        flags: I8080Flags {
            zero: false,
            sign: false,
            parity: false,
            carry: false,
            auxiliary_carry: false,
        },
        memory: buffer,
    };

    loop {
        emulate8080_op(&mut state);
    }
}
