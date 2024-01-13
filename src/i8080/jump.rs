use crate::i8080::State;

impl State {
    // JMP  adr
    // jumps to point in address
    pub fn jmp_jump(&mut self) {
        let address = self.get_next_word();
        self.program_counter = address;
    }

    fn jump_if(&mut self, should: bool) {
        if should {
            self.jmp_jump();
        } else {
            self.program_counter += 3;
        }
    }

    // JC adr
    // Jump if carry flag is set
    pub fn jc_jump_if_carry(&mut self) {
        self.jump_if(self.flags.carry);
    }

    // JNC adr
    // jumps to address if carry flag not set
    pub fn jnc_jump_if_no_carry(&mut self) {
        self.jump_if(!self.flags.carry);
    }

    // JP adr
    // Jump to address if sign flag not set
    pub fn jp_jump_if_plus(&mut self) {
        self.jump_if(!self.flags.sign);
    }

    // JM adr
    // Jump to address if sign flag set
    pub fn jm_jump_if_minus(&mut self) {
        self.jump_if(self.flags.sign);
    }

    // JZ adr
    // jumps to address if zero flag set
    pub fn jz_jump_if_zero(&mut self) {
        self.jump_if(self.flags.zero);
    }

    // JNZ adr
    // jumps to address if zero flag not set
    pub fn jnz_jump_if_not_zero(&mut self) {
        self.jump_if(!self.flags.zero);
    }

    // JPE adr
    // jumps to address if parity flag set
    pub fn jpe_jump_if_parity_even(&mut self) {
        self.jump_if(self.flags.parity);
    }

    // JPO adr
    // jumps to address if parity flag not set
    pub fn jpo_jump_if_parity_odd(&mut self) {
        self.jump_if(!self.flags.parity);
    }

    // CALL adr
    // jump to address and set return point at stack pointer
    pub fn call_function_call(&mut self) {
        let call_to = self.get_next_word();
        let ret = self.program_counter + 3;

        self.push_stack(ret);

        self.program_counter = call_to;
    }

    fn call_if(&mut self, should: bool) {
        if should {
            self.call_function_call();
        } else {
            self.program_counter += 3;
        }
    }

    // CC adr
    // call if carry bit is set
    pub fn cc_call_if_carry(&mut self) {
        self.call_if(self.flags.carry);
    }

    // CNC adr
    // call if carry flag not set
    pub fn cnc_call_if_no_carry(&mut self) {
        self.call_if(!self.flags.carry);
    }

    // CP adr
    // call if sign flag not set
    pub fn cp_call_if_plus(&mut self) {
        self.call_if(!self.flags.sign);
    }

    // CM adr
    // call if sign flag set
    pub fn cm_call_if_minus(&mut self) {
        self.call_if(self.flags.sign);
    }

    // CZ adr
    // call if zero flag set
    pub fn cz_call_if_zero(&mut self) {
        self.call_if(self.flags.zero);
    }

    // CNZ adr
    // call if zero flag not set
    pub fn cnz_call_if_not_zero(&mut self) {
        self.call_if(!self.flags.zero);
    }

    // CPE adr
    // call if parity flag set
    pub fn cpe_call_if_parity_even(&mut self) {
        self.call_if(self.flags.parity);
    }

    // CPO adr
    // call if parity flag not set
    pub fn cpo_call_if_parity_odd(&mut self) {
        self.call_if(!self.flags.parity);
    }

    // RET
    // return to last address in the stack
    pub fn ret_function_return(&mut self) {
        self.program_counter = self.pop_stack();
    }

    fn return_if(&mut self, should: bool) {
        if should {
            self.ret_function_return();
        } else {
            self.program_counter += 1;
        }
    }

    // RC
    // return if carry bit is set
    pub fn rc_return_if_carry(&mut self) {
        self.return_if(self.flags.carry);
    }

    // RNC
    // return if carry flag not set
    pub fn rnc_return_if_no_carry(&mut self) {
        self.return_if(!self.flags.carry);
    }

    // RP
    // return if sign flag not set
    pub fn rp_return_if_plus(&mut self) {
        self.return_if(!self.flags.sign);
    }

    // RM
    // return if sign flag set
    pub fn rm_return_if_minus(&mut self) {
        self.return_if(self.flags.sign);
    }

    // RZ
    // return if zero flag set
    pub fn rz_return_if_zero(&mut self) {
        self.return_if(self.flags.zero);
    }

    // RNZ
    // return if zero flag not set
    pub fn rnz_return_if_not_zero(&mut self) {
        self.return_if(!self.flags.zero);
    }

    // RPE
    // return if parity flag set
    pub fn rpe_return_if_parity_even(&mut self) {
        self.return_if(self.flags.parity);
    }

    // RPO
    // return if parity flag not set
    pub fn rpo_return_if_parity_odd(&mut self) {
        self.return_if(!self.flags.parity);
    }
}
