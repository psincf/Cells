use super::*;

use super::RATIO_MASS;

pub fn create(game: &mut Game) {
    game.map = Map::new(MapInfo {
        size: Size2D::new(100, 100),
    });
    super::helper_base(game);
    game.settings.max_cells_spawn = 20_000;
    game.settings.local_player_food_settings.mass_self_added = -RATIO_MASS * 1;
    super::helper_new_entity_center(game);
}
