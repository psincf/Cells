use super::*;

use std::ops::Range;

pub fn create(game: &mut Game) {
    helper_base(game);
    game.map = Map::new(
        MapInfo {
            size: Size2D::new(
                1_000,
                1_000
            )
        }
    );
    let colors = generate_entity_colors(0.0..1.0, 0..1, 0..255, (1, 1, 120));

    game.settings.max_cells_spawn = 100_000;
    game.settings.max_camera = 10_000.0;

    game.settings.auto_spawn.amount = 1_000;
    game.settings.auto_spawn.color = AutoSpawnEntityColor::Random(colors);

    game.settings.special = true;
    
    generate_red_big_cell(game);
    generate_magenta_big_cell(game);
    generate_blue_big_cell(game);
    generate_light_blue_big_cell(game);
    generate_green_big_cell(game);
    generate_yellow_big_cell(game);

    for i in 0..6 {
        let i_f32 = i as f32;
        let position = position_medium_cell(game, i as f32 * 60.0);
        let color = generate_entity_colors((-i_f32 * 60.0 - 90.0)..(-i_f32 * 60.0 - 90.0 + 1.0), 254..255, 254..255, (1, 1, 1));
        game.new_entity(EntityInfo {
            player: 0,
            position: position,
            speed: Vector2D::new(0.0, 0.0),
            mass: RATIO_MASS * 10_000,
            timer: EntityTimer::default(),
            color: color[0],
            texture: 0,
            characteristics: EntityCharacteristics {
                ..Default::default()
            }
        });
    }


    helper_new_entity_center(game);
}

fn generate_entity_colors(h_range: Range<f32>, s_range: Range<u8>, v_range: Range<u8>, amount_hsv: (i32, i32, i32) ) -> Vec<EntityColor> {
    let mut vec = Vec::new();
    for i in 0..amount_hsv.0 {
        let h = h_range.start + ((h_range.end - h_range.start) / (amount_hsv.0 as f32 + 1.0)) * i as f32;
        for j in 0..amount_hsv.1 {
            let s = s_range.start + ((s_range.end - s_range.start) / (amount_hsv.1 as u8 + 1)) * j as u8;
            for k in 0..amount_hsv.2 {
                let v = v_range.start + ((v_range.end - v_range.start) / (amount_hsv.2 as u8 + 1)) * k as u8;
                let color = crate::game::settings::colors::from_hsv_to_rgb((h, s, v));
                vec.push([color.0, color.1, color.2, 255]);
            }
        }
    }
    vec.iter().map(|rgb|
        EntityColor { center: [rgb[0], rgb[1], rgb[2], 255], edge: [rgb[0], rgb[1], rgb[2], 255] }
    ).collect()
}

fn position_medium_cell(game: &mut Game, degrees: f32) -> Point2D<i32> {
    let position = euclid::default::Vector2D::from_angle_and_length(
        euclid::Angle::degrees(degrees), 1_000.0)
        .to_i32()
        .to_point() * 2_000
        + Vector2D::new(game.map.size.width * RATIO_POSITION / 2, game.map.size.height * RATIO_POSITION / 2
    );
    
    return position
}

fn position_big_cell(game: &mut Game, degrees: f32) -> Point2D<i32> {
    let position = euclid::default::Vector2D::from_angle_and_length(
        euclid::Angle::degrees(degrees), 1_000.0)
        .to_i32()
        .to_point() * 4_000
        + Vector2D::new(game.map.size.width * RATIO_POSITION / 2, game.map.size.height * RATIO_POSITION / 2
    );
    
    return position
}

fn generate_red_big_cell(game: &mut Game) {
    let red_colors = generate_entity_colors(-30.0..30.0, 254..255, 254..255, (120, 1, 1));
    game.new_entity(EntityInfo {
        player: 0,
        position: Point2D::new(5_000_000, 1_000_000),
        speed: Vector2D::new(0.0, 0.0),
        mass: RATIO_MASS * 100_000,
        timer: EntityTimer::default(),
        color: EntityColor { center: [240, 15, 15, 255], edge: [240, 15, 15, 255] },
        texture: 0,
        characteristics: EntityCharacteristics {
            killer: true,
            mergeable: true,
            affected_by_gravity: false,
            bounce: true,
            can_split_on_kill: false,
            invincible: false,
            inertia: 10,
            mass_min: RATIO_MASS * 100_000,
            mass_max: RATIO_MASS * 1_000_000,
            mass_evolution: None,
            //on_death: Option<OnDeathEffect>,
            gravity: None,
            throw_entity: Some(ThrowEntityInfo {
                color: ThrownEntityColor::Random(red_colors),
                direction: 0.0..360.0,
                mass_minimum_to_throw: RATIO_MASS * 100_001,
                mass_entity_thrown: RATIO_MASS * 1,
                mass_self_added: - RATIO_MASS / 2,
                power: 500..10_000,
                throw_ratio: 0.1,
                //timer_entity_thrown: 
                characteristics_entity_thrown: ThrownEntityCharacteristics::Custom(Box::new(EntityCharacteristics {
                    ..Default::default()
                })),
                ..Default::default()
            }),
            //special: Vec<EntitySpecial>,
            ..Default::default()
        },
    });
}

fn generate_magenta_big_cell(game: &mut Game) {
    let magenta_colors = generate_entity_colors(-90.0..-30.0, 254..255, 254..255, (120, 1, 1));
    let position = position_big_cell(game, -30.0);
    game.new_entity(EntityInfo {
        player: 0,
        //position: Point2D::new(7_500_000, 2_500_000),
        position: position,
        speed: Vector2D::new(0.0, 0.0),
        mass: RATIO_MASS * 100_000,
        timer: EntityTimer::default(),
        color: EntityColor { center: [240, 15, 240, 255], edge: [240, 15, 240, 255] },
        texture: 0,
        characteristics: EntityCharacteristics {
            killer: true,
            mergeable: true,
            affected_by_gravity: false,
            bounce: true,
            can_split_on_kill: false,
            invincible: false,
            inertia: 10,
            mass_min: RATIO_MASS * 100_000,
            mass_max: RATIO_MASS * 1_000_000,
            mass_evolution: None,
            //on_death: Option<OnDeathEffect>,
            gravity: None,
            throw_entity: Some(ThrowEntityInfo {
                color: ThrownEntityColor::Random(magenta_colors),
                direction: 0.0..360.0,
                mass_minimum_to_throw: RATIO_MASS * 100_001,
                mass_entity_thrown: RATIO_MASS * 1,
                mass_self_added: - RATIO_MASS / 2,
                power: 500..10_000,
                throw_ratio: 0.1,
                //timer_entity_thrown: 
                characteristics_entity_thrown: ThrownEntityCharacteristics::Custom(Box::new(EntityCharacteristics {
                    ..Default::default()
                })),
                ..Default::default()
            }),
            //special: Vec<EntitySpecial>,
            ..Default::default()
        },
    });
}

fn generate_blue_big_cell(game: &mut Game) {
    let magenta_colors = generate_entity_colors(-150.0..-90.0, 254..255, 254..255, (120, 1, 1));
    let position = position_big_cell(game, 30.0);
    game.new_entity(EntityInfo {
        player: 0,
        //position: Point2D::new(7_500_000, 2_500_000),
        position: position,
        speed: Vector2D::new(0.0, 0.0),
        mass: RATIO_MASS * 100_000,
        timer: EntityTimer::default(),
        color: EntityColor { center: [15, 15, 240, 255], edge: [15, 15, 240, 255] },
        texture: 0,
        characteristics: EntityCharacteristics {
            killer: true,
            mergeable: true,
            affected_by_gravity: false,
            bounce: true,
            can_split_on_kill: false,
            invincible: false,
            inertia: 10,
            mass_min: RATIO_MASS * 100_000,
            mass_max: RATIO_MASS * 1_000_000,
            mass_evolution: None,
            //on_death: Option<OnDeathEffect>,
            gravity: None,
            throw_entity: Some(ThrowEntityInfo {
                color: ThrownEntityColor::Random(magenta_colors),
                direction: 0.0..360.0,
                mass_minimum_to_throw: RATIO_MASS * 100_001,
                mass_entity_thrown: RATIO_MASS * 1,
                mass_self_added: - RATIO_MASS / 2,
                power: 500..10_000,
                throw_ratio: 0.1,
                //timer_entity_thrown: 
                characteristics_entity_thrown: ThrownEntityCharacteristics::Custom(Box::new(EntityCharacteristics {
                    ..Default::default()
                })),
                ..Default::default()
            }),
            //special: Vec<EntitySpecial>,
            ..Default::default()
        },
    });
}


fn generate_light_blue_big_cell(game: &mut Game) {
    let magenta_colors = generate_entity_colors(-210.0..-150.0, 254..255, 254..255, (120, 1, 1));
    let position = position_big_cell(game, 90.0);
    game.new_entity(EntityInfo {
        player: 0,
        //position: Point2D::new(7_500_000, 2_500_000),
        position: position,
        speed: Vector2D::new(0.0, 0.0),
        mass: RATIO_MASS * 100_000,
        timer: EntityTimer::default(),
        color: EntityColor { center: [15, 240, 240, 255], edge: [15, 240, 240, 255] },
        texture: 0,
        characteristics: EntityCharacteristics {
            killer: true,
            mergeable: true,
            affected_by_gravity: false,
            bounce: true,
            can_split_on_kill: false,
            invincible: false,
            inertia: 10,
            mass_min: RATIO_MASS * 100_000,
            mass_max: RATIO_MASS * 1_000_000,
            mass_evolution: None,
            //on_death: Option<OnDeathEffect>,
            gravity: None,
            throw_entity: Some(ThrowEntityInfo {
                color: ThrownEntityColor::Random(magenta_colors),
                direction: 0.0..360.0,
                mass_minimum_to_throw: RATIO_MASS * 100_001,
                mass_entity_thrown: RATIO_MASS * 1,
                mass_self_added: - RATIO_MASS / 2,
                power: 500..10_000,
                throw_ratio: 0.1,
                //timer_entity_thrown: 
                characteristics_entity_thrown: ThrownEntityCharacteristics::Custom(Box::new(EntityCharacteristics {
                    ..Default::default()
                })),
                ..Default::default()
            }),
            //special: Vec<EntitySpecial>,
            ..Default::default()
        },
    });
}

fn generate_green_big_cell(game: &mut Game) {
    let magenta_colors = generate_entity_colors(-270.0..-210.0, 254..255, 254..255, (120, 1, 1));
    let position = position_big_cell(game, 150.0);
    game.new_entity(EntityInfo {
        player: 0,
        //position: Point2D::new(7_500_000, 2_500_000),
        position: position,
        speed: Vector2D::new(0.0, 0.0),
        mass: RATIO_MASS * 100_000,
        timer: EntityTimer::default(),
        color: EntityColor { center: [15, 240, 15, 255], edge: [15, 240, 15, 255] },
        texture: 0,
        characteristics: EntityCharacteristics {
            killer: true,
            mergeable: true,
            affected_by_gravity: false,
            bounce: true,
            can_split_on_kill: false,
            invincible: false,
            inertia: 10,
            mass_min: RATIO_MASS * 100_000,
            mass_max: RATIO_MASS * 1_000_000,
            mass_evolution: None,
            //on_death: Option<OnDeathEffect>,
            gravity: None,
            throw_entity: Some(ThrowEntityInfo {
                color: ThrownEntityColor::Random(magenta_colors),
                direction: 0.0..360.0,
                mass_minimum_to_throw: RATIO_MASS * 100_001,
                mass_entity_thrown: RATIO_MASS * 1,
                mass_self_added: - RATIO_MASS / 2,
                power: 500..10_000,
                throw_ratio: 0.1,
                //timer_entity_thrown: 
                characteristics_entity_thrown: ThrownEntityCharacteristics::Custom(Box::new(EntityCharacteristics {
                    ..Default::default()
                })),
                ..Default::default()
            }),
            //special: Vec<EntitySpecial>,
            ..Default::default()
        },
    });
}

fn generate_yellow_big_cell(game: &mut Game) {
    let magenta_colors = generate_entity_colors(-330.0..-270.0, 254..255, 254..255, (120, 1, 1));
    let position = position_big_cell(game, 210.0);
    game.new_entity(EntityInfo {
        player: 0,
        //position: Point2D::new(7_500_000, 2_500_000),
        position: position,
        speed: Vector2D::new(0.0, 0.0),
        mass: RATIO_MASS * 100_000,
        timer: EntityTimer::default(),
        color: EntityColor { center: [240, 240, 15, 255], edge: [240, 240, 15, 255] },
        texture: 0,
        characteristics: EntityCharacteristics {
            killer: true,
            mergeable: true,
            affected_by_gravity: false,
            bounce: true,
            can_split_on_kill: false,
            invincible: false,
            inertia: 10,
            mass_min: RATIO_MASS * 100_000,
            mass_max: RATIO_MASS * 1_000_000,
            mass_evolution: None,
            //on_death: Option<OnDeathEffect>,
            gravity: None,
            throw_entity: Some(ThrowEntityInfo {
                color: ThrownEntityColor::Random(magenta_colors),
                direction: 0.0..360.0,
                mass_minimum_to_throw: RATIO_MASS * 100_001,
                mass_entity_thrown: RATIO_MASS * 1,
                mass_self_added: - RATIO_MASS / 2,
                power: 500..10_000,
                throw_ratio: 0.1,
                //timer_entity_thrown: 
                characteristics_entity_thrown: ThrownEntityCharacteristics::Custom(Box::new(EntityCharacteristics {
                    ..Default::default()
                })),
                ..Default::default()
            }),
            //special: Vec<EntitySpecial>,
            ..Default::default()
        },
    });
}