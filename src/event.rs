use rapier2d::geometry::CollisionEvent;

pub enum GameEvent {
    CollisionEvent(CollisionEvent),
}

pub trait EventHandler {
    fn handle_event(&mut self, event: &GameEvent);
}
