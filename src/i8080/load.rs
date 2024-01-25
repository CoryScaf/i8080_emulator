use crate::i8080::RegisterSymbols;
use crate::i8080::State;

impl State {
    fn get_pair_b_d_register(&mut self, register: &RegisterSymbols) -> u16 {
        match register {
            RegisterSymbols::B => self.u8_pair_to_u16(self.reg_c, self.reg_b),
            RegisterSymbols::D => self.u8_pair_to_u16(self.reg_e, self.reg_d),
            _ => panic!("Invalid register given"),
        }
    }

    // MOV reg,reg
    pub fn mov_register_move(
        &mut self,
        register_to: RegisterSymbols,
        register_from: RegisterSymbols,
    ) -> u32 {
        let value = self.get_single_register(&register_from);
        self.set_single_register(&register_to, value);

        self.program_counter += 1;
        let mut cycles = 5;
        if matches!(register_to, RegisterSymbols::MEMORY)
            || matches!(register_from, RegisterSymbols::MEMORY)
        {
            cycles += 2;
        }

        return cycles;
    }

    // MVI reg,d8
    pub fn mvi_immediate_move(&mut self, register: RegisterSymbols) -> u32 {
        let value = self.get_next(1);
        self.set_single_register(&register, value);

        self.program_counter += 2;

        return State::get_cycles(register, 7, 10);
    }

    // LDA adr
    pub fn lda_load_accumulator_direct(&mut self) -> u32 {
        let address = self.get_next_word();
        self.reg_a = self.memory[address as usize];

        self.program_counter += 3;

        return 13;
    }

    // LDAX reg
    pub fn ldax_load_accumulator_indirect(&mut self, register: RegisterSymbols) -> u32 {
        let address = self.get_pair_b_d_register(&register);
        self.reg_a = self.memory[address as usize];

        self.program_counter += 1;

        return 7;
    }

    // LXI  register,d16
    pub fn lxi_load_register_pair_immediate(&mut self, register: RegisterSymbols) -> u32 {
        let value = self.get_next_word();
        self.set_pair_sp_register(register, value);

        self.program_counter += 3;

        return 10;
    }

    // LHLD adr
    pub fn lhld_load_hl_direct(&mut self) -> u32 {
        let address = self.get_next_word();

        self.reg_h = self.memory[(address + 1) as usize];
        self.reg_l = self.memory[address as usize];

        self.program_counter += 3;

        return 16;
    }

    // STA adr
    pub fn sta_store_accumulator(&mut self) -> u32 {
        let address = self.get_next_word();
        self.memory[address as usize] = self.reg_a;

        self.program_counter += 3;

        return 13;
    }

    // STAX reg
    pub fn stax_store_accumulator_indirect(&mut self, register: RegisterSymbols) -> u32 {
        let address = self.get_pair_b_d_register(&register);
        self.memory[address as usize] = self.reg_a;

        self.program_counter += 1;

        return 7;
    }

    // SHLD adr
    pub fn shld_store_hl_direct(&mut self) -> u32 {
        let address = self.get_next_word();

        self.memory[(address + 1) as usize] = self.reg_h;
        self.memory[address as usize] = self.reg_l;

        self.program_counter += 3;

        return 16;
    }
}
