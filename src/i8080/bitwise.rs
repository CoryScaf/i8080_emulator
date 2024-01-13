use crate::i8080::RegisterSymbols;
use crate::i8080::State;

impl State {
    // RLC
    // rotate bits left
    pub fn rlc_rotate_left(&mut self) {
        self.flags.carry = self.reg_a & 0x80 != 0;
        let carry: u8 = match self.flags.carry {
            true => 1,
            false => 0,
        };
        self.reg_a = (self.reg_a << 1) | carry;
        self.program_counter += 1;
    }

    // RRC
    // rotate bits right
    pub fn rrc_rotate_right(&mut self) {
        self.flags.carry = self.reg_a & 0x1 != 0;
        let carry: u8 = match self.flags.carry {
            true => 0x80,
            false => 0,
        };
        self.reg_a = (self.reg_a >> 1) | carry;
        self.program_counter += 1;
    }

    // RAL
    // rotate bits left through the carry bit
    pub fn ral_rotate_left_though_carry(&mut self) {
        let carry: u8 = match self.flags.carry {
            true => 1,
            false => 0,
        };
        self.flags.carry = self.reg_a & 0x80 != 0;
        self.reg_a = (self.reg_a << 1) | carry;
        self.program_counter += 1;
    }

    // RAR
    // rotate bits right through the carry bit
    pub fn rar_rotate_right_through_carry(&mut self) {
        let carry: u8 = match self.flags.carry {
            true => 0x80,
            false => 0,
        };
        self.flags.carry = self.reg_a & 0x1 != 0;
        self.reg_a = (self.reg_a >> 1) | carry;
        self.program_counter += 1;
    }

    // CMA
    // compliment the accumulator
    pub fn cma_compliment_accumulator(&mut self) {
        self.reg_a = !self.reg_a;
        self.program_counter += 1;
    }

    // CMC
    // compliment the carry flag
    pub fn cmc_compliment_carry(&mut self) {
        self.flags.carry = !self.flags.carry;
        self.program_counter += 1;
    }

    // STC
    // set the carry flag
    pub fn stc_set_carry_flag(&mut self) {
        self.flags.carry = true;
        self.program_counter += 1;
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
    // Compare accumulator with register value by subtracting
    pub fn cmp_compare_register_to_accumulator(&mut self, register: RegisterSymbols) {
        let cmp = self.get_single_register(&register) as u16;
        let result = (self.reg_a as u16).wrapping_sub(cmp);
        self.flags_for_compare(result);

        self.program_counter += 1;
    }

    // CPI d8
    // Compare accumulator with immediate value by subtracting
    pub fn cpi_compare_immediate_to_accumulator(&mut self) {
        let immediate = self.get_next(1) as u16;
        let result = (self.reg_a as u16).wrapping_sub(immediate);
        self.flags_for_compare(result);

        self.program_counter += 2;
    }

    // ANA reg
    // And between accumulator and register
    pub fn ana_and_register(&mut self, register: RegisterSymbols) {
        let value = self.get_single_register(&register);
        self.reg_a &= value;
        self.flags_on_accumulator();

        self.program_counter += 1;
    }

    // ANI d8
    // And between accumulator and immediate value
    pub fn ani_and_immediate(&mut self) {
        let immediate = self.get_next(1);
        self.reg_a &= immediate;
        self.flags_on_accumulator();

        self.program_counter += 2;
    }

    // XRA reg
    // exclusive or (^) with accumulator and given register
    pub fn xra_exclusive_or_accumulator(&mut self, register: RegisterSymbols) {
        let value = self.get_single_register(&register);
        self.reg_a ^= value;
        self.flags_on_accumulator();

        self.program_counter += 1;
    }

    // XRI d8
    // exclusive or between accumulator and immediate value
    pub fn xri_exclusive_or_immediate(&mut self) {
        let immediate = self.get_next(1);
        self.reg_a ^= immediate;
        self.flags_on_accumulator();

        self.program_counter += 2;
    }

    // ORA reg
    // inclusive or (|) with accumulator and give register
    pub fn ora_inclusive_or_accumulator(&mut self, register: RegisterSymbols) {
        let value = self.get_single_register(&register);
        self.reg_a |= value;
        self.flags_on_accumulator();

        self.program_counter += 1;
    }

    // ORI d8
    // Or between accumulator and immediate value
    pub fn ori_inclusive_or_immediate(&mut self) {
        let immediate = self.get_next(1);
        self.reg_a |= immediate;
        self.flags_on_accumulator();

        self.program_counter += 2;
    }
}
