use crate::i8080::RegisterSymbols;
use crate::i8080::State;

impl State {
    fn set_psw_pair(&mut self, value: u16) {
        self.u8_to_flags((value & 0xff) as u8);
        self.reg_a = ((value >> 8) & 0xff) as u8;
    }
    // XCHG
    // Exchange register pairs H,L and D,E
    pub fn xchg_exchange_registers(&mut self) {
        let tmp_d = self.reg_d;
        let tmp_e = self.reg_e;
        self.reg_d = self.reg_h;
        self.reg_e = self.reg_l;
        self.reg_h = tmp_d;
        self.reg_l = tmp_e;

        self.program_counter += 1;
    }

    // SPHL
    // Load the stack pointer with the contents of H and L
    pub fn sphl_load_sp_from_hl(&mut self) {
        self.stack_pointer = self.u8_pair_to_u16(self.reg_l, self.reg_h);
        self.program_counter += 1;
    }

    // XTHL
    // Load the top of the stack with the contents of H and L
    pub fn xthl_exchange_top_stack_with_hl(&mut self) {
        let new_h = self.memory[(self.stack_pointer + 1) as usize];
        let new_l = self.memory[self.stack_pointer as usize];
        self.memory[(self.stack_pointer + 1) as usize] = self.reg_h;
        self.memory[self.stack_pointer as usize] = self.reg_l;
        self.reg_h = new_h;
        self.reg_l = new_l;

        self.program_counter += 1;
    }

    // PCHL
    // Load the program counter with the contents of H and L
    pub fn pchl_load_pc_from_hl(&mut self) {
        self.program_counter = self.u8_pair_to_u16(self.reg_l, self.reg_h);
    }

    // PUSH reg
    // push register pair to next point in stack
    pub fn push_add_to_stack(&mut self, register: RegisterSymbols) {
        let value = match register {
            RegisterSymbols::B => self.u8_pair_to_u16(self.reg_c, self.reg_b),
            RegisterSymbols::D => self.u8_pair_to_u16(self.reg_e, self.reg_d),
            RegisterSymbols::H => self.u8_pair_to_u16(self.reg_l, self.reg_h),
            RegisterSymbols::PSW => {
                let u8flags = self.flags_to_u8();
                self.u8_pair_to_u16(u8flags, self.reg_a)
            }
            _ => panic!("Invalid register given"),
        };
        self.push_stack(value);

        self.program_counter += 1;
    }

    // POP reg
    // pop last value off the stack and put it into register
    pub fn pop_remove_from_stack(&mut self, register: RegisterSymbols) {
        let result = self.pop_stack();
        match register {
            RegisterSymbols::B => self.set_bc_pair(result),
            RegisterSymbols::D => self.set_de_pair(result),
            RegisterSymbols::H => self.set_hl_pair(result),
            RegisterSymbols::PSW => self.set_psw_pair(result),
            _ => panic!("Invalid register given"),
        }

        self.program_counter += 1;
    }
}
