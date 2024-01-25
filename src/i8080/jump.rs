use crate::i8080::State;

impl State {
    // JMP  adr
    pub fn jmp_jump(&mut self) -> u32 {
        let address = self.get_next_word();
        self.program_counter = address;

        return 10;
    }

    fn jump_if(&mut self, should: bool) -> u32 {
        if should {
            return self.jmp_jump();
        } else {
            self.program_counter += 3;
            return 10;
        }
    }

    // JC adr
    pub fn jc_jump_if_carry(&mut self) -> u32 {
        self.jump_if(self.flags.carry)
    }

    // JNC adr
    pub fn jnc_jump_if_no_carry(&mut self) -> u32 {
        self.jump_if(!self.flags.carry)
    }

    // JP adr
    pub fn jp_jump_if_plus(&mut self) -> u32 {
        self.jump_if(!self.flags.sign)
    }

    // JM adr
    pub fn jm_jump_if_minus(&mut self) -> u32 {
        self.jump_if(self.flags.sign)
    }

    // JZ adr
    pub fn jz_jump_if_zero(&mut self) -> u32 {
        self.jump_if(self.flags.zero)
    }

    // JNZ adr
    pub fn jnz_jump_if_not_zero(&mut self) -> u32 {
        self.jump_if(!self.flags.zero)
    }

    // JPE adr
    pub fn jpe_jump_if_parity_even(&mut self) -> u32 {
        self.jump_if(self.flags.parity)
    }

    // JPO adr
    pub fn jpo_jump_if_parity_odd(&mut self) -> u32 {
        self.jump_if(!self.flags.parity)
    }

    // CALL adr
    pub fn call_function_call(&mut self) -> u32 {
        let call_to = self.get_next_word();
        let ret = self.program_counter + 3;

        self.push_stack(ret);

        self.program_counter = call_to;

        return 17;
    }

    fn call_if(&mut self, should: bool) -> u32 {
        if should {
            return self.call_function_call();
        } else {
            self.program_counter += 3;
            return 11;
        }
    }

    // CC adr
    pub fn cc_call_if_carry(&mut self) -> u32 {
        self.call_if(self.flags.carry)
    }

    // CNC adr
    pub fn cnc_call_if_no_carry(&mut self) -> u32 {
        self.call_if(!self.flags.carry)
    }

    // CP adr
    pub fn cp_call_if_plus(&mut self) -> u32 {
        self.call_if(!self.flags.sign)
    }

    // CM adr
    pub fn cm_call_if_minus(&mut self) -> u32 {
        self.call_if(self.flags.sign)
    }

    // CZ adr
    pub fn cz_call_if_zero(&mut self) -> u32 {
        self.call_if(self.flags.zero)
    }

    // CNZ adr
    pub fn cnz_call_if_not_zero(&mut self) -> u32 {
        self.call_if(!self.flags.zero)
    }

    // CPE adr
    pub fn cpe_call_if_parity_even(&mut self) -> u32 {
        self.call_if(self.flags.parity)
    }

    // CPO adr
    pub fn cpo_call_if_parity_odd(&mut self) -> u32 {
        self.call_if(!self.flags.parity)
    }

    // RET
    pub fn ret_function_return(&mut self) -> u32 {
        self.program_counter = self.pop_stack();

        return 10;
    }

    fn return_if(&mut self, should: bool) -> u32 {
        if should {
            return self.ret_function_return() + 1;
        } else {
            self.program_counter += 1;
            return 5;
        }
    }

    // RC
    pub fn rc_return_if_carry(&mut self) -> u32 {
        self.return_if(self.flags.carry)
    }

    // RNC
    pub fn rnc_return_if_no_carry(&mut self) -> u32 {
        self.return_if(!self.flags.carry)
    }

    // RP
    pub fn rp_return_if_plus(&mut self) -> u32 {
        self.return_if(!self.flags.sign)
    }

    // RM
    pub fn rm_return_if_minus(&mut self) -> u32 {
        self.return_if(self.flags.sign)
    }

    // RZ
    pub fn rz_return_if_zero(&mut self) -> u32 {
        self.return_if(self.flags.zero)
    }

    // RNZ
    pub fn rnz_return_if_not_zero(&mut self) -> u32 {
        self.return_if(!self.flags.zero)
    }

    // RPE
    pub fn rpe_return_if_parity_even(&mut self) -> u32 {
        self.return_if(self.flags.parity)
    }

    // RPO
    pub fn rpo_return_if_parity_odd(&mut self) -> u32 {
        self.return_if(!self.flags.parity)
    }
}
