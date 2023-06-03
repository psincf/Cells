use crate::game::entity::EntityAction;
use crate::game::Game;

pub struct NoInteractionsSolver<'a> {
    game: &'a Game,
    entity_index: usize,
}

impl<'a> NoInteractionsSolver<'a> {
    #[inline]
    pub fn new(entity_index: usize, game: &'a Game) -> NoInteractionsSolver<'a> {
        NoInteractionsSolver {
            game,
            entity_index
        }
    }

    #[inline]
    pub fn solve(&self) {
        self.update_lifetime();
        self.update_timer();
        self.update_mass();
    }

    #[inline]
    pub fn update_lifetime(&self) {
        let entity_lifetime = &self.game.entities.lifetime[self.entity_index];
        let lifetime_ptr = entity_lifetime as *const i32 as *mut i32;
        unsafe { 
            *lifetime_ptr += 1;
            if *lifetime_ptr > 200 { *lifetime_ptr = 100 }
        }
    }
    
    #[inline]
    pub fn update_timer(&self) {
        let entity_timer = &self.game.entities.timer[self.entity_index];
        if entity_timer.mergeable.is_some() {
            self.game.entities.send_buffer(self.entity_index, EntityAction::AddMergeableTime(-1));
        }
        if entity_timer.inertia.is_some() {
            self.game.entities.send_buffer(self.entity_index, EntityAction::AddInertiaTime(-1));
        }
        if entity_timer.collision.is_some() {
            self.game.entities.send_buffer(self.entity_index, EntityAction::AddCollisionTime(-1));
        }
        if entity_timer.collision_ratio.is_some() {
            self.game.entities.send_buffer(self.entity_index, EntityAction::AddCollisionRatioTime(-1));
        }
        if let Some(lifetime) = entity_timer.lifetime_left { //TODO: So bad!!
            if lifetime == 0 {
                self.game.entities.send_buffer(self.entity_index, EntityAction::Killed(self.entity_index));
            } else {
                self.game.entities.send_buffer(self.entity_index, EntityAction::AddLifetimeLeftTime(-1));
            }
        }
    }

    #[inline]
    pub fn update_mass(&self) {
        //let entity_mass_evolution = self.game.entities.core[self.entity_index].characteristics.mass_evolution;
        let entity_mass_evolution = self.game.entities.mass_evolution[self.entity_index];
        if let Some(ratio) = entity_mass_evolution {
            let entity_mass = self.game.entities.mass[self.entity_index];
            if ratio == 1.0 { return }
            let new_mass = (entity_mass as f32 * ratio) as i64;
            self.game.entities.send_buffer(self.entity_index, EntityAction::AddMass(new_mass - entity_mass));
        }
    }
}