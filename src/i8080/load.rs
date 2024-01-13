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
    // move contents of second register to first register given
    pub fn mov_register_move(
        &mut self,
        register_to: RegisterSymbols,
        register_from: RegisterSymbols,
    ) {
        let value = self.get_single_register(&register_from);
        self.set_single_register(&register_to, value);

        self.program_counter += 1;
    }

    // MVI reg,d8
    // move immediate value to register given
    pub fn mvi_immediate_move(&mut self, register: RegisterSymbols) {
        let value = self.get_next(1);
        self.set_single_register(&register, value);

        self.program_counter += 2;
    }

    // LDA adr
    // load the accumulator from memory
    pub fn lda_load_accumulator_direct(&mut self) {
        let address = self.get_next_word();
        self.reg_a = self.memory[address as usize];

        self.program_counter += 3;
    }

    // LDAX reg
    // load the accumulator register (A) with the address in the register pair provided
    pub fn ldax_load_accumulator_indirect(&mut self, register: RegisterSymbols) {
        let address = self.get_pair_b_d_register(&register);
        self.reg_a = self.memory[address as usize];

        self.program_counter += 1;
    }

    // LXI  register,d16
    // load u16 into pair of u8 registers (stored in little endian style so ffee => B=ee, C=ff)
    pub fn lxi_load_register_pair_immediate(&mut self, register: RegisterSymbols) {
        let value = self.get_next_word();
        self.set_pair_sp_register(register, value);

        self.program_counter += 3;
    }

    // LHLD adr
    // load the hl pair with value of memory at the address given
    pub fn lhld_load_hl_direct(&mut self) {
        let address = self.get_next_word();

        self.reg_h = self.memory[(address + 1) as usize];
        self.reg_l = self.memory[address as usize];

        self.program_counter += 3;
    }

    // STA adr
    // store the accumulator into memory
    pub fn sta_store_accumulator(&mut self) {
        let address = self.get_next_word();
        self.memory[address as usize] = self.reg_a;

        self.program_counter += 3;
    }

    // STAX reg
    // store the accumulator into memory
    pub fn stax_store_accumulator_indirect(&mut self, register: RegisterSymbols) {
        let address = self.get_pair_b_d_register(&register);
        self.memory[address as usize] = self.reg_a;

        self.program_counter += 1;
    }

    // SHLD adr
    // store the hl pair in memory at the address given
    pub fn shld_store_hl_direct(&mut self) {
        let address = self.get_next_word();

        self.memory[(address + 1) as usize] = self.reg_h;
        self.memory[address as usize] = self.reg_l;

        self.program_counter += 3;
    }
}
