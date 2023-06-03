use super::*;

use super::RATIO_MASS;

pub fn create(game: &mut Game) {
    game.map = Map::new(MapInfo {
        size: Size2D::new(200, 200),
    });
    super::helper_base(game);
    super::helper_new_entity_up_left(game);
    game.settings.max_cells_spawn = 20_000;
    game.settings.auto_spawn.amount = 100;
    //game.settings.background_color = [0.0, 0.0, 0.0, 1.0];

    game.new_entity(EntityInfo {
        player: 0,
        position: Point2D::new(
            game.map.size.width * game.map.size_field / 2,
            game.map.size.height * game.map.size_field / 2
        ),
        speed: Vector2D::new(0.0, 0.0),
        mass: RATIO_MASS * 200,
        characteristics: EntityCharacteristics {
            killer: true,
            gravity: Some(EntityGravityInfo {
                power: 4.0,
                distance_clamp: (RATIO_POSITION as f32 * 5.0)..f32::MAX,
                ..Default::default()
            }),
            invincible: false,
            on_death: Some(OnDeathEffect::Split(8)),
            mass_min: RATIO_MASS * 100,
            mass_max: RATIO_MASS * 10_000_000,
            throw_entity: Some(ThrowEntityInfo {
                mass_minimum_to_throw: RATIO_MASS * 1_000,
                mass_self_added: -RATIO_MASS,
                mass_entity_thrown: RATIO_MASS,
                throw_ratio: 2.0,
                power: 10_000..20_000,
                color: ThrownEntityColor::Random(Vec::from(crate::game::settings::DEFAULT_COLOR_RANDOM)),
                texture: ThrownEntityTexture::Random(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]),
                characteristics_entity_thrown: ThrownEntityCharacteristics::Custom(Box::new(EntityCharacteristics {                                
                    killer: false,
                    collide: false,
                    mass_min: RATIO_MASS * 1,
                    mass_max: RATIO_MASS * 1,
                    mass_evolution: None,
                    throw_entity: None,
                    inertia: 20,
                    ..Default::default()
                })),
                ..Default::default()
            }),
            ..Default::default()
        },
        timer: EntityTimer::default(),
        color: crate::game::settings::DEFAULT_COLOR[0],
        texture: 0,
    });
    
    for x in 0..7 {
        if x == 0 || x == 7 { continue }
        for y in 0..=1 {
            let y = if y == 0 { 0.1 } else { 0.9 };
            game.new_entity(EntityInfo {
                player: 0,
                position: Point2D::new(
                    (game.map.size.width * game.map.size_field / 7) * x,
                    (((game.map.size.height * game.map.size_field ) as f32) * y) as i32,
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

    for x in 0..=1 {
        let x = if x == 0 { 0.1 } else { 0.9 };
        for y in 0..7 {
            if y == 0 || y == 7 { continue }
            game.new_entity(EntityInfo {
                player: 0,
                position: Point2D::new(
                    (((game.map.size.width * game.map.size_field ) as f32) * x) as i32,
                    (game.map.size.height * game.map.size_field / 7) * y
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


}
