use crate::i8080::State;

impl State {
    // OUT d8
    pub fn out_send_output(&mut self) -> u32 {
        let port = self.get_next(1);

        match port {
            0x02 => {
                // Shift
                self.shift_amount = self.reg_a & 0b111;
            }
            0x03 => {
                // Sound
                println!("Sound played as {:08b}", self.reg_a);
            }
            0x04 => {
                self.in_ports[0x03] = self.reg_a << self.shift_amount;
            }
            0x05 => {
                // Sound
                println!("Sound played as {:08b}", self.reg_a);
            }
            0x06 => {
                // watchdog
                ()
            }
            _ => panic!("OUT on unimplemented port: {:02x}", port),
        }

        self.program_counter += 2;

        return 10;
    }

    // IN d8
    pub fn in_update_input(&mut self) -> u32 {
        let port = self.get_next(1) as usize;

        self.reg_a = self.in_ports[port];
        self.program_counter += 2;

        return 10;
    }

    // RST d8 (0-7)
    pub fn rst_reset(&mut self, code: u8) -> u32 {
        let rst_loc = (code as u16).wrapping_mul(8);

        self.push_stack(self.program_counter + 1);
        self.program_counter = rst_loc;

        return 11;
    }

    pub fn call_interrupt(&mut self, code: u8) {
        if self.flags.interrupts_enabled {
            self.flags.interrupts_enabled = false;
            self.program_counter -= 1;
            self.rst_reset(code);
        }
    }

    pub fn start_debug_stepping(&mut self) {
        println!("Enabling stepping");
        self.enable_stepping = true;
        self.step_count = 1;
    }

    pub fn stop_debug_stepping(&mut self) {
        println!("Disabling stepping");
        self.enable_stepping = false;
    }
}
