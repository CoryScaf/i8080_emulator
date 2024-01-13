mod disassemble;
mod i8080;

use std::env;
use std::fs;
use std::io;

// call appropriate function for each code
fn emulate8080_op(state: &mut i8080::State) {
    match state.memory[state.program_counter as usize] {
        0x00 | 0x08 | 0x10 | 0x18 | 0x20 | 0x28 | 0x30 | 0x38 | 0xcb | 0xd9 | 0xdd | 0xed
        | 0xfd => state.program_counter += 1,
        0x01 => state.lxi_load_register_pair_immediate(i8080::RegisterSymbols::B),
        0x02 => state.stax_store_accumulator_indirect(i8080::RegisterSymbols::B),
        0x03 => state.inx_increment_register_pair(i8080::RegisterSymbols::B),
        0x04 => state.inr_increment_register(i8080::RegisterSymbols::B),
        0x05 => state.dcr_decrement_register(i8080::RegisterSymbols::B),
        0x06 => state.mvi_immediate_move(i8080::RegisterSymbols::B),
        0x07 => state.rlc_rotate_left(),
        0x09 => state.dad_double_add(i8080::RegisterSymbols::B),
        0x0a => state.ldax_load_accumulator_indirect(i8080::RegisterSymbols::B),
        0x0b => state.dcx_decrement_register_pair(i8080::RegisterSymbols::B),
        0x0c => state.inr_increment_register(i8080::RegisterSymbols::C),
        0x0d => state.dcr_decrement_register(i8080::RegisterSymbols::C),
        0x0e => state.mvi_immediate_move(i8080::RegisterSymbols::C),
        0x0f => state.rrc_rotate_right(),
        0x11 => state.lxi_load_register_pair_immediate(i8080::RegisterSymbols::D),
        0x12 => state.stax_store_accumulator_indirect(i8080::RegisterSymbols::D),
        0x13 => state.inx_increment_register_pair(i8080::RegisterSymbols::D),
        0x14 => state.inr_increment_register(i8080::RegisterSymbols::D),
        0x15 => state.dcr_decrement_register(i8080::RegisterSymbols::D),
        0x16 => state.mvi_immediate_move(i8080::RegisterSymbols::D),
        0x17 => state.ral_rotate_left_though_carry(),
        0x19 => state.dad_double_add(i8080::RegisterSymbols::D),
        0x1a => state.ldax_load_accumulator_indirect(i8080::RegisterSymbols::D),
        0x1b => state.dcx_decrement_register_pair(i8080::RegisterSymbols::D),
        0x1c => state.inr_increment_register(i8080::RegisterSymbols::E),
        0x1d => state.dcr_decrement_register(i8080::RegisterSymbols::E),
        0x1e => state.mvi_immediate_move(i8080::RegisterSymbols::E),
        0x1f => state.rar_rotate_right_through_carry(),
        0x21 => state.lxi_load_register_pair_immediate(i8080::RegisterSymbols::H),
        0x22 => state.shld_store_hl_direct(),
        0x23 => state.inx_increment_register_pair(i8080::RegisterSymbols::H),
        0x24 => state.inr_increment_register(i8080::RegisterSymbols::H),
        0x25 => state.dcr_decrement_register(i8080::RegisterSymbols::H),
        0x26 => state.mvi_immediate_move(i8080::RegisterSymbols::H),
        0x27 => state.daa_decimal_adjust_accumulator(),
        0x29 => state.dad_double_add(i8080::RegisterSymbols::H),
        0x2a => state.lhld_load_hl_direct(),
        0x2b => state.dcx_decrement_register_pair(i8080::RegisterSymbols::H),
        0x2c => state.inr_increment_register(i8080::RegisterSymbols::L),
        0x2d => state.dcr_decrement_register(i8080::RegisterSymbols::L),
        0x2e => state.mvi_immediate_move(i8080::RegisterSymbols::L),
        0x2f => state.cma_compliment_accumulator(),
        0x31 => state.lxi_load_register_pair_immediate(i8080::RegisterSymbols::SP),
        0x32 => state.sta_store_accumulator(),
        0x33 => state.inx_increment_register_pair(i8080::RegisterSymbols::SP),
        0x34 => state.inr_increment_register(i8080::RegisterSymbols::MEMORY),
        0x35 => state.dcr_decrement_register(i8080::RegisterSymbols::MEMORY),
        0x36 => state.mvi_immediate_move(i8080::RegisterSymbols::MEMORY),
        0x37 => state.stc_set_carry_flag(),
        0x39 => state.dad_double_add(i8080::RegisterSymbols::SP),
        0x3a => state.lda_load_accumulator_direct(),
        0x3b => state.dcx_decrement_register_pair(i8080::RegisterSymbols::SP),
        0x3c => state.inr_increment_register(i8080::RegisterSymbols::A),
        0x3d => state.dcr_decrement_register(i8080::RegisterSymbols::A),
        0x3e => state.mvi_immediate_move(i8080::RegisterSymbols::A),
        0x3f => state.cmc_compliment_carry(),
        0x40 => state.mov_register_move(i8080::RegisterSymbols::B, i8080::RegisterSymbols::B),
        0x41 => state.mov_register_move(i8080::RegisterSymbols::B, i8080::RegisterSymbols::C),
        0x42 => state.mov_register_move(i8080::RegisterSymbols::B, i8080::RegisterSymbols::D),
        0x43 => state.mov_register_move(i8080::RegisterSymbols::B, i8080::RegisterSymbols::E),
        0x44 => state.mov_register_move(i8080::RegisterSymbols::B, i8080::RegisterSymbols::H),
        0x45 => state.mov_register_move(i8080::RegisterSymbols::B, i8080::RegisterSymbols::L),
        0x46 => state.mov_register_move(i8080::RegisterSymbols::B, i8080::RegisterSymbols::MEMORY),
        0x47 => state.mov_register_move(i8080::RegisterSymbols::B, i8080::RegisterSymbols::A),
        0x48 => state.mov_register_move(i8080::RegisterSymbols::C, i8080::RegisterSymbols::B),
        0x49 => state.mov_register_move(i8080::RegisterSymbols::C, i8080::RegisterSymbols::C),
        0x4a => state.mov_register_move(i8080::RegisterSymbols::C, i8080::RegisterSymbols::D),
        0x4b => state.mov_register_move(i8080::RegisterSymbols::C, i8080::RegisterSymbols::E),
        0x4c => state.mov_register_move(i8080::RegisterSymbols::C, i8080::RegisterSymbols::H),
        0x4d => state.mov_register_move(i8080::RegisterSymbols::C, i8080::RegisterSymbols::L),
        0x4e => state.mov_register_move(i8080::RegisterSymbols::C, i8080::RegisterSymbols::MEMORY),
        0x4f => state.mov_register_move(i8080::RegisterSymbols::C, i8080::RegisterSymbols::A),
        0x50 => state.mov_register_move(i8080::RegisterSymbols::D, i8080::RegisterSymbols::B),
        0x51 => state.mov_register_move(i8080::RegisterSymbols::D, i8080::RegisterSymbols::C),
        0x52 => state.mov_register_move(i8080::RegisterSymbols::D, i8080::RegisterSymbols::D),
        0x53 => state.mov_register_move(i8080::RegisterSymbols::D, i8080::RegisterSymbols::E),
        0x54 => state.mov_register_move(i8080::RegisterSymbols::D, i8080::RegisterSymbols::H),
        0x55 => state.mov_register_move(i8080::RegisterSymbols::D, i8080::RegisterSymbols::L),
        0x56 => state.mov_register_move(i8080::RegisterSymbols::D, i8080::RegisterSymbols::MEMORY),
        0x57 => state.mov_register_move(i8080::RegisterSymbols::D, i8080::RegisterSymbols::A),
        0x58 => state.mov_register_move(i8080::RegisterSymbols::E, i8080::RegisterSymbols::B),
        0x59 => state.mov_register_move(i8080::RegisterSymbols::E, i8080::RegisterSymbols::C),
        0x5a => state.mov_register_move(i8080::RegisterSymbols::E, i8080::RegisterSymbols::D),
        0x5b => state.mov_register_move(i8080::RegisterSymbols::E, i8080::RegisterSymbols::E),
        0x5c => state.mov_register_move(i8080::RegisterSymbols::E, i8080::RegisterSymbols::H),
        0x5d => state.mov_register_move(i8080::RegisterSymbols::E, i8080::RegisterSymbols::L),
        0x5e => state.mov_register_move(i8080::RegisterSymbols::E, i8080::RegisterSymbols::MEMORY),
        0x5f => state.mov_register_move(i8080::RegisterSymbols::E, i8080::RegisterSymbols::A),
        0x60 => state.mov_register_move(i8080::RegisterSymbols::H, i8080::RegisterSymbols::B),
        0x61 => state.mov_register_move(i8080::RegisterSymbols::H, i8080::RegisterSymbols::C),
        0x62 => state.mov_register_move(i8080::RegisterSymbols::H, i8080::RegisterSymbols::D),
        0x63 => state.mov_register_move(i8080::RegisterSymbols::H, i8080::RegisterSymbols::E),
        0x64 => state.mov_register_move(i8080::RegisterSymbols::H, i8080::RegisterSymbols::H),
        0x65 => state.mov_register_move(i8080::RegisterSymbols::H, i8080::RegisterSymbols::L),
        0x66 => state.mov_register_move(i8080::RegisterSymbols::H, i8080::RegisterSymbols::MEMORY),
        0x67 => state.mov_register_move(i8080::RegisterSymbols::H, i8080::RegisterSymbols::A),
        0x68 => state.mov_register_move(i8080::RegisterSymbols::L, i8080::RegisterSymbols::B),
        0x69 => state.mov_register_move(i8080::RegisterSymbols::L, i8080::RegisterSymbols::C),
        0x6a => state.mov_register_move(i8080::RegisterSymbols::L, i8080::RegisterSymbols::D),
        0x6b => state.mov_register_move(i8080::RegisterSymbols::L, i8080::RegisterSymbols::E),
        0x6c => state.mov_register_move(i8080::RegisterSymbols::L, i8080::RegisterSymbols::H),
        0x6d => state.mov_register_move(i8080::RegisterSymbols::L, i8080::RegisterSymbols::L),
        0x6e => state.mov_register_move(i8080::RegisterSymbols::L, i8080::RegisterSymbols::MEMORY),
        0x6f => state.mov_register_move(i8080::RegisterSymbols::L, i8080::RegisterSymbols::A),
        0x70 => state.mov_register_move(i8080::RegisterSymbols::MEMORY, i8080::RegisterSymbols::B),
        0x71 => state.mov_register_move(i8080::RegisterSymbols::MEMORY, i8080::RegisterSymbols::C),
        0x72 => state.mov_register_move(i8080::RegisterSymbols::MEMORY, i8080::RegisterSymbols::D),
        0x73 => state.mov_register_move(i8080::RegisterSymbols::MEMORY, i8080::RegisterSymbols::E),
        0x74 => state.mov_register_move(i8080::RegisterSymbols::MEMORY, i8080::RegisterSymbols::H),
        0x75 => state.mov_register_move(i8080::RegisterSymbols::MEMORY, i8080::RegisterSymbols::L),
        0x76 => state.hlt_halt(),
        0x77 => state.mov_register_move(i8080::RegisterSymbols::MEMORY, i8080::RegisterSymbols::A),
        0x78 => state.mov_register_move(i8080::RegisterSymbols::A, i8080::RegisterSymbols::B),
        0x79 => state.mov_register_move(i8080::RegisterSymbols::A, i8080::RegisterSymbols::C),
        0x7a => state.mov_register_move(i8080::RegisterSymbols::A, i8080::RegisterSymbols::D),
        0x7b => state.mov_register_move(i8080::RegisterSymbols::A, i8080::RegisterSymbols::E),
        0x7c => state.mov_register_move(i8080::RegisterSymbols::A, i8080::RegisterSymbols::H),
        0x7d => state.mov_register_move(i8080::RegisterSymbols::A, i8080::RegisterSymbols::L),
        0x7e => state.mov_register_move(i8080::RegisterSymbols::A, i8080::RegisterSymbols::MEMORY),
        0x7f => state.mov_register_move(i8080::RegisterSymbols::A, i8080::RegisterSymbols::A),
        0x80 => state.add_register_add(i8080::RegisterSymbols::B),
        0x81 => state.add_register_add(i8080::RegisterSymbols::C),
        0x82 => state.add_register_add(i8080::RegisterSymbols::D),
        0x83 => state.add_register_add(i8080::RegisterSymbols::E),
        0x84 => state.add_register_add(i8080::RegisterSymbols::H),
        0x85 => state.add_register_add(i8080::RegisterSymbols::L),
        0x86 => state.add_register_add(i8080::RegisterSymbols::MEMORY),
        0x87 => state.add_register_add(i8080::RegisterSymbols::A),
        0x88 => state.adc_add_with_carry_register(i8080::RegisterSymbols::B),
        0x89 => state.adc_add_with_carry_register(i8080::RegisterSymbols::C),
        0x8a => state.adc_add_with_carry_register(i8080::RegisterSymbols::D),
        0x8b => state.adc_add_with_carry_register(i8080::RegisterSymbols::E),
        0x8c => state.adc_add_with_carry_register(i8080::RegisterSymbols::H),
        0x8d => state.adc_add_with_carry_register(i8080::RegisterSymbols::L),
        0x8e => state.adc_add_with_carry_register(i8080::RegisterSymbols::MEMORY),
        0x8f => state.adc_add_with_carry_register(i8080::RegisterSymbols::A),
        0x90 => state.sub_register_subtract(i8080::RegisterSymbols::B),
        0x91 => state.sub_register_subtract(i8080::RegisterSymbols::C),
        0x92 => state.sub_register_subtract(i8080::RegisterSymbols::D),
        0x93 => state.sub_register_subtract(i8080::RegisterSymbols::E),
        0x94 => state.sub_register_subtract(i8080::RegisterSymbols::H),
        0x95 => state.sub_register_subtract(i8080::RegisterSymbols::L),
        0x96 => state.sub_register_subtract(i8080::RegisterSymbols::MEMORY),
        0x97 => state.sub_register_subtract(i8080::RegisterSymbols::A),
        0x98 => state.sbb_subtract_with_carry_register(i8080::RegisterSymbols::B),
        0x99 => state.sbb_subtract_with_carry_register(i8080::RegisterSymbols::C),
        0x9a => state.sbb_subtract_with_carry_register(i8080::RegisterSymbols::D),
        0x9b => state.sbb_subtract_with_carry_register(i8080::RegisterSymbols::E),
        0x9c => state.sbb_subtract_with_carry_register(i8080::RegisterSymbols::H),
        0x9d => state.sbb_subtract_with_carry_register(i8080::RegisterSymbols::L),
        0x9e => state.sbb_subtract_with_carry_register(i8080::RegisterSymbols::MEMORY),
        0x9f => state.sbb_subtract_with_carry_register(i8080::RegisterSymbols::A),
        0xa0 => state.ana_and_register(i8080::RegisterSymbols::B),
        0xa1 => state.ana_and_register(i8080::RegisterSymbols::C),
        0xa2 => state.ana_and_register(i8080::RegisterSymbols::D),
        0xa3 => state.ana_and_register(i8080::RegisterSymbols::E),
        0xa4 => state.ana_and_register(i8080::RegisterSymbols::H),
        0xa5 => state.ana_and_register(i8080::RegisterSymbols::L),
        0xa6 => state.ana_and_register(i8080::RegisterSymbols::MEMORY),
        0xa7 => state.ana_and_register(i8080::RegisterSymbols::A),
        0xa8 => state.xra_exclusive_or_accumulator(i8080::RegisterSymbols::B),
        0xa9 => state.xra_exclusive_or_accumulator(i8080::RegisterSymbols::C),
        0xaa => state.xra_exclusive_or_accumulator(i8080::RegisterSymbols::D),
        0xab => state.xra_exclusive_or_accumulator(i8080::RegisterSymbols::E),
        0xac => state.xra_exclusive_or_accumulator(i8080::RegisterSymbols::H),
        0xad => state.xra_exclusive_or_accumulator(i8080::RegisterSymbols::L),
        0xae => state.xra_exclusive_or_accumulator(i8080::RegisterSymbols::MEMORY),
        0xaf => state.xra_exclusive_or_accumulator(i8080::RegisterSymbols::A),
        0xb0 => state.ora_inclusive_or_accumulator(i8080::RegisterSymbols::B),
        0xb1 => state.ora_inclusive_or_accumulator(i8080::RegisterSymbols::C),
        0xb2 => state.ora_inclusive_or_accumulator(i8080::RegisterSymbols::D),
        0xb3 => state.ora_inclusive_or_accumulator(i8080::RegisterSymbols::E),
        0xb4 => state.ora_inclusive_or_accumulator(i8080::RegisterSymbols::H),
        0xb5 => state.ora_inclusive_or_accumulator(i8080::RegisterSymbols::L),
        0xb6 => state.ora_inclusive_or_accumulator(i8080::RegisterSymbols::MEMORY),
        0xb7 => state.ora_inclusive_or_accumulator(i8080::RegisterSymbols::A),
        0xb8 => state.cmp_compare_register_to_accumulator(i8080::RegisterSymbols::B),
        0xb9 => state.cmp_compare_register_to_accumulator(i8080::RegisterSymbols::C),
        0xba => state.cmp_compare_register_to_accumulator(i8080::RegisterSymbols::D),
        0xbb => state.cmp_compare_register_to_accumulator(i8080::RegisterSymbols::E),
        0xbc => state.cmp_compare_register_to_accumulator(i8080::RegisterSymbols::H),
        0xbd => state.cmp_compare_register_to_accumulator(i8080::RegisterSymbols::L),
        0xbe => state.cmp_compare_register_to_accumulator(i8080::RegisterSymbols::MEMORY),
        0xbf => state.cmp_compare_register_to_accumulator(i8080::RegisterSymbols::A),
        0xc0 => state.rnz_return_if_not_zero(),
        0xc1 => state.pop_remove_from_stack(i8080::RegisterSymbols::B),
        0xc2 => state.jnz_jump_if_not_zero(),
        0xc3 => state.jmp_jump(),
        0xc4 => state.cnz_call_if_not_zero(),
        0xc5 => state.push_add_to_stack(i8080::RegisterSymbols::B),
        0xc6 => state.adi_immediate_add(),
        0xc8 => state.rz_return_if_zero(),
        0xc9 => state.ret_function_return(),
        0xca => state.jz_jump_if_zero(),
        0xcd => state.call_function_call(),
        0xcc => state.cz_call_if_zero(),
        0xce => state.aci_add_with_carry_immediate(),
        0xd0 => state.rnc_return_if_no_carry(),
        0xd1 => state.pop_remove_from_stack(i8080::RegisterSymbols::D),
        0xd2 => state.jnc_jump_if_no_carry(),
        0xd4 => state.cnc_call_if_no_carry(),
        0xd5 => state.push_add_to_stack(i8080::RegisterSymbols::D),
        0xd6 => state.sui_immediate_subtract(),
        0xd8 => state.rc_return_if_carry(),
        0xda => state.jc_jump_if_carry(),
        0xdc => state.cc_call_if_carry(),
        0xde => state.sbi_subtract_with_carry_immediate(),
        0xe0 => state.rpo_return_if_parity_odd(),
        0xe1 => state.pop_remove_from_stack(i8080::RegisterSymbols::H),
        0xe2 => state.jpo_jump_if_parity_odd(),
        0xe3 => state.xthl_exchange_top_stack_with_hl(),
        0xe4 => state.cpo_call_if_parity_odd(),
        0xe5 => state.push_add_to_stack(i8080::RegisterSymbols::H),
        0xe6 => state.ani_and_immediate(),
        0xe8 => state.rpe_return_if_parity_even(),
        0xe9 => state.pchl_load_pc_from_hl(),
        0xea => state.jpe_jump_if_parity_even(),
        0xeb => state.xchg_exchange_registers(),
        0xec => state.cpe_call_if_parity_even(),
        0xee => state.xri_exclusive_or_immediate(),
        0xf0 => state.rp_return_if_plus(),
        0xf1 => state.pop_remove_from_stack(i8080::RegisterSymbols::PSW),
        0xf2 => state.jp_jump_if_plus(),
        0xf3 => state.di_disable_interrupts(),
        0xf4 => state.cp_call_if_plus(),
        0xf5 => state.push_add_to_stack(i8080::RegisterSymbols::PSW),
        0xf6 => state.ori_inclusive_or_immediate(),
        0xf8 => state.rm_return_if_minus(),
        0xf9 => state.sphl_load_sp_from_hl(),
        0xfa => state.jm_jump_if_minus(),
        0xfb => state.ei_enable_interrupts(),
        0xfc => state.cm_call_if_minus(),
        0xfe => state.cpi_compare_immediate_to_accumulator(),
        _ => state.unimplemented_instruction(),
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut do_dissassemble = false;
    let mut filename = String::new();
    let mut arg_iterator = 1;
    let mut do_test = false;
    while arg_iterator < args.len() {
        match args[arg_iterator].as_str() {
            "-d" => do_dissassemble = true,
            "-f" => {
                arg_iterator += 1;
                filename = args[arg_iterator].clone();
            }
            "-t" => do_test = true,
            _ => panic!("Unknown flag given {}", args[arg_iterator]),
        }
        arg_iterator += 1;
    }
    if filename == "" {
        println!("Enter ROM filename:");
        io::stdin()
            .read_line(&mut filename)
            .expect("Filename not entered.");
        filename = filename.trim().to_string();
    }
    let mut buffer: Vec<u8> = match fs::read(filename.clone()) {
        Ok(res) => res,
        Err(why) => panic!("Failed to open file {}: {}", filename, why),
    };

    buffer.resize(0x10000, 0);

    if do_dissassemble {
        let mut program_counter = 0;
        while program_counter < buffer.len() {
            program_counter += disassemble::disassemble8080_op(&buffer, program_counter);
        }
        return;
    }

    let mut state = i8080::State::new(buffer, do_test);

    if state.testing {
        // provided test needs to start at 0x100
        state.memory.rotate_right(0x100);

        // jump to 0x100
        state.memory[0x00] = 0xc3;
        state.memory[0x01] = 0x00;
        state.memory[0x02] = 0x01;

        // fix stack pointer location since starts at 0x100
        state.memory[0x170] = 0x07;

        // change 0x05 to ret since it is a print call
        state.memory[0x05] = 0xc9; // make sure it returns

        // start testing loop which adds special calls
        while !state.should_exit {
            state.check_and_print_call();
            let prev_pc = state.program_counter;
            emulate8080_op(&mut state);
            if state.program_counter == 0 {
                println!("Exit from {:04x}", prev_pc);
                state.should_exit = true;
            }
        }
    }

    while !state.should_exit {
        emulate8080_op(&mut state);
    }
}
