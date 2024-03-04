use macroquad::time::get_frame_time;
use rapier2d::{
    dynamics::{
        CCDSolver, ImpulseJointSet, IntegrationParameters, IslandManager, MultibodyJointSet,
        RigidBodySet,
    },
    geometry::{BroadPhase, ColliderSet, NarrowPhase},
    math::{Real, Vector},
    pipeline::{PhysicsPipeline, QueryPipeline},
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
    pub fn step(&mut self) {
        self.integration_params.dt = get_frame_time();

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
            Some(&mut self.query_pipeline),
            &(),
            &(),
        )
    }
}
