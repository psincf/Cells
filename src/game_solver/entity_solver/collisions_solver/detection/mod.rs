pub mod solver;

use solver::CollisionsDetectionSolverInner;

use crate::prelude::*;

pub struct CollisionsDetectionSolver<'a> {
    entity: &'a EntityCore,
    game: &'a Game,
}

impl<'a> CollisionsDetectionSolver<'a> {
    pub fn new(entity: &'a EntityCore, game: &'a Game) -> CollisionsDetectionSolver<'a> {
        CollisionsDetectionSolver {
            entity,
            game,
        }
    }

    pub fn solve(&self) {
        CollisionsDetectionSolverInner::new(self.entity, self.game).solve_multi_matrix();
    }
}