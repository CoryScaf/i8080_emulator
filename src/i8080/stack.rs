use crate::i8080::RegisterSymbols;
use crate::i8080::State;

impl State {
    fn set_psw_pair(&mut self, value: u16) {
        self.u8_to_flags((value & 0xff) as u8);
        self.reg_a = ((value >> 8) & 0xff) as u8;
    }
    // XCHG
    pub fn xchg_exchange_registers(&mut self) -> u32 {
        let tmp_d = self.reg_d;
        let tmp_e = self.reg_e;
        self.reg_d = self.reg_h;
        self.reg_e = self.reg_l;
        self.reg_h = tmp_d;
        self.reg_l = tmp_e;

        self.program_counter += 1;

        return 5;
    }

    // SPHL
    pub fn sphl_load_sp_from_hl(&mut self) -> u32 {
        self.stack_pointer = self.u8_pair_to_u16(self.reg_l, self.reg_h);
        self.program_counter += 1;

        return 5;
    }

    // XTHL
    pub fn xthl_exchange_top_stack_with_hl(&mut self) -> u32 {
        let new_h = self.memory[(self.stack_pointer + 1) as usize];
        let new_l = self.memory[self.stack_pointer as usize];
        self.memory[(self.stack_pointer + 1) as usize] = self.reg_h;
        self.memory[self.stack_pointer as usize] = self.reg_l;
        self.reg_h = new_h;
        self.reg_l = new_l;

        self.program_counter += 1;

        return 18;
    }

    // PCHL
    pub fn pchl_load_pc_from_hl(&mut self) -> u32 {
        self.program_counter = self.u8_pair_to_u16(self.reg_l, self.reg_h);

        return 5;
    }

    // PUSH reg
    pub fn push_add_to_stack(&mut self, register: RegisterSymbols) -> u32 {
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

        return 11;
    }

    // POP reg
    pub fn pop_remove_from_stack(&mut self, register: RegisterSymbols) -> u32 {
        let result = self.pop_stack();
        match register {
            RegisterSymbols::B => self.set_bc_pair(result),
            RegisterSymbols::D => self.set_de_pair(result),
            RegisterSymbols::H => self.set_hl_pair(result),
            RegisterSymbols::PSW => self.set_psw_pair(result),
            _ => panic!("Invalid register given"),
        }

        self.program_counter += 1;

        return 10;
    }
}
