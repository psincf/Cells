use crate::game::{Game, GameInfo};
use crate::game_solver::GameSolver;
use crate::window::Window;
use crate::renderer::Renderer;
use app_trait::AppRunnerInfos;
use std::sync::atomic::AtomicBool;

pub struct AppInfo {
    pub threads: usize,
}

pub struct App {
    pub app_runner_infos: AppRunnerInfos, //TODO: Not very good. Find a better way
    pub window: Window,
    pub game: Game,
    pub renderer: Renderer,
    pub update_thread_running: AtomicBool,
}

impl App {
    pub fn new(info: AppInfo) -> App {
        let app_runner_infos = AppRunnerInfos::new();

        let window = Window::new();
        let game = Game::new(GameInfo {
            window: &window,
            threads: info.threads,
        });
        let renderer = Renderer::new(&window.window, &game);

        App {
            app_runner_infos,
            window,
            game,
            renderer,
            update_thread_running: AtomicBool::new(true),
        }
    }

    pub fn run(&mut self) {
        self.game.init();
        
        self.app_runner_infos.add_function(|| {
            let app = crate::APP.get_mut();
            app.update();
        });

        self.app_runner_infos.add_function(|| {
            let app = crate::APP.get_mut();
            app.draw();
        });

        self.app_runner_infos.run_multithread();
        self.check_events();
        self.app_runner_infos.stop();
        self.handle_closing();
        self.app_runner_infos.stop_and_wait();
    }

    fn check_events(&mut self) {
        self.window.poll_events(&self.game, &mut self.renderer);
    }

    fn update(&mut self) {
        let mut game_solver = GameSolver::new(&mut self.game, &mut self.window.events, self.renderer.camera.clone());
        game_solver.solve();

        if !self.app_runner_infos.running() {
            self.update_thread_running.store(false, std::sync::atomic::Ordering::Relaxed);
        }
    }

    fn draw(&mut self) {
        self.renderer.draw(&self.game, &self.window);
    }

    fn handle_closing(&mut self) {
        while self.update_thread_running.load(std::sync::atomic::Ordering::Relaxed) {
            let waiting_thread = self.game.step.waiting.lock();
            if let Some(thread) = waiting_thread.as_ref() {
                if thread.0.elapsed() > thread.1 && thread.0.elapsed() > thread.2 { thread.3.send(()).unwrap(); }
            }
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }
}