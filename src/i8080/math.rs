use crate::i8080::RegisterSymbols;
use crate::i8080::State;

impl State {
    fn get_pair_sp_register(&mut self, register: &RegisterSymbols) -> u16 {
        match register {
            RegisterSymbols::B => self.u8_pair_to_u16(self.reg_c, self.reg_b),
            RegisterSymbols::D => self.u8_pair_to_u16(self.reg_e, self.reg_d),
            RegisterSymbols::H => self.u8_pair_to_u16(self.reg_l, self.reg_h),
            RegisterSymbols::SP => self.stack_pointer,
            _ => panic!("Invalid register given"),
        }
    }

    fn add_and_set_flags(&mut self, val: u16) {
        let answer = (self.reg_a as u16).wrapping_add(val);
        self.check_flags_single(answer);
    }

    fn sub_and_set_flags(&mut self, val: u16) {
        let answer = (self.reg_a as u16).wrapping_sub(val);
        self.check_flags_single(answer);
    }

    // ADD reg
    // Add given register to register A
    pub fn add_register_add(&mut self, register: RegisterSymbols) {
        let answer = self.get_single_register(&register) as u16;
        self.add_and_set_flags(answer);

        self.program_counter += 1;
    }

    // SUB reg
    // Subtract given register from register A
    pub fn sub_register_subtract(&mut self, register: RegisterSymbols) {
        let answer = self.get_single_register(&register) as u16;
        self.sub_and_set_flags(answer);

        self.program_counter += 1;
    }

    // ADI d8
    // Add immediate to register A
    pub fn adi_immediate_add(&mut self) {
        let answer = self.get_next(1) as u16;
        self.add_and_set_flags(answer);

        self.program_counter += 2;
    }

    // SUI d8
    // Subtract immediate to register A
    pub fn sui_immediate_subtract(&mut self) {
        let answer = self.get_next(1) as u16;
        self.sub_and_set_flags(answer);

        self.program_counter += 2;
    }

    // ACI d8
    // Add immediate to register A and 0x1 if carry bit is set
    pub fn aci_add_with_carry_immediate(&mut self) {
        let carry: u16 = match self.flags.carry {
            true => 0x1,
            false => 0x0,
        };
        let answer = (self.get_next(1) as u16).wrapping_add(carry);
        self.add_and_set_flags(answer);

        self.program_counter += 2;
    }

    // ADC reg
    // Add register to register A and 0x1 if carry bit is set
    pub fn adc_add_with_carry_register(&mut self, register: RegisterSymbols) {
        let carry: u16 = match self.flags.carry {
            true => 0x1,
            false => 0x0,
        };
        let answer = (self.get_single_register(&register) as u16).wrapping_add(carry);
        self.add_and_set_flags(answer);

        self.program_counter += 1;
    }

    // SBI d8
    // Subtract immediate to register A and 0x1 if carry bit is set
    pub fn sbi_subtract_with_carry_immediate(&mut self) {
        let carry: u16 = match self.flags.carry {
            true => 0x1,
            false => 0x0,
        };
        let answer = (self.get_next(1) as u16).wrapping_add(carry);
        self.sub_and_set_flags(answer);

        self.program_counter += 2;
    }

    // SBB reg
    // Subtract register to register A and 0x1 if carry bit is set
    pub fn sbb_subtract_with_carry_register(&mut self, register: RegisterSymbols) {
        let carry: u16 = match self.flags.carry {
            true => 0x1,
            false => 0x0,
        };
        let answer = (self.get_single_register(&register) as u16).wrapping_add(carry);
        self.sub_and_set_flags(answer);

        self.program_counter += 1;
    }

    // DAD reg
    // Adds register pair to register pair HL, stores in HL
    pub fn dad_double_add(&mut self, register: RegisterSymbols) {
        let hl_val = self.u8_pair_to_u16(self.reg_l, self.reg_h) as u32;
        let rp_val = self.get_pair_sp_register(&register) as u32;

        let result = hl_val.wrapping_add(rp_val);
        self.set_hl_pair((result & 0xffff) as u16);

        self.flags.carry = result & 0x100 != 0;

        self.program_counter += 1;
    }

    // DAA
    // Decimal Adjust Accumulator prepares accumulator to be represented as two nibbles
    pub fn daa_decimal_adjust_accumulator(&mut self) {
        let mut result = self.reg_a;
        let mut should_carry = false;
        if (result & 0xf) > 9 || self.flags.auxiliary_carry {
            result = result.wrapping_add(6);
            should_carry = true;
        }
        if (result >> 4) & 0xf > 9 || self.flags.carry {
            result = result.wrapping_add(6 << 4);
        }
        self.flags.carry = should_carry;
        self.reg_a = result;

        self.program_counter += 1;
    }

    fn check_increment_carry(&mut self, result: u8) {
        self.flags.sign = (result & 0x80) != 0;
        self.flags.zero = result == 0;
        self.flags.parity = self.check_parity(result & 0xff);
        self.flags.auxiliary_carry = (result & 0x10) != ((result.wrapping_sub(1)) & 0x10);
    }

    // INR reg
    // increment given register by 1
    pub fn inr_increment_register(&mut self, register: RegisterSymbols) {
        let result = self.get_single_register(&register).wrapping_add(1);
        self.check_increment_carry(result);
        self.set_single_register(&register, result);

        self.program_counter += 1;
    }

    // INX reg
    // increment given register pair by 1
    pub fn inx_increment_register_pair(&mut self, register: RegisterSymbols) {
        let value = self.get_pair_sp_register(&register).wrapping_add(1);
        self.set_pair_sp_register(register, value);

        self.program_counter += 1;
    }

    // DCR reg
    // decrement given register by 1
    pub fn dcr_decrement_register(&mut self, register: RegisterSymbols) {
        let result = self.get_single_register(&register).wrapping_sub(1);
        self.check_increment_carry(result);
        self.set_single_register(&register, result);

        self.program_counter += 1;
    }

    // DCX reg
    // decrement given register pair by 1
    pub fn dcx_decrement_register_pair(&mut self, register: RegisterSymbols) {
        let value = self.get_pair_sp_register(&register).wrapping_sub(1);
        self.set_pair_sp_register(register, value);

        self.program_counter += 1;
    }
}
