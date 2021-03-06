use gb_emu::cpu::Cpu;
use gb_emu::mmu::Memory;

use super::{InterfaceData, InterfaceState, DataState, NetworkRequest, offsets, text};

pub fn sprite_check(cpu: &mut Cpu, mem: &mut Memory, interface_data: &mut InterfaceData) {
    if cpu.pc == offsets::OVERWORLD_LOOP_START {
        interface_data.sprite_id_state = DataState::Normal;
    }

    if (cpu.pc == offsets::SPRITE_CHECK_EXIT_1 && mem.lb(offsets::NUM_SPRITES) == 0) ||
        cpu.pc == offsets::SPRITE_CHECK_EXIT_2
    {
        let map_id = mem.lb(offsets::MAP_ID);

        // Determine the tile that the player is trying to move into.
        let mut x = mem.lb(offsets::MAP_X);
        let mut y = mem.lb(offsets::MAP_Y);
        match mem.lb(offsets::PLAYER_DIR) {
            0x00 => y += 1, // Down
            0x04 => y -= 1, // Up
            0x0C => x += 1, // Right
            _    => x -= 1, // Left
        }

        // Check if there are any other players that occupy this tile
        for (id, player) in interface_data.players.iter() {
            if player.movement_data.map_id == map_id && player.check_collision(x, y) {
                // If there was a player set a sentinel value so the game thinks that there is
                // something in the way.
                mem.sb(offsets::SPRITE_INDEX, 0xFF);
                interface_data.sprite_id_state = DataState::Hacked;
                interface_data.last_interaction = *id;
                break;
            }
        }
    }
}

pub fn display_text(cpu: &mut Cpu, mem: &mut Memory, interface_data: &mut InterfaceData) {
    if interface_data.sprite_id_state == DataState::Hacked &&
        cpu.pc == offsets::DISPLAY_TEXT_ID_AFTER_INIT
    {
        // Skip unnecessary parts of the DISPLAY_TEXT_ID routine releated to finding the correct
        // message address when we are interacting with a hacked object.
        cpu.jump(offsets::DISPLAY_TEXT_SETUP_DONE);
        // Set the delay time (this is normally set in the middle of the code we just skipped)
        mem.sb(offsets::FRAME_COUNTER, 30);

        interface_data.text_state = DataState::Hacked;
        interface_data.create_message_box("PLAYER has nothing\nto say.");

        interface_data.network_request = NetworkRequest::Battle(interface_data.last_interaction);
        // We probably want to defer this until as late as possible, to avoid latency causing too
        // much of an issue
        interface_data.state = InterfaceState::Waiting;
    }

    // If the text state is hacked when running the text processor, read from our message buffer
    // instead of from the emulator's memory
    if interface_data.text_state == DataState::Hacked &&
        (cpu.pc == offsets::GET_NEXT_CHAR_1 || cpu.pc == offsets::GET_NEXT_CHAR_2)
    {
        cpu.a = interface_data.current_message.pop_front().unwrap_or(text::special::TERMINATOR);
        cpu.pc += 1;
    }

    // Ensure that when we leave the text processor, we reset the text state so that the next call
    // to the text processor will correctly read from the game.
    if cpu.pc == offsets::TEXT_PROCESSOR_END {
        interface_data.text_state = DataState::Normal;
    }
}

pub fn sprite_update_tracker(cpu: &Cpu, mem: &Memory, interface_data: &mut InterfaceData) {
    if cpu.pc == offsets::UPDATE_SPRITES {
        interface_data.sprites_enabled = mem.lb(offsets::SPRITES_ENABLED) == 0x01;
    }
    else if cpu.pc == offsets::CLEAR_SPRITES {
        interface_data.sprites_enabled = false;
    }
}
