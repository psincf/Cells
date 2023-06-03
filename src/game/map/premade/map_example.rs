use super::*;

use super::RATIO_MASS;

pub fn create(game: &mut Game) {
    game.map = Map::new(MapInfo {
        size: Size2D::new(
            100,
            100,
        ),
    });
    
    super::helper_base(game);
    super::helper_new_entity_up_left(game);
    
    game.settings.max_cells_spawn = 20_000;
    
    for x in 0..3 {
        for y in 0..3 {
            if x == 1 && y == 1 {continue}
            game.new_entity(EntityInfo {
                player: 0,
                position: Point2D::new(
                    x * game.map.size.width * game.map.size_field / 3 + game.map.size.width * game.map.size_field / 6,
                    y * game.map.size.height * game.map.size_field / 3 + game.map.size.height * game.map.size_field / 6
                ),
                speed: Vector2D::new(0.0, 0.0),
                mass: RATIO_MASS * 50,
                characteristics: EntityCharacteristics {
                    killer: true,
                    collide: false,
                    mass_min: RATIO_MASS * 200,
                    mass_max: RATIO_MASS * 50_000,
                    mass_evolution: None,
                    throw_entity: Some(ThrowEntityInfo {                            
                        mass_minimum_to_throw: RATIO_MASS * 200,
                        mass_self_added: -RATIO_MASS / 2,
                        mass_entity_thrown: RATIO_MASS * 1,
                        throw_ratio: 0.2,
                        power: 500..1_000,
                        color: ThrownEntityColor::Random(Vec::from(crate::game::settings::DEFAULT_COLOR_RANDOM)),
                        texture: ThrownEntityTexture::Random(vec![1, 2, 3, 4 , 5 , 6, 7, 8, 9, 10, 11, 12]),
                        characteristics_entity_thrown: ThrownEntityCharacteristics::Custom(Box::new(EntityCharacteristics {                                
                            killer: false,
                            collide: false,
                            mass_min: RATIO_MASS * 1,
                            mass_max: RATIO_MASS * 1,
                            mass_evolution: None,
                            throw_entity: None,
                            ..Default::default()
                        })),
                        ..Default::default()
                    }),
                    on_death: Some(OnDeathEffect::Split(8)),
                    ..Default::default()
                },
                timer: EntityTimer::default(),
                color: crate::game::settings::DEFAULT_COLOR[1],
                texture: 1,
            });
        }
    }
    
    game.new_entity(EntityInfo {
        player: 0,
        position: Point2D::new(
            game.map.size.width * game.map.size_field / 2,
            game.map.size.height * game.map.size_field / 2
        ),
        speed: Vector2D::new(0.0, 0.0),
        mass: RATIO_MASS * 5_000,
        characteristics: EntityCharacteristics {
            killer: true,
            collide: false,
            mass_min: RATIO_MASS * 1_000,
            mass_max: RATIO_MASS * 100_000,
            mass_evolution: None,
            throw_entity: Some(ThrowEntityInfo {                            
                mass_minimum_to_throw: RATIO_MASS * 5_000,
                mass_self_added: -RATIO_MASS / 2,
                mass_entity_thrown: RATIO_MASS * 1,
                throw_ratio: 1.0,
                power: 500..10_000,
                color: ThrownEntityColor::Random(Vec::from(crate::game::settings::DEFAULT_COLOR_RANDOM)),
                texture: ThrownEntityTexture::Random(vec![1, 2, 3, 4 , 5 , 6, 7, 8, 9, 10, 11, 12]),
                characteristics_entity_thrown: ThrownEntityCharacteristics::Custom(Box::new(EntityCharacteristics {                                
                    killer: true,
                    collide: false,
                    mass_min: RATIO_MASS * 1,
                    mass_max: RATIO_MASS * 20,
                    mass_evolution: None,
                    throw_entity: None,
                    inertia: 100,
                    ..Default::default()
                })),
                ..Default::default()
            }),
            on_death: Some(OnDeathEffect::Split(64)),
            ..Default::default()
        },
        timer: EntityTimer::default(),
        color: crate::game::settings::DEFAULT_COLOR[3],
        texture: 3,
    });
}