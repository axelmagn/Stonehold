use macroquad::time::get_frame_time;
use rapier2d::{
    crossbeam::{self, channel::Receiver},
    dynamics::{
        CCDSolver, ImpulseJointSet, IntegrationParameters, IslandManager, MultibodyJointSet,
        RigidBody, RigidBodyHandle, RigidBodySet,
    },
    geometry::{BroadPhase, ColliderSet, CollisionEvent, ContactForceEvent, NarrowPhase},
    math::{Real, Vector},
    pipeline::{ChannelEventCollector, PhysicsPipeline, QueryPipeline},
};

/// Game physics manager
#[derive(Default)]
pub struct Physics {
    pub bodies: RigidBodySet,
    pub colliders: ColliderSet,

    // simulation structures - generally not fussed with much
    pub gravity: Vector<Real>,
    pub integration_params: IntegrationParameters,
    pub physics_pipeline: PhysicsPipeline,
    pub islands: IslandManager,
    pub broad_phase: BroadPhase,
    pub narrow_phase: NarrowPhase,
    pub impulse_joints: ImpulseJointSet,
    pub multibody_joints: MultibodyJointSet,
    pub ccd_solver: CCDSolver,
    pub query_pipeline: QueryPipeline,
}

impl Physics {
    pub fn step(&mut self) -> (Receiver<CollisionEvent>, Receiver<ContactForceEvent>) {
        self.integration_params.dt = get_frame_time();

        let (collision_send, collision_recv) = crossbeam::channel::unbounded();
        let (contact_force_send, contact_force_recv) = crossbeam::channel::unbounded();
        let event_handler = ChannelEventCollector::new(collision_send, contact_force_send);

        self.physics_pipeline.step(
            &self.gravity,
            &self.integration_params,
            &mut self.islands,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.bodies,
            &mut self.colliders,
            &mut self.impulse_joints,
            &mut self.multibody_joints,
            &mut self.ccd_solver,
            // Some(&mut self.query_pipeline),
            None,
            &(),
            &event_handler,
        );

        (collision_recv, contact_force_recv)
    }

    pub fn remove_body(
        &mut self,
        body_handle: &RigidBodyHandle,
        remove_attached_colliders: bool,
    ) -> Option<RigidBody> {
        self.bodies.remove(
            *body_handle,
            &mut self.islands,
            &mut self.colliders,
            &mut self.impulse_joints,
            &mut self.multibody_joints,
            remove_attached_colliders,
        )
    }
}
