use super::*;

use super::RATIO_MASS;

pub fn create(game: &mut Game) {
    game.map = Map::new(MapInfo {
        size: Size2D::new(50, 50),
    });
    
    super::helper_base(game);

    game.settings.max_cells_spawn = 0;
    game.settings.local_player_characteristics.throw_entity = Some(ThrowEntityInfo {
        mass_minimum_to_throw: RATIO_MASS * 100,
        mass_self_added: -RATIO_MASS * 2,
        mass_entity_thrown: RATIO_MASS * 2,
        throw_ratio: 0.2,
        power: 500..1_000,
        color: ThrownEntityColor::Same,
        texture: ThrownEntityTexture::Same,
        characteristics_entity_thrown: ThrownEntityCharacteristics::Custom(Box::new(EntityCharacteristics {
            ..Default::default()
        })),
        ..Default::default()
    });
    /*
    game.settings.auto_spawn.amount = 50;
    game.settings.auto_spawn.position = SpawnPosition::Exact(game.map.size.width * game.map.size_field / 2, game.map.size.height * game.map.size_field / 2);
    */
    game.new_player(PlayerInfo {
        kind: PlayerKind::Neutral,
        entities: Vec::new(),
        ..Default::default()
    });
    game.new_player(PlayerInfo {
        kind: PlayerKind::Player,
        entities: Vec::new(),
        cell_default_color: EntityColor { center: [255, 255, 255, 255], edge: [0, 0, 0, 255] },
        cell_default_texture: 14,
    });

    game.new_entity(EntityInfo {
        player: 1,
        position: Point2D::new(0, 0),
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
        mass: RATIO_MASS * 100_000,
        characteristics: EntityCharacteristics {
            mass_min: RATIO_MASS * 500,
            mass_max: RATIO_MASS * 100_000,
            killer: true,
            on_death: Some(OnDeathEffect::Split(8)),
            throw_entity: Some(ThrowEntityInfo {                            
                mass_minimum_to_throw: RATIO_MASS * 1_000,
                mass_self_added: -RATIO_MASS * 1,
                mass_entity_thrown: RATIO_MASS * 1,
                throw_ratio: 1.0,
                power: 100..10_000,
                color: ThrownEntityColor::Random(Vec::from(crate::game::settings::DEFAULT_COLOR_RANDOM)),
                texture: ThrownEntityTexture::Random(vec![1, 2, 3, 4 , 5 , 6, 7, 8, 9, 10, 11, 12]),
                characteristics_entity_thrown: ThrownEntityCharacteristics::Custom(Box::new(EntityCharacteristics {                                
                    killer: false,
                    collide: false,
                    mass_min: RATIO_MASS * 1,
                    mass_max: RATIO_MASS * 100,
                    mass_evolution: None,
                    throw_entity: None,
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
