use super::*;

use super::RATIO_MASS;

#[allow(dead_code)]
pub fn create(game: &mut Game) {
    game.map = Map::new(MapInfo {
        size: Size2D::new(200, 100),
    });
    game.settings.max_cells_spawn = 0;
    game.settings.local_player_characteristics = EntityCharacteristics {
        killer: true,
        collide: true,
        //gravity: Some(-10.0),
        mass_min: RATIO_MASS * 10,
        mass_max: RATIO_MASS * 100_000,
        mass_evolution: Some(0.9998),
        //can_split: true,
        throw_entity: None,
        inertia: 2,
        ..Default::default()
    };
    

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
        position: Point2D::new(
            game.map.size.width * game.map.size_field / 2,
            game.map.size.height * game.map.size_field,
        ),
        speed: Vector2D::new(0.0, 0.0),
        mass: RATIO_MASS * 10,
        characteristics: game.settings.local_player_characteristics.clone(),
        timer: EntityTimer::default(),
        color: crate::game::settings::DEFAULT_COLOR[14],
        texture: 14,
    });

    use rand::Rng;
    let mut rng = rand::thread_rng();

    for i in 0..200 {
        for j in 0..100 {
            game.new_entity(EntityInfo {
                player: 0,
                position: Point2D::new(
                    (300 - i) * game.map.size.width * game.map.size_field / 400,
                    (190 - j) * game.map.size.height * game.map.size_field / 200,
                ),
                speed: Vector2D::new(0.0, 0.0),
                mass: RATIO_MASS * 10,
                characteristics: EntityCharacteristics {
                    inertia: 100,
                    killer: false,
                    collide: true,
                    mass_min: RATIO_MASS * 10,
                    mass_max: RATIO_MASS * 100_000,
                    throw_entity: None,
                    ..Default::default()
                },
                timer: EntityTimer {
                    mergeable: None,
                    ..Default::default()
                },
                color: crate::game::settings::DEFAULT_COLOR_RANDOM[rng.gen_range(0..12)],
                texture: 3,
            });
        }
    }
}