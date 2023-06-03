use super::*;

pub fn create(game: &mut Game) {
    game.map = Map::new(MapInfo {
        //size: Size2D::new(80, 130),
        size: Size2D::new(40, 65),
    });

    super::helper_base(game);
    game.settings.local_player_characteristics.killer = true;
    game.settings.local_player_characteristics.mass_min = RATIO_MASS * 1;
    game.settings.local_player_characteristics.mass_max = RATIO_MASS * 100;

    game.settings.camera_initial = 50.0;
    game.settings.max_camera = 625.0;

    game.new_entity(EntityInfo {
        player: 1,
        //position: Point2D::new(400_000, 1_250_000),
        position: Point2D::new(200_000, 625_000),
        speed: Vector2D::new(0.0, 0.0),
        mass: RATIO_MASS * 100,
        characteristics: game.settings.local_player_characteristics.clone(),
        timer: EntityTimer::default(),
        color: game.players[1].cell_default_color,
        texture: game.players[1].cell_default_texture,
    });

    game.settings.max_cells_spawn = 0;
    /*
    let mut file = std::fs::File::open("Mona Lisa.jpg").unwrap();

    let mut image_data: Vec<u8> = Vec::new();
    file.read_to_end(&mut image_data).unwrap();

    let image = image::load_from_memory(&image_data).unwrap();
    */
    //let image = image::load_from_memory(include_bytes!("../../../../assets/textures/Mona Lisa.jpg")).unwrap();
    let image = image::load_from_memory(include_bytes!("../../../../assets/textures/Mona Lisa 2.jpg")).unwrap();

    let image = image.into_rgba8();

    for (x, y, pixel) in image.enumerate_pixels() {
        game.new_entity(EntityInfo {
            player: 0,
            position: Point2D::new(x as i32 * 1_000, y as i32 * 1_000),
            speed: Vector2D::zero(),
            mass: RATIO_MASS * 1,
            characteristics: EntityCharacteristics::default(),
            timer: EntityTimer::default(),
            color: EntityColor {
                center: [pixel[0], pixel[1], pixel[2], pixel[3]],
                edge: [pixel[0], pixel[1], pixel[2], pixel[3]],
            },
            texture: 0,
        });
    }
}