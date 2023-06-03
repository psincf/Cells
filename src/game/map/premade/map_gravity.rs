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

    super::helper_base(game);
    game.settings.local_player_characteristics.gravity = Some(EntityGravityInfo {
        power: 1.0,
        distance_clamp: (RATIO_POSITION as f32 * 2.0)..f32::MAX,
        ..Default::default()
    });
    game.settings.max_split = 4;
    super::helper_new_entity_center(game);

    for _ in 0..10 {
        game.new_entity(EntityInfo {
            player: 0,
            position: Point2D::new(
                rng.gen_range(0..(game.map.size.width * game.map.size_field)),
                rng.gen_range(0..(game.map.size.height * game.map.size_field))
            ),
            speed: Vector2D::new(0.0, 0.0),
            mass: RATIO_MASS * 200,
            characteristics: EntityCharacteristics {
                killer: true,
                on_death: Some(OnDeathEffect::Split(8)),
                mass_max: RATIO_MASS * 100_000,
                throw_entity: Some(ThrowEntityInfo {
                    mass_minimum_to_throw: RATIO_MASS * 200,
                    mass_self_added: -RATIO_MASS / 2,
                    mass_entity_thrown: RATIO_MASS * 1,
                    throw_ratio: 0.2,
                    power: 500..1_000,
                    color: ThrownEntityColor::Random(Vec::from(crate::game::settings::DEFAULT_COLOR_RANDOM)),
                    texture: ThrownEntityTexture::Random(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]),
                    characteristics_entity_thrown: ThrownEntityCharacteristics::Custom(Box::new(EntityCharacteristics {                                
                        killer: false,
                        collide: false,
                        mass_min: RATIO_MASS * 1,
                        mass_max: RATIO_MASS * 1,
                        mass_evolution: None,
                        throw_entity: None,
                        inertia: 10,
                        ..Default::default()
                    })),
                    ..Default::default()
                }),
                ..Default::default()
            },
            timer: EntityTimer::default(),
            color: crate::game::settings::DEFAULT_COLOR[1],
            texture: 1,
        })
    }
}
