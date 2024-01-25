mod bitwise;
mod io_ops;
mod jump;
mod load;
mod math;
mod stack;

pub enum RegisterSymbols {
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

pub struct Flags {
    zero: bool,
    sign: bool,
    parity: bool,
    carry: bool,
    auxiliary_carry: bool,
    interrupts_enabled: bool,
}

pub struct State {
    pub reg_a: u8,
    pub reg_b: u8,
    pub reg_c: u8,
    pub reg_d: u8,
    pub reg_e: u8,
    pub reg_h: u8,
    pub reg_l: u8,
    pub stack_pointer: u16,
    pub program_counter: u16,
    flags: Flags,
    pub memory: Vec<u8>,
    pub testing: bool,
    pub should_exit: bool,
    pub step_count: u16,
    pub enable_stepping: bool,
    pub in_ports: [u8; 4],
    shift_amount: u8,
}

impl State {
    pub fn new(mem: Vec<u8>, test: bool) -> State {
        State {
            reg_a: 0,
            reg_b: 0,
            reg_c: 0,
            reg_d: 0,
            reg_e: 0,
            reg_h: 0,
            reg_l: 0,
            stack_pointer: 0,
            program_counter: 0,
            flags: Flags {
                zero: false,
                sign: false,
                parity: false,
                carry: false,
                auxiliary_carry: false,
                interrupts_enabled: false,
            },
            memory: mem,
            testing: test,
            should_exit: false,
            step_count: 1,
            enable_stepping: false,
            in_ports: [0b00001110, 0b00001000, 0b00000000, 0],
            shift_amount: 0,
        }
    }

    pub fn check_and_print_call(&mut self) {
        if self.program_counter == 5 {
            if self.reg_c == 9 {
                let address = self.u8_pair_to_u16(self.reg_e, self.reg_d);
                let mut ind: u16 = 0;
                while self.memory[(address + ind) as usize] as char != '$' {
                    print!("{}", self.memory[(address + ind) as usize] as char);
                    ind += 1;
                }
                println!("");
            } else if self.reg_c == 2 {
                println!("{}", self.reg_e as char);
            }
        }
    }

    fn get_next(&mut self, offset: u16) -> u8 {
        self.memory[(self.program_counter + offset) as usize]
    }

    fn get_next_word(&mut self) -> u16 {
        ((self.get_next(2) as u16) << 8) | (self.get_next(1) as u16)
    }

    // flags order in as a d8 is SZ0A0P1C
    pub fn flags_to_u8(&self) -> u8 {
        let mut result: u8 = 0x02;
        if self.flags.sign {
            result |= 0x80;
        }
        if self.flags.zero {
            result |= 0x40;
        }
        if self.flags.auxiliary_carry {
            result |= 0x10;
        }
        if self.flags.parity {
            result |= 0x04;
        }
        if self.flags.carry {
            result |= 0x01;
        }
        return result;
    }

    fn u8_to_flags(&mut self, value: u8) {
        self.flags.sign = value & 0x80 != 0;
        self.flags.zero = value & 0x40 != 0;
        self.flags.auxiliary_carry = value & 0x10 != 0;
        self.flags.parity = value & 0x04 != 0;
        self.flags.carry = value & 0x01 != 0;
    }

    // check if number of set bits are even
    fn check_parity(&mut self, value: u8) -> bool {
        let mut result = 0;
        for i in 0..8 {
            result += (value >> i) & 0x1;
        }
        return (result % 2) == 0;
    }

    fn check_flags_single(&mut self, answer: u16) {
        self.flags.zero = (answer & 0xff) == 0;
        self.flags.sign = (answer & 0x80) != 0;
        self.flags.carry = answer > 0xff;
        self.flags.parity = self.check_parity((answer & 0xff) as u8);
        self.flags.auxiliary_carry = (answer & 0x10) != ((self.reg_a as u16) & 0x10);
        self.reg_a = (answer & 0xff) as u8;
    }

    fn hl_to_address(&self) -> usize {
        ((self.reg_h as usize) << 8) | (self.reg_l as usize)
    }

    fn u8_pair_to_u16(&self, low: u8, high: u8) -> u16 {
        ((high as u16) << 8) | (low as u16)
    }

    fn set_bc_pair(&mut self, value: u16) {
        self.reg_b = ((value >> 8) & 0xff) as u8;
        self.reg_c = (value & 0xff) as u8;
    }

    fn set_de_pair(&mut self, value: u16) {
        self.reg_d = ((value >> 8) & 0xff) as u8;
        self.reg_e = (value & 0xff) as u8;
    }

    fn set_hl_pair(&mut self, value: u16) {
        self.reg_h = ((value >> 8) & 0xff) as u8;
        self.reg_l = (value & 0xff) as u8;
    }

    fn get_single_register(&mut self, register: &RegisterSymbols) -> u8 {
        match register {
            RegisterSymbols::B => self.reg_b,
            RegisterSymbols::C => self.reg_c,
            RegisterSymbols::D => self.reg_d,
            RegisterSymbols::E => self.reg_e,
            RegisterSymbols::H => self.reg_h,
            RegisterSymbols::L => self.reg_l,
            RegisterSymbols::A => self.reg_a,
            RegisterSymbols::MEMORY => self.memory[self.hl_to_address()],
            _ => panic!("Invalid register given"),
        }
    }

    fn set_single_register(&mut self, register: &RegisterSymbols, value: u8) {
        match register {
            RegisterSymbols::B => self.reg_b = value,
            RegisterSymbols::C => self.reg_c = value,
            RegisterSymbols::D => self.reg_d = value,
            RegisterSymbols::E => self.reg_e = value,
            RegisterSymbols::H => self.reg_h = value,
            RegisterSymbols::L => self.reg_l = value,
            RegisterSymbols::A => self.reg_a = value,
            RegisterSymbols::MEMORY => {
                let address = self.hl_to_address();
                self.memory[address] = value
            }
            _ => panic!("Invalid register given"),
        }
    }

    fn pop_stack(&mut self) -> u16 {
        let res = (self.memory[self.stack_pointer as usize] as u16)
            | ((self.memory[(self.stack_pointer + 1) as usize] as u16) << 8);
        self.stack_pointer += 2;
        res
    }

    fn push_stack(&mut self, value: u16) {
        self.memory[(self.stack_pointer - 1) as usize] = ((value >> 8) & 0xff) as u8;
        self.memory[(self.stack_pointer - 2) as usize] = (value & 0xff) as u8;
        self.stack_pointer -= 2;
    }

    fn set_pair_sp_register(&mut self, register: RegisterSymbols, value: u16) {
        match register {
            RegisterSymbols::B => self.set_bc_pair(value),
            RegisterSymbols::D => self.set_de_pair(value),
            RegisterSymbols::H => self.set_hl_pair(value),
            RegisterSymbols::SP => self.stack_pointer = value,
            _ => panic!("Invalid register given"),
        }
    }

    fn get_cycles(register: RegisterSymbols, low: u32, high: u32) -> u32 {
        let mut cycles = low;
        if matches!(register, RegisterSymbols::MEMORY) {
            cycles = high;
        }

        return cycles;
    }

    // HLT
    pub fn hlt_halt(&mut self) -> u32 {
        println!("Halting at PC: {:04x}", self.program_counter);
        self.should_exit = true;

        return 7;
    }

    // DI
    pub fn di_disable_interrupts(&mut self) -> u32 {
        self.flags.interrupts_enabled = false;
        self.program_counter += 1;

        return 4;
    }

    // EI
    pub fn ei_enable_interrupts(&mut self) -> u32 {
        self.flags.interrupts_enabled = true;
        self.program_counter += 1;

        return 4;
    }
}
