use super::*;

use super::RATIO_MASS;

pub fn create(game: &mut Game) {
    game.map = Map::new(MapInfo {
        size: Size2D::new(200, 200),
    });
    
    super::helper_base(game);
    super::helper_new_entity_center(game);
    
    game.settings.max_cells_spawn = 100_000;
    game.settings.auto_spawn.amount = 500;
    
    let mut rng = rand::thread_rng();
    use rand::Rng;
    for _ in 0..100 {
        game.new_entity(EntityInfo {
            player: 0,
            position: Point2D::new(
                rng.gen_range(0..(game.map.size.width * game.map.size_field)),
                rng.gen_range(0..(game.map.size.height * game.map.size_field))
            ),
            speed: Vector2D::new(0.0, 0.0),
            mass: RATIO_MASS * 100,
            characteristics: EntityCharacteristics {
                on_death: Some(OnDeathEffect::Split(8)),
                ..Default::default()
            },
            timer: EntityTimer::default(),
            color: crate::game::settings::DEFAULT_COLOR[4],
            texture: 10,
        })
    }

    for _ in 0..25 {
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
            color: crate::game::settings::DEFAULT_COLOR[10],
            /*
            color: EntityColor {
                center: crate::game::settings::DEFAULT_COLOR[10].center,
                edge: [125, 10, 10, 255],
            },
            */
            texture: 1,
        })
    }


}
