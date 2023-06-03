pub mod map;
pub mod player;
pub mod entity;
pub mod gui;
pub mod settings;

pub use settings::Settings;
use map::{Map, MapInfo};
use player::{Player, PlayerInfo};
use entity::{EntityCore, Entities, EntityCharacteristics, EntityInfo};
use gui::Gui;

use crate::window::Window;

use benchmark::Benchmark;
//use benchmark::timer_node::Benchmark;
use buffer::BufferMulti;
use euclid::default::Point2D;
use euclid::default::Size2D;
use parking_lot::Mutex;
#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};
use quintuple_buffer::QuintupleBuffer;

use std::io::{Read, Write};
use std::sync::{RwLock, Weak};
use std::sync::mpsc::Sender;
use std::sync::atomic::AtomicUsize;
use std::time::Duration;
use std::time::Instant;

pub struct GameStep {
    pub actual_count: i32,
    pub duration_vec: Vec<Duration>,
    pub last_duration: Duration,
    pub changed_map: bool,
    pub full_speed: bool,
    pub waiting: Mutex<Option<(Instant, Duration, Duration, Sender<()>)>>,
}

impl Default for GameStep {
    fn default() -> GameStep {
        GameStep {
            actual_count: 0,
            //duration_vec: vec![20, 30, 50, 80, 100, 120, 150, 200, 300, 400, 500, 700, 1000, 1200, 1500, 1800, 2000].iter().map(|num| Duration::from_millis(*num)).collect(),
            duration_vec: { let mut vec = Vec::new(); let mut d = 20.0; while d < 2_000.0 { vec.push(d); d *= 2.0f64.sqrt().sqrt(); } vec.iter().map(| &d | Duration::from_secs_f64(d / 1_000.0)).collect() },
            last_duration: Duration::from_millis(20),
            changed_map: false,
            full_speed: false,
            waiting: Mutex::new(None),
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum GameState {
    MainMenu,
    Editor,
    Playing,
}
#[derive(Clone, Default)]
pub struct EditorState {
    pub new_entity: EntityInfo,
    pub new_entity_on_click: bool,
    pub tab: i32,
    pub selection: Option<Point2D<i32>>,
    pub selected: Option<Vec<usize>>,
    pub selected_atomic: Option<Vec<std::sync::Weak<AtomicUsize>>>,
    pub entity_selected: Option<usize>,
    pub entity_selected_atomic: Option<std::sync::Weak<AtomicUsize>>,
    pub entity_hovered: Option<usize>,
    pub entity_hovered_atomic: Option<std::sync::Weak<AtomicUsize>>,
    pub hovered: bool,
}

impl EditorState {
    pub fn reinit(&mut self) {
        self.selection.take();
        self.selected.take();
        self.entity_selected.take();
        self.entity_selected_atomic.take();
        self.entity_hovered.take();
        self.entity_hovered_atomic.take();
    }
}

pub enum GameAction { //TODO: change all BufferThing to ActionThing
    AddEntity(Box<EntityInfo>),
    KillEntity(Weak<AtomicUsize>), //TODO: just Arc<AtomicUsize> should be enough and better
}

#[derive(Default)]
pub struct SolverCache {
    pub collision: RwLock<Vec<&'static EntityCore>>,
}

pub struct GameInfo<'a> {
    pub window: &'a Window,
    pub threads: usize,
}

pub struct Game {
    pub settings: Settings,
    pub state: GameState,
    pub editor_state: EditorState,
    pub map: Map,
    pub players: Vec<Player>,
    pub entities: Entities, //TODO: Double Vec ? -> Avoid big allocation
    //pub entities: crate::utils::vec_chunk::VecChunk<Entity>,
    pub entities_characteristics: Vec<EntityCharacteristics>,
    pub gui: Gui,
    pub benchmark: Benchmark,
    pub threadpool: threadpool::ThreadPool, //TODO: not in Game
    //pub buffer: BufferMulti<GameAction>,
    pub buffer_add_entity: BufferMulti<Box<EntityInfo>>, //TODO: Refactor
    pub buffer_kill_entity: BufferMulti<Weak<AtomicUsize>>, //TODO: Refactor
    pub solver_cache: SolverCache,
    pub drawable: QuintupleBuffer<DrawableGame>,
    pub step: GameStep,
    pub id_generator: crate::utils::VecUniqueIndex, //TODO: Improve
}

unsafe impl Send for Game {} //TODO: bad
unsafe impl Sync for Game {} //TODO: bad

impl Game {
    pub fn new(info: GameInfo) -> Game {
        let settings = Settings::default();
        let state = GameState::Playing;
        let map = Map::new(MapInfo {
            size: Size2D::new(
                100,
                100,
            ),
        });
        let players = Vec::new();
        let entities = Entities::new();
        //let entities = crate::utils::vec_chunk::VecChunk::new();
        let entities_characteristics = Vec::new();
        let gui = Gui::new(info.window);
        let benchmark = Benchmark::new(2);
        let threadpool = threadpool::ThreadPool::new(info.threads);
        //let buffer = BufferMulti::with_capacity(10, 8);
        let buffer_add_entity = BufferMulti::with_capacity(10, 8);
        let buffer_kill_entity = BufferMulti::with_capacity(10, 8);
        let solver_cache = SolverCache::default();
        let drawable = QuintupleBuffer::new(DrawableGame::default());
        let step = GameStep::default();

        Game {
            settings,
            state,
            editor_state: Default::default(),
            map,
            players,
            entities,
            entities_characteristics,
            gui,
            benchmark,
            threadpool,
            //buffer,
            buffer_add_entity,
            buffer_kill_entity,
            solver_cache,
            drawable,
            step,
            id_generator: crate::utils::VecUniqueIndex::new(),
        }
    }

    pub fn init(&mut self) {
        //map::premade::create_premade_map(self, "example");
        map::premade::create_premade_map(self, "definitive");
    }

    pub fn new_player(&mut self, info: PlayerInfo) {
        let player = Player::new(info);
        self.players.push(player);
    }

    pub fn new_entity(&mut self, info: EntityInfo) { // TODO: Manage for multithreading
        let entities = unsafe { &* (&self.entities as *const Entities) };

        let index = self.entities.new_entity(&self.map, &info);
        let entity_core = &mut self.entities.core[index];
        entity_core.index.player = self.players[info.player].entities.len();
        entity_core.index.unique_id = self.id_generator.gen_id();
        self.map.add_entity(entities, entity_core);
        self.players[info.player].entities.push(index);
    }

    pub fn delete_entity(&mut self, index: usize) { // TODO: Manage for multithreading
        let game_bis = unsafe { &*(self as *const Game) };

        let entity = unsafe { &mut *(&mut self.entities.core[index] as *mut EntityCore) };
        let player = unsafe { &mut *(&mut self.players[entity.player] as *mut Player) };
        self.id_generator.remove(entity.index.unique_id);
        self.map.delete_entity(&self.entities, entity);
        player.entities.swap_remove(entity.index.player);
        if player.entities.len() != entity.index.player {
            self.entities.core[player.entities[entity.index.player]].index.player = entity.index.player;
        }
        self.entities.swap_remove(index);
        if self.entities.len() != index {
            let entity_moved = &mut self.entities.core[index];
            entity_moved.index.main = index;
            entity_moved.index.main_ptr.store(index, std::sync::atomic::Ordering::Relaxed);
            self.players[entity_moved.player].entities[entity_moved.index.player] = index;
            self.map.update_entity_index(&game_bis.entities, entity_moved);
        }
    }

    pub fn total_mass(&self) -> i64 {
        let mut total = 0;
        for &mass in self.entities.mass.iter() {
            total += mass;
        }
        return total;
    }

    pub fn clear(&mut self) { //TODO: more consistent clear     
        self.settings = Settings::default();
        self.map = Map::new(MapInfo {
            size: Size2D::new(
                100,
                100
            ),
        });
        self.players.clear();
        self.entities.clear();
        self.step.last_duration = self.step.duration_vec[0];
    }

    #[cfg(feature = "serialize")]
    pub fn to_binary_file(&mut self) {
        let game_serialized = GameSerialize::from_game(&self);
        let game_string = bincode::serialize(&game_serialized).unwrap();
        std::fs::File::create("maps/Map_test.binmap").unwrap().write(&game_string).unwrap();
    }

    #[cfg(feature = "serialize")]
    pub fn from_binary_file(&mut self) {
        let mut game_string = Vec::new();
        let mut game_file = std::fs::File::open("maps/Map_test.binmap").unwrap();
        game_file.read_to_end(&mut game_string).unwrap();
        let game_serialize: GameSerialize = bincode::deserialize(&game_string).unwrap();
        game_serialize.to_game(self);
    }

    #[cfg(feature = "serialize")]
    pub fn to_ron_file(&mut self) {
        let game_serialized = GameSerialize::from_game(&self);
        let game_string = ron::ser::to_string(&game_serialized).unwrap();
        std::fs::File::create("maps/Map_test.ronmap").unwrap().write(&game_string.as_bytes()).unwrap();
    }

    #[cfg(feature = "serialize")]
    pub fn from_ron_file(&mut self) {
        let mut game_string = Vec::new();
        let mut game_file = std::fs::File::open("maps/Map_test.ronmap").unwrap();
        game_file.read_to_end(&mut game_string).unwrap();
        let game_serialize: GameSerialize = ron::de::from_reader(game_string.as_slice()).unwrap();
        game_serialize.to_game(self);
    }
}
#[derive(Clone)]
pub struct DrawableGame {
    pub instant: std::time::Instant,
    pub update_count: i32,
    pub update_duration: std::time::Duration,
    pub state: GameState,
    pub editor_state: EditorState,
    pub local_player: PlayerInfo,
    pub entities: Vec<crate::game::entity::DrawableEntity>,
    //pub entities: crate::utils::vec_chunk::VecChunk<crate::game::entity::DrawableEntity>,
    pub lifetime: Vec<i32>,
    pub position: Vec<Point2D<i32>>,
    pub mass: Vec<f32>,
    pub background_color: [f32;4],
}

impl Default for DrawableGame {
    fn default() -> DrawableGame {
        DrawableGame {
            instant: std::time::Instant::now(),
            update_count: 0,
            update_duration: std::time::Duration::from_millis(20),
            state: GameState::Playing,
            editor_state: EditorState::default(),
            local_player: PlayerInfo::default(),
            entities: Vec::new(),
            //entities: crate::utils::vec_chunk::VecChunk::new(),
            lifetime: Vec::new(),
            position: Vec::new(),
            mass: Vec::new(),
            background_color: [0.0, 0.0, 0.0, 1.0],
        }
    }
}


#[cfg_attr(feature = "serialize", derive(Deserialize, Serialize, reflect::Reflect))]
pub struct GameSerialize {
    pub settings: Settings,
    pub players_info: Vec<PlayerInfo>,
    pub entities_info: Vec<EntityInfo>, //TODO: Change "usize" texture by the "String" texture name.. Or not!!!
    pub entities_characteristics: Vec<EntityCharacteristics>,
    pub map_info: MapInfo,
}

impl GameSerialize {
    fn from_game(game: &Game) -> GameSerialize {
        let settings = game.settings.clone();
        let mut players_info = Vec::new();
        for player in game.players.iter() {
            let player_info = PlayerInfo::from_player(player);
            players_info.push(player_info);
        }
        let mut entities_info = Vec::new();
        for entity in game.entities.core.iter() {
            let entity_info = EntityInfo::from_entity(game, entity.index.main);
            entities_info.push(entity_info);
        }
        let entities_characteristics = game.entities_characteristics.clone();
        let map_info = MapInfo {            
            size: game.map.size,
        };

        GameSerialize {
            settings,
            players_info,
            entities_info,
            entities_characteristics,
            map_info,
        }
    }
    
    fn to_game(&self, game: &mut Game) {
        game.settings = self.settings.clone();
        game.map = Map::new(self.map_info.clone());
        game.players.clear();
        for player_info in self.players_info.iter() {
            game.new_player(player_info.clone());
        }
        game.entities.clear();
        for entity_info in self.entities_info.iter() {
            game.new_entity(entity_info.clone());
        }
        game.entities_characteristics = self.entities_characteristics.clone();
    }
    
}