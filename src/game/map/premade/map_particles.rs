use super::*;

use super::RATIO_MASS;

use euclid::default::{Point2D, Size2D, Vector2D};

pub fn create(game: &mut Game) {
    let mut rng = rand::thread_rng();
    use rand::Rng;

    game.map = Map::new(MapInfo {
        size: Size2D::new(100, 100),
    });
    game.settings.max_cells_spawn = 20_000;
    game.settings.auto_spawn.characteristics.inertia = 500;
    game.settings.auto_spawn.color = AutoSpawnEntityColor::Custom(crate::game::settings::DEFAULT_COLOR_UNIFORM[14]);

    super::helper_base(game);
    game.settings.local_player_characteristics.mass_min = RATIO_MASS * 1;
    game.settings.local_player_characteristics.mass_max = RATIO_MASS * 1;
    game.settings.local_player_characteristics.inertia = 20;
    game.settings.max_split = 1;
    super::helper_new_entity_up_left(game);
    
    for _ in 0..3 {
        game.new_entity(EntityInfo {
            player: 0,
            position: Point2D::new(
                rng.gen_range(0..(game.map.size.width * game.map.size_field)),
                rng.gen_range(0..(game.map.size.height * game.map.size_field))
            ),
            speed: Vector2D::new(0.0, 0.0),
            mass: RATIO_MASS * 200,
            characteristics: EntityCharacteristics {
                killer: false,
                affected_by_gravity: true,
                invincible: true,
                gravity: Some(EntityGravityInfo {
                    power: 10.0,
                    speed_clamp: 0.0..100.0,
                    ..Default::default()
                }),
                on_death: Some(OnDeathEffect::Split(8)),
                mass_max: RATIO_MASS * 100_000,
                throw_entity: None,
                inertia: 10_000,
                ..Default::default()
            },
            timer: EntityTimer::default(),
            color: crate::game::settings::DEFAULT_COLOR_UNIFORM[0],
            texture: 0,
        });
    }
}
