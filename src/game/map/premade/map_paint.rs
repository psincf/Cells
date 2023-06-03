use super::*;

use super::RATIO_MASS;

pub fn create(game: &mut Game) {
    game.map = Map::new(MapInfo {
        size: Size2D::new(50, 50),
    });

    super::helper_base(game);

    game.settings.auto_spawn.amount = 0;
    //game.settings.background_color = [1.0, 1.0, 1.0, 1.0];

    game.settings.local_player_food_settings.mass_minimum_to_throw = RATIO_MASS * 1 - 1;
    game.settings.local_player_food_settings.mass_self_added = 0;
    game.settings.local_player_food_settings.mass_entity_thrown = RATIO_MASS * 1;
    game.settings.local_player_food_settings.color = ThrownEntityColor::Custom(crate::game::settings::DEFAULT_COLOR[14]);
    game.settings.local_player_food_settings.texture = ThrownEntityTexture::CustomIndex(14);
    game.settings.local_player_characteristics.mass_min = RATIO_MASS * 1;
    game.settings.local_player_characteristics.mass_max = RATIO_MASS * 1;

    game.new_entity(EntityInfo {
        player: 1,
        position: Point2D::new(game.map.max().width / 2, (game.map.max().height / 10) * 9),
        speed: Vector2D::new(0.0, 0.0),
        mass: RATIO_MASS * 10,
        characteristics: game.settings.local_player_characteristics.clone(),
        timer: EntityTimer::default(),
        color: crate::game::settings::DEFAULT_COLOR[14],
        texture: 14,
    });

    for x in 0..200 {
        for y in 0..=1 {
            let y = if y == 0 { 0 } else { game.map.max().height };
            game.new_entity(EntityInfo {
                player: 0,
                position: Point2D::new((game.map.max().width / 200) * x, y),
                speed: Vector2D::new(0.0, 0.0),
                mass: RATIO_MASS * 10,
                characteristics: EntityCharacteristics::default(),
                timer: EntityTimer::default(),
                color: crate::game::settings::DEFAULT_COLOR[14],
                texture: 0,
            });
        }
    }

    for x in 0..=1 {
        for y in 0..200 {
            let x = if x == 0 { 0 } else { game.map.max().width };
            game.new_entity(EntityInfo {
                player: 0,
                position: Point2D::new(x, (game.map.max().height / 200) * y),
                speed: Vector2D::new(0.0, 0.0),
                mass: RATIO_MASS * 10,
                characteristics: EntityCharacteristics::default(),
                timer: EntityTimer::default(),
                color: crate::game::settings::DEFAULT_COLOR[14],
                texture: 0,
            });
        }
    }

}
