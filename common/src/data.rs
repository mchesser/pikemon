pub mod pokeid {
    // TODO: Add more pokemon
    pub const RYHDON: u8 = 0x01;
    pub const WEEDLE: u8 = 0x70;
}

pub mod status {
    pub const NONE    : u8 = 0;
    pub const POISON  : u8 = 3;
    pub const BURN    : u8 = 4;
    pub const FREEZE  : u8 = 5;
    pub const PARALYZE: u8 = 6;
    pub const SLEEP   : u8 = 7;
}

pub mod types {
    pub const NORMAL  : u8 = 0x00;
    pub const FIGHTING: u8 = 0x01;
    pub const FLYING  : u8 = 0x02;
    pub const POISON  : u8 = 0x03;
    pub const GROUND  : u8 = 0x04;
    pub const ROCK    : u8 = 0x05;
    pub const BUG     : u8 = 0x07;
    pub const GHOST   : u8 = 0x08;
    pub const FIRE    : u8 = 0x14;
    pub const WATER   : u8 = 0x15;
    pub const GRASS   : u8 = 0x16;
    pub const ELECTRIC: u8 = 0x17;
    pub const PSYCHIC : u8 = 0x18;
    pub const ICE     : u8 = 0x19;
    pub const DRAGON  : u8 = 0x1A;
}

pub mod moves {
    // TODO: Add more moves
    pub const NONE        : u8 = 0x00;
    pub const POUND       : u8 = 0x01;
    pub const KARATE_CHOP : u8 = 0x02;
    pub const DOUBLESLAP  : u8 = 0x03;
    pub const COMET_PUNCH : u8 = 0x04;
    pub const MEGA_PUNCH  : u8 = 0x05;
    pub const PAY_DAY     : u8 = 0x06;
    pub const FIRE_PUNCH  : u8 = 0x07;
    pub const ICE_PUNCH   : u8 = 0x08;
    pub const THUNDERPUNCH: u8 = 0x09;
    pub const SCRATCH     : u8 = 0x0a;
    pub const VICEGRIP    : u8 = 0x0b;
    pub const GUILLOTINE  : u8 = 0x0c;
    pub const RAZOR_WIND  : u8 = 0x0d;
    pub const SWORDS_DANCE: u8 = 0x0e;
    pub const CUT         : u8 = 0x0f;
}

#[derive(Clone, RustcEncodable, RustcDecodable)]
pub struct Party {
    pub num_pokemon: u8,
    pub pokemon: (PokemonData, PokemonData, PokemonData, PokemonData, PokemonData, PokemonData),
}

#[derive(Clone, RustcEncodable, RustcDecodable)]
#[allow(missing_copy_implementations)]
pub struct PokemonData {
    pub species: u8,
    pub hp: u16,
    pub unknown: u8, // TODO: Determine what this does
    pub status: u8,
    pub type1: u8,
    pub type2: u8,
    pub catch_rate: u8,
    pub moves: (u8, u8, u8, u8),
    pub ot_id: u16,

    pub exp: (u8, u8, u8),
    pub hp_ev: u16,
    pub attack_ev: u16,
    pub defense_ev: u16,
    pub speed_ev: u16,
    pub special_ev: u16,
    pub individual_values: (u8, u8),
    pub move_pp: (u8, u8, u8, u8),

    pub level: u8,
    pub max_hp: u16,
    pub attack: u16,
    pub defense: u16,
    pub speed: u16,
    pub special: u16,
}

impl PokemonData {
    pub fn test_data() -> PokemonData {
        PokemonData {
            species: pokeid::WEEDLE,
            hp: 10,
            unknown: 0,
            status: status::NONE,
            type1: types::BUG,
            type2: types::NORMAL,
            catch_rate: 0,
            moves: (moves::POUND, moves::NONE, moves::NONE, moves::NONE),
            ot_id: 0x1234,

            exp: (0, 0, 0),
            hp_ev: 0,
            attack_ev: 0,
            defense_ev: 0,
            speed_ev: 0,
            special_ev: 0,
            individual_values: (0, 0),
            move_pp: (20, 0, 0, 0),

            level: 10,
            max_hp: 20,
            attack: 10,
            defense: 10,
            speed: 10,
            special: 10,
        }
    }
}
