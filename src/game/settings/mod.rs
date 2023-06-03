pub mod colors;

use crate::prelude::*;
use crate::game::entity::EntityColor;
use crate::game::entity::EntityTimer;

use crate::game::entity::RATIO_MASS;

use std::ops::Range;
#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};

/*
pub const COLOR_GEN: [[u8;4];12] = {
    const SMALL: u8 = 13;
    const MEDIUM: u8 = 127;
    const BIG: u8 = 242;
    /*
    const fn gen() -> [[u8;4];12] {
        const fn check_state(state: &[u8;3]) -> bool {
            //let count_small = state.iter().filter(|c| **c == SMALL).count();
            //let count_medium = state.iter().filter(|c| **c == MEDIUM).count();
            //let count_big = state.iter().filter(|c| **c == BIG).count();

            let count_small = (state[0] == SMALL) as u8 + (state[1] == SMALL) as u8 + (state[2] == SMALL) as u8;
            let count_medium = (state[0] == MEDIUM) as u8 + (state[1] == MEDIUM) as u8 + (state[2] == MEDIUM) as u8;
            let count_big = (state[0] == BIG) as u8 + (state[1] == BIG) as u8 + (state[2] == BIG) as u8;

            if count_small == 3 || count_medium == 3 || count_big == 3 { return false }
            if count_medium == 2 { return false }
            if (count_medium == 1 && count_small == 2) || (count_medium == 1 && count_big == 2) { return false }
            return true
        }
        let mut result = [[0;4];12];
        let mut state = [SMALL, SMALL, SMALL];
        let mut i = 0;
        loop {
            let mut r = 0;
            let mut g = 0;
            let mut b = 0;

            const fn int_to_color(num: i32) -> u8 {
                match num { 0 => { return SMALL }, 1 => { return MEDIUM }, 2 => { return BIG }, _ => { return SMALL } };
            };

            let r_color = int_to_color(r);
            let g_color = int_to_color(g);
            let b_color = int_to_color(b);

            let color = [r_color, g_color, b_color];

            if check_state(&color) {
                result[i] = [color[0], color[1], color[2], 255];
                i += 1;
            }

            b += 1;
            if b == 2 { b = 0; g += 1; }
            if g == 2 { g = 0; r += 1; }
            if r == 2 { break }
        }
        result
    }
    gen()
    */
    [
        [SMALL, SMALL, BIG, 255],
        [SMALL, MEDIUM, BIG, 255],
        [SMALL, BIG, SMALL, 255],
        [SMALL, BIG, MEDIUM, 255],
        [SMALL, BIG, BIG, 255],
        [MEDIUM, SMALL, BIG, 255],
        [MEDIUM, BIG, SMALL, 255],
        [BIG, SMALL, SMALL, 255],
        [BIG, SMALL, MEDIUM, 255],
        [BIG, SMALL, BIG, 255],
        [BIG, MEDIUM, SMALL, 255],
        [BIG, BIG, SMALL, 255],
    ]
};
*/

pub const DEFAULT_COLOR: [EntityColor; 15] = [
    EntityColor { center: [0, 0, 0, 255], edge: [0, 0, 0, 255] },
    EntityColor { center: [13, 13, 242, 255], edge: [0, 0, 0, 255] },
    EntityColor { center: [13, 242, 242, 255], edge: [0, 0, 0, 255] },
    EntityColor { center: [127, 13, 242, 255], edge: [0, 0, 0, 255] },
    EntityColor { center: [13, 242, 13, 255], edge: [0, 0, 0, 255] },
    EntityColor { center: [13, 242, 242, 255], edge: [0, 0, 0, 255] },
    EntityColor { center: [127, 242, 13, 255], edge: [0, 0, 0, 255] },
    EntityColor { center: [242, 127, 13, 255], edge: [0, 0, 0, 255] },
    EntityColor { center: [242, 13, 242, 255], edge: [0, 0, 0, 255] },
    EntityColor { center: [127, 13, 242, 255], edge: [0, 0, 0, 255] },
    EntityColor { center: [242, 13, 13, 255], edge: [0, 0, 0, 255] },
    EntityColor { center: [13, 242, 127, 255], edge: [0, 0, 0, 255] },
    EntityColor { center: [242, 242, 13, 255], edge: [0, 0, 0, 255] },
    EntityColor { center: [127, 127, 127, 255], edge: [0, 0, 0, 255] },
    EntityColor { center: [255, 255, 255, 255], edge: [0, 0, 0, 255] }
];

pub const DEFAULT_COLOR_UNIFORM: [EntityColor; 15] = [
    EntityColor { center: [0, 0, 0, 255], edge: [0, 0, 0, 255] },
    EntityColor { center: [13, 13, 242, 255], edge: [13, 13, 242, 255] },
    EntityColor { center: [13, 242, 242, 255], edge: [13, 242, 242, 255] },
    EntityColor { center: [127, 13, 242, 255], edge: [127, 13, 242, 255] },
    EntityColor { center: [13, 242, 13, 255], edge: [13, 242, 13, 255] },
    EntityColor { center: [13, 242, 242, 255], edge: [13, 242, 242, 255] },
    EntityColor { center: [127, 242, 13, 255], edge: [127, 242, 13, 255] },
    EntityColor { center: [242, 127, 13, 255], edge: [242, 127, 13, 255] },
    EntityColor { center: [242, 13, 242, 255], edge:  [242, 13, 242, 255] },
    EntityColor { center: [127, 13, 242, 255], edge: [127, 13, 242, 255] },
    EntityColor { center: [242, 13, 13, 255], edge: [242, 13, 13, 255] },
    EntityColor { center: [13, 242, 127, 255], edge: [13, 242, 127, 255] },
    EntityColor { center: [242, 242, 13, 255], edge: [242, 242, 13, 255] },
    EntityColor { center: [127, 127, 127, 255], edge: [125, 125, 125, 255] },
    EntityColor { center: [255, 255, 255, 255], edge: [255, 255, 255, 255] }
];
/*
pub const DEFAULT_COLOR_RANDOM: [EntityColor; 12] = [
    EntityColor { center: [13, 13, 242, 255], edge: [0, 0, 0, 255] },
    EntityColor { center: [13, 242, 242, 255], edge: [0, 0, 0, 255] },
    EntityColor { center: [127, 13, 242, 255], edge: [0, 0, 0, 255] },
    EntityColor { center: [13, 242, 13, 255], edge: [0, 0, 0, 255] },
    EntityColor { center: [13, 242, 242, 255], edge: [0, 0, 0, 255] },
    EntityColor { center: [127, 242, 13, 255], edge: [0, 0, 0, 255] },
    EntityColor { center: [242, 127, 13, 255], edge: [0, 0, 0, 255] },
    EntityColor { center: [242, 13, 242, 255], edge: [0, 0, 0, 255] },
    EntityColor { center: [127, 13, 242, 255], edge: [0, 0, 0, 255] },
    EntityColor { center: [242, 13, 13, 255], edge: [0, 0, 0, 255] },
    EntityColor { center: [13, 242, 127, 255], edge: [0, 0, 0, 255] },
    EntityColor { center: [242, 242, 13, 255], edge: [0, 0, 0, 255] },
];
*/
pub const DEFAULT_COLOR_RANDOM: [EntityColor; 6] = [
    EntityColor { center: [13, 13, 242, 255], edge: [0, 0, 0, 255] },
    EntityColor { center: [13, 242, 13, 255], edge: [0, 0, 0, 255] },
    EntityColor { center: [13, 242, 242, 255], edge: [0, 0, 0, 255] },
    EntityColor { center: [242, 13, 13, 255], edge: [0, 0, 0, 255] },
    EntityColor { center: [242, 13, 242, 255], edge: [0, 0, 0, 255] },
    EntityColor { center: [242, 242, 13, 255], edge: [0, 0, 0, 255] },
];
/*
pub const DEFAULT_COLOR_RANDOM_UNIFORM: [EntityColor; 6] = [
    EntityColor { center: [13, 13, 242, 255], edge: [13, 13, 242, 255] },
    EntityColor { center: [13, 242, 13, 255], edge: [13, 242, 13, 255] },
    EntityColor { center: [13, 242, 242, 255], edge: [13, 242, 242, 255] },
    EntityColor { center: [242, 13, 13, 255], edge: [242, 13, 13, 255] },
    EntityColor { center: [242, 13, 242, 255], edge: [242, 13, 242, 255] },
    EntityColor { center: [242, 242, 13, 255], edge: [242, 242, 13, 255] },
];
*/
pub const DEFAULT_COLOR_RANDOM_UNIFORM: [EntityColor; 6] = [
    EntityColor { center: [242, 13, 13, 255], edge: [242, 13, 13, 255] },
    EntityColor { center: [242, 13, 128, 255], edge: [242, 13, 128, 255] },
    EntityColor { center: [242, 13, 242, 255], edge: [242, 13, 242, 255] },
    EntityColor { center: [242, 128, 13, 255], edge: [242, 128, 13, 255] },
    EntityColor { center: [242, 128, 128, 255], edge: [242, 128, 128, 255] },
    EntityColor { center: [242, 242, 13, 255], edge: [242, 242, 13, 255] },
];
/*
pub const DEFAULT_COLOR_RANDOM_UNIFORM: [EntityColor; 12] = [
    EntityColor { center: [13, 13, 242, 255], edge: [13, 13, 242, 255] },
    EntityColor { center: [13, 242, 242, 255], edge: [13, 242, 242, 255] },
    EntityColor { center: [127, 13, 242, 255], edge: [127, 13, 242, 255] },
    EntityColor { center: [13, 242, 13, 255], edge: [13, 242, 13, 255] },
    EntityColor { center: [13, 242, 242, 255], edge: [13, 242, 242, 255] },
    EntityColor { center: [127, 242, 13, 255], edge: [127, 242, 13, 255] },
    EntityColor { center: [242, 127, 13, 255], edge: [242, 127, 13, 255] },
    EntityColor { center: [242, 13, 242, 255], edge:  [242, 13, 242, 255] },
    EntityColor { center: [127, 13, 242, 255], edge: [127, 13, 242, 255] },
    EntityColor { center: [242, 13, 13, 255], edge: [242, 13, 13, 255] },
    EntityColor { center: [13, 242, 127, 255], edge: [13, 242, 127, 255] },
    EntityColor { center: [242, 242, 13, 255], edge: [242, 242, 13, 255] },
];
*/

#[cfg_attr(feature = "serialize", derive(Deserialize, Serialize, reflect::Reflect))]
#[derive(Clone)]
pub enum SpawnPosition {
    Exact(i32, i32),
    Random,
    RandomRange(Range<i32>, Range<i32>),
}

impl SpawnPosition {
    pub fn validate(&mut self, game: &Game) {
        if let SpawnPosition::Exact(x, y) = self {
            *x = (*x).max(0).min(game.map.max().width - 1);
            *y = (*y).max(0).min(game.map.max().height - 1);
        }
        if let SpawnPosition::RandomRange(range_x, range_y) = self {
            range_x.start = range_x.start.max(0);
            range_y.start = range_y.start.max(0);

            range_x.end = range_x.end.max(range_x.start + 1);
            range_y.end = range_y.end.max(range_y.start + 1);
        }
    }
}

#[cfg_attr(feature = "serialize", derive(Deserialize, Serialize, reflect::Reflect))]
#[derive(Clone)]
pub enum AutoSpawnMass {
    Exact(i64),
    Random(Vec<i64>),
    RandomRange(Range<i64>),
}

impl AutoSpawnMass {
    pub fn validate(&mut self) {
        if let AutoSpawnMass::Exact(mass) = self {
            *mass = (*mass).max(0);
        }
        if let AutoSpawnMass::Random(mass_vec) = self {
            if mass_vec.is_empty() { mass_vec.push(0); }
            for mass in mass_vec.iter_mut() {
                *mass = (*mass).max(0);
            }
        }
        if let AutoSpawnMass::RandomRange(range_mass) = self {
            range_mass.start = range_mass.start.max(0);
            range_mass.end = range_mass.end.max(range_mass.start);
        }
    }
}


#[cfg_attr(feature = "serialize", derive(Deserialize, Serialize, reflect::Reflect))]
#[derive(Clone)]
pub enum AutoSpawnEntityTexture {
    CustomIndex(usize),
    Random(Vec<usize>),
}

#[cfg_attr(feature = "serialize", derive(Deserialize, Serialize, reflect::Reflect))]
#[derive(Clone)]
pub enum AutoSpawnEntityColor {
    Custom(EntityColor),
    Random(Vec<EntityColor>),
}

impl AutoSpawnEntityColor {
    pub fn validate(&mut self) {
        if let AutoSpawnEntityColor::Random(vec) = self {
            if vec.len() == 0 { vec.push(EntityColor::default()); }
        }
    }
}

#[cfg_attr(feature = "serialize", derive(Deserialize, Serialize, reflect::Reflect))]
#[derive(Clone)]
pub struct AutoSpawnSettings {
    pub amount: usize,
    pub position: SpawnPosition,
    pub mass: AutoSpawnMass,
    pub color: AutoSpawnEntityColor,
    pub texture: AutoSpawnEntityTexture,
    pub timer: EntityTimer,
    pub characteristics: EntityCharacteristics,
}

#[cfg_attr(feature = "serialize", derive(Deserialize, Serialize, reflect::Reflect))]
#[derive(Clone)]
pub struct ThrownFoodInfo {
    pub mass_minimum_to_throw: i64,
    pub mass_self_added: i64,
    pub mass_entity_thrown: i64,
    pub throw_ratio: i32,
    pub power: std::ops::RangeInclusive<i32>,
    pub angle: std::ops::RangeInclusive<f32>,
    pub color: crate::game::entity::ThrownEntityColor,
    pub texture: crate::game::entity::ThrownEntityTexture,
    pub timer: EntityTimer,
    pub characteristics_entity_thrown: EntityCharacteristics,
}

#[cfg_attr(feature = "serialize", derive(Deserialize, Serialize, reflect::Reflect))]
#[derive(Clone)]
pub struct Settings {
    pub background_color: [f32;4],
    pub matrix_color: [f32;4],
    pub local_player: usize,
    pub local_player_characteristics: EntityCharacteristics,
    pub local_player_food_settings: ThrownFoodInfo, //TODO: per player instead ? or per entity ?
    pub unit_speed: f32,
    pub unit_speed_split: f32,
    pub collision_speed: f32,
    pub max_split: usize, //TODO: change. per player instead ?
    pub max_cells_spawn: usize,
    pub auto_spawn: AutoSpawnSettings, // TODO: make it in an array for multiple possibility
    pub camera_initial: f32,
    pub max_camera: f32,
    pub special: bool, //TODO: delete
}

impl Settings {
    pub fn validate(&mut self, game: &Game) {
        if self.local_player > game.players.len() { self.local_player = game.players.len() - 1 }
        
        self.local_player_characteristics.validate();

        self.local_player_food_settings.power = (*self.local_player_food_settings.power.start()).max(0)..=(*self.local_player_food_settings.power.end()).max((*self.local_player_food_settings.power.start()).max(0));
        self.local_player_food_settings.angle = (*self.local_player_food_settings.angle.start())..=(*self.local_player_food_settings.angle.end()).max(*self.local_player_food_settings.angle.start());
        self.local_player_food_settings.color.validate();
        self.local_player_food_settings.characteristics_entity_thrown.validate();

        self.auto_spawn.position.validate(game);
        self.auto_spawn.color.validate();
        self.auto_spawn.characteristics.validate();
        self.auto_spawn.mass.validate();

        self.max_camera = self.max_camera.max(20.0);
    }
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            //background_color: [0.1, 0.1, 0.1, 1.0],
            background_color: [0.0, 0.0, 0.0, 1.0],
            //background_color: [1.0, 1.0, 1.0, 1.0],
            matrix_color: [1.0, 1.0, 1.0, 1.0],
            local_player: 1,
            local_player_characteristics: EntityCharacteristics::default(),
            local_player_food_settings: ThrownFoodInfo {
                mass_minimum_to_throw: RATIO_MASS * 10,
                mass_self_added: -RATIO_MASS * 2,
                mass_entity_thrown: RATIO_MASS * 2,
                throw_ratio: 1,
                power: 500..=500,
                angle: 0.0..=0.0,
                color: crate::game::entity::ThrownEntityColor::Same,
                texture: crate::game::entity::ThrownEntityTexture::Same,
                timer: EntityTimer::default(),
                characteristics_entity_thrown: EntityCharacteristics {
                    killer: false,
                    collide: false,
                    mass_min: RATIO_MASS * 2,
                    mass_max: RATIO_MASS * 200,
                    mass_evolution: None,
                    on_death: None,
                    throw_entity: None,
                    inertia: 20,
                    ..Default::default()
                }
            },
            unit_speed: 5_000.0,
            unit_speed_split: 0.05,
            collision_speed: 1.0,
            max_split: 64,
            max_cells_spawn: 50_000,
            auto_spawn: AutoSpawnSettings {
                amount: 100,
                position: SpawnPosition::Random,
                mass: AutoSpawnMass::Exact(RATIO_MASS * 1),
                color: AutoSpawnEntityColor::Random(Vec::from(DEFAULT_COLOR_RANDOM_UNIFORM)),
                texture: AutoSpawnEntityTexture::Random(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]),
                //texture: AutoSpawnEntityTexture::CustomIndex(13),
                timer: EntityTimer::default(),
                characteristics: EntityCharacteristics {
                        killer: false,
                        collide: false,
                        mass_min: RATIO_MASS * 1,
                        mass_max: RATIO_MASS * 1_000_000,
                        mass_evolution: None,
                        ..Default::default()
                    },
            },
            camera_initial: 50.0,
            max_camera: 5_000.0,
            special: false,
        }
    }
}