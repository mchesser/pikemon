// Player positional data
pub const MAP_ID: u16 = 0xD35E;
pub const MAP_Y: u16 = 0xD361;
pub const MAP_X: u16 = 0xD362;
pub const PLAYER_DY: u16 = 0xC103;
pub const PLAYER_DX: u16 = 0xC105;

// The direction which the player is facing (0: down, 4: up, 8: left, 12: right)
pub const PLAYER_DIR: u16 = 0xC109;

// When a player moves, this value counts down from 8 to 0
pub const WALK_COUNTER: u16 = 0xCFC5;

// General player data
pub const PLAYER_NAME_START: u16 = 0xD158;

// The address of the player spritesheet encoded as 2bpp in the rom
pub const PLAYER_SPRITE_ADDR: u16 = 0x4180;
pub const PLAYER_SPRITE_BANK: usize = 5;

// The address of the main font encoded as a 1bpp sprite in the rom
pub const FONT_ADDR: u16 = 0x5A80;
pub const FONT_BANK: usize = 4;

// The address of the textbox border encoded as 2bpp sprite in the rom
pub const BORDER_ADDR: u16 = 0x6288 + 2 * 8 * (4 * 6 + 1);
pub const BORDER_BANK: usize = 4;

// The location of the tile map
pub const TILE_MAP: u16 = 0xC3A0;

// Useful addresses for hacks
pub const LOADED_ROM_BANK: u16 = 0xFFB8;
pub const FRAME_COUNTER: u16 = 0xFFD5;
pub const BANK_SWITCH: u16 = 0x35D6;

// Addresses for sprite check hack
pub const NUM_SPRITES: u16 = 0xD4E1;
pub const OVERWORLD_LOOP_START: u16 = 0x03FF;
pub const SPRITE_CHECK_START: u16 = 0x0B23;
pub const SPRITE_CHECK_EXIT_1: u16 = 0x0BA0;
pub const SPRITE_CHECK_EXIT_2: u16 = 0x0BC4;
pub const SPRITE_INDEX: u16 = 0xFF8C;

// Addresses for sprite update hack
pub const CLEAR_SPRITES: u16 = 0x0082;
pub const UPDATE_SPRITES: u16 = 0x2429;
pub const SPRITES_ENABLED: u16 = 0xCFCB;

// Addresses for display text hack
pub const DISPLAY_TEXT_ID: u16 = 0x2920;
pub const DISPLAY_TEXT_ID_AFTER_INIT: u16 = 0x292B;
pub const DISPLAY_TEXT_SETUP_DONE: u16 = 0x29CD;
pub const GET_NEXT_CHAR_1: u16 = 0x1B55;
pub const GET_NEXT_CHAR_2: u16 = 0x1956;
pub const TEXT_PROCESSOR_END: u16 = 0x1B5E;

// Addresses for battle hack
pub const TRAINER_CLASS: u16 = 0xD031;
pub const TRAINER_NAME: u16 = 0xD04A;
pub const TRAINER_NUM: u16 = 0xD05D;
pub const ACTIVE_BATTLE: u16 = 0xD057;
pub const CURRRENT_OPPONENT: u16 = 0xD059;
pub const CURRENT_ENEMY_LEVEL: u16 = 0xD127;
pub const CURRENT_ENEMY_NICK: u16 = 0x0000;
pub const BATTLE_TYPE: u16 = 0xD05A;
pub const IS_LINK_BATTLE: u16 = 0xD12B;

// The Prof. Oak battle is unused by the game, so it is a convenient place to replace with our
// battle data.
pub const PROF_OAK_DATA_ADDR: u16 = 0x621D;
pub const PROF_OAK_DATA_BANK: usize = 0xE;

// Addresses for battle data
pub const PLAYER_BATTLE_DATA_START: u16 = 0xD163;
pub const ENEMY_BATTLE_DATA_START: u16 = 0xD89C;
pub const ENEMY_NAME_START: u16 = 0xD887;

// Addresses for specific party data
pub const PARTY_COUNT: u16 = 0xD163;
pub const PARTY_POKE_1: u16 = 0xD16B;
pub const PARTY_POKE_2: u16 = 0xD197;
pub const PARTY_POKE_3: u16 = 0xD1C3;
pub const PARTY_POKE_4: u16 = 0xD1EF;
pub const PARTY_POKE_5: u16 = 0xD21B;
pub const PARTY_POKE_6: u16 = 0xD247;
