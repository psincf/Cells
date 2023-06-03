mod map_example;
mod map_classic;
mod map_1_000_000_cells;
mod map_exponential;
mod map_exponential_extreme;
mod map_trail;
mod map_gravity;
mod map_negative_gravity;
mod map_black_hole;
mod map_collisions;
mod map_autonomous;
mod map_particles;
mod map_paint;
mod map_mona_lisa;
mod map_definitive;


use crate::game::Game;
use crate::game::entity::EntityColor;
use crate::game::entity::EntityInfo;
use crate::game::entity::EntityTimer;
use crate::game::entity::EntityGravityInfo;
use crate::game::entity::EntityCharacteristics;
use crate::game::entity::OnDeathEffect;
use crate::game::entity::RATIO_MASS;
use crate::game::entity::ThrownEntityCharacteristics;
use crate::game::entity::ThrowEntityInfo;
use crate::game::entity::ThrownEntityColor;
use crate::game::entity::ThrownEntityTexture;
use crate::game::Map;
use crate::game::map::RATIO_POSITION;
use crate::game::MapInfo;
use crate::game::player::PlayerInfo;
use crate::game::player::PlayerKind;
use crate::game::settings::AutoSpawnEntityColor;
use euclid::default::{Point2D, Size2D, Vector2D};

mod debug_map_collisions;

pub const PREMADE_MAPS: [&str; 15] = [
    "definitive",
    "example",
    "classic",
    "1_000_000 cells",
    "exponential",
    "exponential extreme",
    "trail",
    "gravity",
    "negative gravity",
    "black hole",
    "collisions",
    "autonomous",
    "particles",
    "paint",
    "mona lisa"
];

pub fn create_premade_map(game: &mut Game, map_choosen: &str) {
    game.clear();
    match map_choosen {
        /*
        "example" => {
            debug_map_collisions::create(game);
        }
        */
        "definitive" => {
            map_definitive::create(game);
        }
        "example" => {
            map_example::create(game);
        }
        "classic" => {
            map_classic::create(game);
        }
        "1_000_000 cells" => {
            map_1_000_000_cells::create(game);
        }
        "exponential" => {
            map_exponential::create(game);
        }
        "exponential extreme" => {
            map_exponential_extreme::create(game);
        }
        "trail" => {
            map_trail::create(game);
        }
        "negative gravity" => {
            map_negative_gravity::create(game);
        }
        "gravity" => {
            map_gravity::create(game);
        }
        "black hole" => {
            map_black_hole::create(game);
        }
        "collisions" => {
            map_collisions::create(game);
        }
        "autonomous" => {
            map_autonomous::create(game);
        }
        "particles" => {
            map_particles::create(game);
        }
        "paint" => {
            map_paint::create(game);
        }
        "mona lisa" => {
            map_mona_lisa::create(game);
        }
        _ => { panic!(); }
    }
    crate::APP.get_mut().renderer.camera_future.size = game.settings.camera_initial;
}

fn helper_base(game: &mut Game) {
    game.new_player(PlayerInfo {
        kind: PlayerKind::Neutral,
        entities: Vec::new(),
        ..Default::default()
    });
    game.new_player(PlayerInfo {
        kind: PlayerKind::Player,
        entities: Vec::new(),
        cell_default_color: crate::game::settings::DEFAULT_COLOR[14],
        cell_default_texture: 14,
    });
    
    game.settings.local_player_characteristics = EntityCharacteristics {
        killer: true,
        collide: true,
        mergeable: true,
        //gravity: Some(-10.0),
        mass_min: RATIO_MASS * 10,
        mass_max: RATIO_MASS * 100_000,
        mass_evolution: Some(0.9998),
        can_split_on_kill: true,
        throw_entity: None,
        inertia: 2,
        ..Default::default()
    };
}

fn helper_new_entity_center(game: &mut Game) {
    helper_new_entity_location(game, Point2D::new(game.map.max().width / 2, game.map.max().height / 2));
}

fn helper_new_entity_up_left(game: &mut Game) {
    helper_new_entity_location(game, Point2D::new(0, 0));
}

fn helper_new_entity_location(game: &mut Game, position: Point2D<i32>) {
    game.new_entity(EntityInfo {
        player: 1,
        position: position,
        speed: Vector2D::new(0.0, 0.0),
        mass: RATIO_MASS * 10,
        characteristics: game.settings.local_player_characteristics.clone(),
        timer: EntityTimer::default(),
        color: game.players[1].cell_default_color,
        texture: game.players[1].cell_default_texture,
    });
}
