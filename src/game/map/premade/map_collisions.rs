use super::*;

use super::RATIO_MASS;

pub fn create(game: &mut Game) {
    game.map = Map::new(MapInfo {
        size: Size2D::new(100, 100),
    });
    game.settings.max_cells_spawn = 10_000;
    
    super::helper_base(game);


    game.settings.auto_spawn.amount = 100;
    //game.settings.auto_spawn.mass = RATIO_MASS * 10;
    //game.settings.auto_spawn.position = crate::game::settings::SpawnPosition::RandomRange(950_000..1_050_000, 950_000..1_050_000);
    //game.settings.auto_spawn.characteristics.collide = true;
    game.settings.auto_spawn.color = AutoSpawnEntityColor::Random(Vec::from(crate::game::settings::DEFAULT_COLOR_RANDOM));

    game.new_entity(EntityInfo {
        player: 1,
        position: Point2D::new(
            game.map.size.width * game.map.size_field / 2 - 100_000,
            game.map.size.height * game.map.size_field / 2,
        ),
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
        mass: RATIO_MASS * 2_000,
        characteristics: EntityCharacteristics {
            mass_min: RATIO_MASS * 1_000,
            mass_max: RATIO_MASS * 100_000,
            killer: true,
            on_death: Some(OnDeathEffect::Split(8)),
            throw_entity: Some(ThrowEntityInfo {
                mass_minimum_to_throw: RATIO_MASS * 2_000,
                mass_self_added: -RATIO_MASS * 5,
                mass_entity_thrown: RATIO_MASS * 10,
                throw_ratio: 0.1,
                power: 1_000..2_000,
                color: ThrownEntityColor::Random(Vec::from(crate::game::settings::DEFAULT_COLOR_RANDOM)),
                texture: ThrownEntityTexture::CustomIndex(2),
                //texture: ThrownEntityTexture::Random(vec![1, 2, 3, 4 , 5 , 6, 7, 8, 9, 10, 11, 12]),
                characteristics_entity_thrown: ThrownEntityCharacteristics::Custom(Box::new(EntityCharacteristics {                                
                    killer: false,
                    collide: true,
                    collide_when_mergeable: true,
                    mass_min: RATIO_MASS * 1,
                    mass_max: RATIO_MASS * 10_000,
                    mass_evolution: None,
                    throw_entity: None,
                    inertia: 30,
                    ..Default::default()
                })),
                ..Default::default()
            }),
            ..Default::default()
        },
        timer: EntityTimer {
            mergeable: None,
            ..Default::default()
        },
        color: crate::game::settings::DEFAULT_COLOR[3],
        texture: 3,
    });
}