use crate::i8080::RegisterSymbols;
use crate::i8080::State;

impl State {
    // RLC
    pub fn rlc_rotate_left(&mut self) -> u32 {
        self.flags.carry = self.reg_a & 0x80 != 0;
        let carry: u8 = match self.flags.carry {
            true => 1,
            false => 0,
        };
        self.reg_a = (self.reg_a << 1) | carry;
        self.program_counter += 1;

        return 4;
    }

    // RRC
    pub fn rrc_rotate_right(&mut self) -> u32 {
        self.flags.carry = self.reg_a & 0x1 != 0;
        let carry: u8 = match self.flags.carry {
            true => 0x80,
            false => 0,
        };
        self.reg_a = (self.reg_a >> 1) | carry;
        self.program_counter += 1;

        return 4;
    }

    // RAL
    pub fn ral_rotate_left_though_carry(&mut self) -> u32 {
        let carry: u8 = match self.flags.carry {
            true => 1,
            false => 0,
        };
        self.flags.carry = self.reg_a & 0x80 != 0;
        self.reg_a = (self.reg_a << 1) | carry;
        self.program_counter += 1;

        return 4;
    }

    // RAR
    pub fn rar_rotate_right_through_carry(&mut self) -> u32 {
        let carry: u8 = match self.flags.carry {
            true => 0x80,
            false => 0,
        };
        self.flags.carry = self.reg_a & 0x1 != 0;
        self.reg_a = (self.reg_a >> 1) | carry;
        self.program_counter += 1;

        return 4;
    }

    // CMA
    pub fn cma_compliment_accumulator(&mut self) -> u32 {
        self.reg_a = !self.reg_a;
        self.program_counter += 1;

        return 4;
    }

    // CMC
    pub fn cmc_compliment_carry(&mut self) -> u32 {
        self.flags.carry = !self.flags.carry;
        self.program_counter += 1;

        return 4;
    }

    // STC
    pub fn stc_set_carry_flag(&mut self) -> u32 {
        self.flags.carry = true;
        self.program_counter += 1;

        return 4;
    }

    pub fn flags_on_accumulator(&mut self) {
        self.flags.zero = self.reg_a == 0;
        self.flags.sign = self.reg_a & 0x80 != 0;
        self.flags.parity = self.check_parity(self.reg_a);
        self.flags.carry = false;
        self.flags.auxiliary_carry = false;
    }

    fn flags_for_compare(&mut self, result: u16) {
        self.flags.zero = result == 0;
        self.flags.carry = (result & 0x100) == 0x100;
        self.flags.sign = (result & 0x80) == 0x80;
        self.flags.parity = self.check_parity((result & 0xff) as u8);
        self.flags.auxiliary_carry = ((result & 0x10) as u8) >= (self.reg_a & 0x10);
    }

    // CMP reg
    pub fn cmp_compare_register_to_accumulator(&mut self, register: RegisterSymbols) -> u32 {
        let cmp = self.get_single_register(&register) as u16;
        let result = (self.reg_a as u16).wrapping_sub(cmp);
        self.flags_for_compare(result);

        self.program_counter += 1;

        return State::get_cycles(register, 4, 7);
    }

    // CPI d8
    pub fn cpi_compare_immediate_to_accumulator(&mut self) -> u32 {
        let immediate = self.get_next(1) as u16;
        let result = (self.reg_a as u16).wrapping_sub(immediate);
        self.flags_for_compare(result);

        self.program_counter += 2;

        return 7;
    }

    // ANA reg
    pub fn ana_and_register(&mut self, register: RegisterSymbols) -> u32 {
        let value = self.get_single_register(&register);
        self.reg_a &= value;
        self.flags_on_accumulator();

        self.program_counter += 1;

        return State::get_cycles(register, 4, 7);
    }

    // ANI d8
    pub fn ani_and_immediate(&mut self) -> u32 {
        let immediate = self.get_next(1);
        self.reg_a &= immediate;
        self.flags_on_accumulator();

        self.program_counter += 2;

        return 7;
    }

    // XRA reg
    pub fn xra_exclusive_or_accumulator(&mut self, register: RegisterSymbols) -> u32 {
        let value = self.get_single_register(&register);
        self.reg_a ^= value;
        self.flags_on_accumulator();

        self.program_counter += 1;

        return State::get_cycles(register, 4, 7);
    }

    // XRI d8
    pub fn xri_exclusive_or_immediate(&mut self) -> u32 {
        let immediate = self.get_next(1);
        self.reg_a ^= immediate;
        self.flags_on_accumulator();

        self.program_counter += 2;

        return 7;
    }

    // ORA reg
    pub fn ora_inclusive_or_accumulator(&mut self, register: RegisterSymbols) -> u32 {
        let value = self.get_single_register(&register);
        self.reg_a |= value;
        self.flags_on_accumulator();

        self.program_counter += 1;

        return State::get_cycles(register, 4, 7);
    }

    // ORI d8
    pub fn ori_inclusive_or_immediate(&mut self) -> u32 {
        let immediate = self.get_next(1);
        self.reg_a |= immediate;
        self.flags_on_accumulator();

        self.program_counter += 2;

        return 7;
    }
}
