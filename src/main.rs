//TODO: Log crates
//TODO: Speedup single thread part with barrier lock for other thread;
//TODO: Occasional bug when quit (freeze and then error).

#[cfg(feature = "rpmalloc_feature")]
#[global_allocator]
static ALLOC: rpmalloc::RpMalloc = rpmalloc::RpMalloc;

#[cfg(not(feature = "shipping"))]
mod debug;

pub mod prelude;
mod app;
mod window;
mod game;
mod game_solver;
mod renderer;
mod utils;

pub static APP: static_data::StaticDataPtr<app::App> = static_data::StaticDataPtr::new();
#[cfg(not(feature = "shipping"))]
pub static DEBUG: static_data::StaticData<bool> = static_data::StaticData::new(false);
#[cfg(not(feature = "shipping"))]
pub static DEBUG_SETTINGS: static_data::StaticData<debug::DebugSettings> = static_data::StaticData::new(debug::DebugSettings::new());


fn main() {
    let mut app = app::App::new(app::AppInfo {
        threads: 2,
    });

    APP.set(&mut app);
    app.run();
    APP.set(std::ptr::null_mut());
}
