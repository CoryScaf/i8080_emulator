use crate::i8080::State;

impl State {
    pub fn out_send_output(&mut self) {
        let port = self.get_next(1);

        match port {
            0x03 => {
                // Sound
                println!("Sound played as {:08b}", self.reg_a);
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

        self.program_counter += 1;
    }
}
