use super::*;

use super::RATIO_MASS;

use euclid::default::{Point2D, Size2D, Vector2D};

pub fn create(game: &mut Game) {
    game.map = Map::new(MapInfo {
        size: Size2D::new(100, 100),
    });
    game.settings.max_cells_spawn = 10_000;
    
    super::helper_base(game);
    game.settings.local_player_characteristics.mass_max = RATIO_MASS * 50_000;
    game.new_entity(EntityInfo {
        player: 1,
        position: Point2D::new(game.map.max().width / 2 - 100_000, game.map.max().height / 2),
        speed: Vector2D::new(0.0, 0.0),
        mass: RATIO_MASS * 10,
        characteristics: game.settings.local_player_characteristics.clone(),
        timer: EntityTimer::default(),
        color: crate::game::settings::DEFAULT_COLOR[14],
        texture: 14,
    });

    game.new_entity(EntityInfo {
        player: 0,
        position: Point2D::new(
            game.map.size.width * game.map.size_field / 2,
            game.map.size.height * game.map.size_field / 2, 
        ),
        speed: Vector2D::new(0.0, 0.0),
        mass: RATIO_MASS * 500,
        characteristics: EntityCharacteristics {
            mass_min: RATIO_MASS * 100,
            mass_max: RATIO_MASS * 50_000,
            killer: true,
            on_death: Some(OnDeathEffect::Split(8)),
            throw_entity: Some(ThrowEntityInfo {                            
                mass_minimum_to_throw: RATIO_MASS * 200,
                mass_self_added: -RATIO_MASS / 2,
                mass_entity_thrown: RATIO_MASS,
                throw_ratio: 0.5,
                power: 100..1_000,
                color: ThrownEntityColor::Random(Vec::from(crate::game::settings::DEFAULT_COLOR_RANDOM)),
                texture: ThrownEntityTexture::Random(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]),
                characteristics_entity_thrown: ThrownEntityCharacteristics::Custom(Box::new(EntityCharacteristics {                                
                    killer: true,
                    collide: false,
                    mass_min: RATIO_MASS * 1,
                    mass_max: RATIO_MASS * 5_000,
                    mass_evolution: None,
                    throw_entity: Some(ThrowEntityInfo {                            
                        mass_minimum_to_throw: RATIO_MASS * 2,
                        mass_self_added: -RATIO_MASS / 2,
                        mass_entity_thrown: RATIO_MASS * 1,
                        throw_ratio: 0.2,
                        power: 500..5_000,
                        color: ThrownEntityColor::Random(Vec::from(crate::game::settings::DEFAULT_COLOR_RANDOM)),
                        texture: ThrownEntityTexture::Random(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]),
                        characteristics_entity_thrown: ThrownEntityCharacteristics::Same,
                        ..Default::default()
                    }),
                    ..Default::default()
                })),
                ..Default::default()
            }),
            ..Default::default()
        },
        timer: EntityTimer::default(),
        color: crate::game::settings::DEFAULT_COLOR[12],
        texture: 12,
    })
}
