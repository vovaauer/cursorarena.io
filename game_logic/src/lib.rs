use std::collections::HashMap;
use std::num::NonZero;
use rapier2d::prelude::*;
use rapier2d::control::KinematicCharacterController;
use serde::{Serialize, Deserialize};
use nalgebra::{Point2, Vector2};

pub type PlayerId = u32;

const GRABBABLE_USER_DATA: u128 = 1;
const DEATH_USER_DATA: u128 = 2;

const GROUP_WALLS: u32 = 1 << 0;
const GROUP_PLAYER: u32 = 1 << 1;
const GROUP_SQUARE: u32 = 1 << 2;

#[derive(Serialize, Deserialize, Debug)]
pub enum ShapeType {
    Square,
    Circle,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Boundary {
    pub x: f32,
    pub y: f32,
    pub half_width: f32,
    pub half_height: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Player {
    pub id: PlayerId,
    pub x: f32,
    pub y: f32,
    pub is_grabbing: bool,
    pub is_over_grabbable: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PhysicsObject {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
    pub shape: ShapeType,
    pub user_data: u128,
    pub half_width: Option<f32>,
    pub half_height: Option<f32>,
    pub radius: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GameState {
    pub players: Vec<Player>,
    pub objects: Vec<PhysicsObject>,
    pub boundaries: Vec<Boundary>,
}

#[derive(Clone, Deserialize)]
pub struct PlayerInput {
    pub mouse_dx: f32,
    pub mouse_dy: f32,
    pub is_mouse_down: bool,
}

pub struct Game {
    pub paused: bool,
    pub gravity: Vector2<f32>,
    pub integration_parameters: IntegrationParameters,
    pub physics_pipeline: PhysicsPipeline,
    pub island_manager: IslandManager,
    pub broad_phase: BroadPhase,
    pub narrow_phase: NarrowPhase,
    pub impulse_joint_set: ImpulseJointSet,
    pub multibody_joint_set: MultibodyJointSet,
    pub ccd_solver: CCDSolver,
    pub rigid_body_set: RigidBodySet,
    pub collider_set: ColliderSet,
    pub query_pipeline: QueryPipeline,
    pub character_controller: KinematicCharacterController,
    pub boundaries: Vec<Boundary>,
    pub substeps: u32,
    pub players: HashMap<PlayerId, ColliderHandle>,
    pub grab_joints: HashMap<PlayerId, ImpulseJointHandle>,
    player_inputs: HashMap<PlayerId, PlayerInput>,
}

#[derive(Serialize, Deserialize)]
pub struct MapData {
    pub gravity: Option<Vector2<f32>>,
    pub dimensions: Option<DimensionsData>,
    pub entities: Option<Vec<EntityData>>,
}

#[derive(Serialize, Deserialize)]
pub struct DimensionsData(pub f32, pub f32);

#[derive(Serialize, Deserialize)]
pub struct EntityData {
    pub shape: String,
    pub x1: Option<f32>,
    pub y1: Option<f32>,
    pub x2: Option<f32>,
    pub y2: Option<f32>,
    pub x: Option<f32>,
    pub y: Option<f32>,
    pub radius: Option<f32>,
    pub is_static: Option<bool>,
    pub is_death: Option<bool>,
    pub restitution: Option<f32>,
}

impl Game {
    pub fn new(map_data: Option<MapData>) -> Self {
        let mut rigid_body_set = RigidBodySet::new();
        let mut collider_set = ColliderSet::new();
        let mut integration_parameters = IntegrationParameters::default();
        
        let substeps = 10;
        integration_parameters.dt = (1.0 / 60.0) / (substeps as f32);

        integration_parameters.allowed_linear_error = 0.0005;
        integration_parameters.prediction_distance = 0.001;
        integration_parameters.min_ccd_dt = integration_parameters.dt;

        integration_parameters.num_solver_iterations = NonZero::new(8).unwrap();
        integration_parameters.num_additional_friction_iterations = 4;

        let (world_width, world_height) = if let Some(ref data) = map_data {
            if let Some(dims) = &data.dimensions {
                (dims.0, dims.1)
            } else {
                (16.0, 9.0)
            }
        } else {
            (16.0, 9.0)
        };

        let wall_thickness = 0.1;
        let half_width = world_width / 2.0;
        let half_height = world_height / 2.0;
        let mut boundaries = Vec::new();
        let wall_filter = InteractionGroups::new(GROUP_WALLS.into(), (GROUP_PLAYER | GROUP_SQUARE).into());

        let floor_pos = vector![0.0, -half_height];
        collider_set.insert(ColliderBuilder::cuboid(half_width, wall_thickness).translation(floor_pos).collision_groups(wall_filter).build());
        boundaries.push(Boundary { x: floor_pos.x, y: floor_pos.y, half_width, half_height: wall_thickness });

        let ceil_pos = vector![0.0, half_height];
        collider_set.insert(ColliderBuilder::cuboid(half_width, wall_thickness).translation(ceil_pos).collision_groups(wall_filter).build());
        boundaries.push(Boundary { x: ceil_pos.x, y: ceil_pos.y, half_width, half_height: wall_thickness });

        let left_pos = vector![-half_width, 0.0];
        collider_set.insert(ColliderBuilder::cuboid(wall_thickness, half_height).translation(left_pos).collision_groups(wall_filter).build());
        boundaries.push(Boundary { x: left_pos.x, y: left_pos.y, half_width: wall_thickness, half_height });

        let right_pos = vector![half_width, 0.0];
        collider_set.insert(ColliderBuilder::cuboid(wall_thickness, half_height).translation(right_pos).collision_groups(wall_filter).build());
        boundaries.push(Boundary { x: right_pos.x, y: right_pos.y, half_width: wall_thickness, half_height });

        let mut gravity = vector![0.0, -2.0];

        if let Some(ref data) = map_data {
            if let Some(g) = data.gravity {
                gravity = g;
            }

            if let Some(entities) = &data.entities {
                let square_filter = InteractionGroups::new(GROUP_SQUARE.into(), (GROUP_WALLS | GROUP_SQUARE | GROUP_PLAYER).into());
                for entity in entities {
                    let is_static = entity.is_static.unwrap_or(false);
                    let is_death = entity.is_death.unwrap_or(false);
                    let restitution = entity.restitution.unwrap_or(0.0);

                    let body_builder = if is_static {
                        RigidBodyBuilder::fixed()
                    } else {
                        RigidBodyBuilder::dynamic().ccd_enabled(true).linear_damping(0.5).angular_damping(0.8)
                    };
                    
                    let user_data = if is_death { DEATH_USER_DATA } else { GRABBABLE_USER_DATA };

                    let collider_builder = if entity.shape == "rect" {
                        let x1 = entity.x1.unwrap_or(0.0) * world_width - world_width / 2.0;
                        let y1 = entity.y1.unwrap_or(0.0) * world_height - world_height / 2.0;
                        let x2 = entity.x2.unwrap_or(0.0) * world_width - world_width / 2.0;
                        let y2 = entity.y2.unwrap_or(0.0) * world_height - world_height / 2.0;
                        let half_width = (x2 - x1) / 2.0;
                        let half_height = (y2 - y1) / 2.0;
                        ColliderBuilder::cuboid(half_width.abs(), half_height.abs())
                            .translation(vector![(x1 + x2) / 2.0, (y1 + y2) / 2.0])
                    } else if entity.shape == "circle" {
                        let x = entity.x.unwrap_or(0.0) * world_width - world_width / 2.0;
                        let y = entity.y.unwrap_or(0.0) * world_height - world_height / 2.0;
                        let radius = entity.radius.unwrap_or(0.1) * world_width;
                        ColliderBuilder::ball(radius).translation(vector![x, y])
                    } else {
                        continue;
                    };

                    let body = body_builder.user_data(user_data).build();
                    let collider = collider_builder.restitution(restitution).density(1.0).collision_groups(square_filter).build();
                    let handle = rigid_body_set.insert(body);
                    collider_set.insert_with_parent(collider, handle, &mut rigid_body_set);
                }
            }
        } else {
            // Default map creation
            let square_filter = InteractionGroups::new(GROUP_SQUARE.into(), (GROUP_WALLS | GROUP_SQUARE | GROUP_PLAYER).into());
            for i in 0..8 {
                for j in 0..5 {
                    let x = (i as f32 - 3.5) * 1.0;
                    let y = (j as f32 - 2.0) * 1.0;
                    let body = RigidBodyBuilder::dynamic().translation(vector![x, y]).user_data(GRABBABLE_USER_DATA).ccd_enabled(true).linear_damping(0.5).angular_damping(0.8).build();
                    let collider = ColliderBuilder::cuboid(0.3, 0.3).restitution(0.0).density(1.0).collision_groups(square_filter).build();
                    let handle = rigid_body_set.insert(body);
                    collider_set.insert_with_parent(collider, handle, &mut rigid_body_set);
                }
            }
        }

        Self {
            paused: false,
            gravity,
            integration_parameters,
            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: BroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            impulse_joint_set: ImpulseJointSet::new(),
            multibody_joint_set: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            rigid_body_set,
            collider_set,
            query_pipeline: QueryPipeline::new(),
            character_controller: KinematicCharacterController::default(),
            boundaries,
            substeps,
            players: HashMap::new(),
            grab_joints: HashMap::new(),
            player_inputs: HashMap::new(),
        }
    }

    pub fn add_player(&mut self, player_id: PlayerId) {
        let player_filter = InteractionGroups::new(GROUP_PLAYER.into(), GROUP_WALLS.into());
        let character_body = RigidBodyBuilder::kinematic_position_based().build();
        let character_handle = self.rigid_body_set.insert(character_body);
        let character_collider = ColliderBuilder::ball(0.000625).collision_groups(player_filter).build();
        let character_collider_handle = self.collider_set.insert_with_parent(character_collider, character_handle, &mut self.rigid_body_set);
        self.players.insert(player_id, character_collider_handle);
    }

    pub fn remove_player(&mut self, player_id: PlayerId) {
        if let Some(collider_handle) = self.players.remove(&player_id) {
            if let Some(collider) = self.collider_set.get(collider_handle) {
                if let Some(body_handle) = collider.parent() {
                    self.rigid_body_set.remove(body_handle, &mut self.island_manager, &mut self.collider_set, &mut self.impulse_joint_set, &mut self.multibody_joint_set, true);
                }
            }
        }
        self.grab_joints.remove(&player_id);
        self.player_inputs.remove(&player_id);
    }

    pub fn apply_input(&mut self, player_id: PlayerId, input: PlayerInput) {
        self.player_inputs.insert(player_id, input);
    }

    pub fn tick(&mut self) {
        if self.paused {
            return;
        }

        // Apply player inputs to move characters
        for (player_id, character_collider_handle) in &self.players {
            if let Some(input) = self.player_inputs.get(player_id) {
                let char_body_handle = self.collider_set[*character_collider_handle].parent().unwrap();
                let frame_translation = vector![input.mouse_dx, input.mouse_dy];
                
                let char_collider = &self.collider_set[*character_collider_handle];
                let current_position = *self.rigid_body_set[char_body_handle].translation();
                let filter = QueryFilter::default().groups(InteractionGroups::new(GROUP_PLAYER.into(), GROUP_WALLS.into()));

                let collision = self.character_controller.move_shape(
                    self.integration_parameters.dt * self.substeps as f32, // Full frame dt
                    &self.rigid_body_set, 
                    &self.collider_set, 
                    &self.query_pipeline,
                    char_collider.shape(), 
                    char_collider.position(), 
                    frame_translation, 
                    filter, 
                    |_| {}
                );
                if let Some(char_body) = self.rigid_body_set.get_mut(char_body_handle) {
                    char_body.set_next_kinematic_translation(current_position + collision.translation);
                }
            }
        }

        // Run the physics simulation in substeps
        for _ in 0..self.substeps {
            self.physics_pipeline.step(
                &self.gravity,
                &self.integration_parameters,
                &mut self.island_manager,
                &mut self.broad_phase,
                &mut self.narrow_phase,
                &mut self.rigid_body_set,
                &mut self.collider_set,
                &mut self.impulse_joint_set,
                &mut self.multibody_joint_set,
                &mut self.ccd_solver,
                None,
                &(),
                &(),
            );
        }

        // Handle grab logic once per frame, after physics has settled
        for (player_id, character_collider_handle) in &self.players {
            if let Some(input) = self.player_inputs.get(player_id) {
                let char_body_handle = self.collider_set[*character_collider_handle].parent().unwrap();
                let player_pos = self.rigid_body_set[char_body_handle].translation();
                let grab_point = Point2::new(player_pos.x, player_pos.y);
                let grab_filter = QueryFilter::default().groups(InteractionGroups::new(GROUP_PLAYER.into(), GROUP_SQUARE.into()));
                let mut hovered_object: Option<RigidBodyHandle> = None;

                self.query_pipeline.intersections_with_point(
                    &self.rigid_body_set, &self.collider_set, &grab_point, grab_filter,
                    |handle| {
                        if let Some(collider) = self.collider_set.get(handle) {
                            if let Some(body) = self.rigid_body_set.get(collider.parent().unwrap()) {
                                if body.user_data == GRABBABLE_USER_DATA {
                                    hovered_object = Some(collider.parent().unwrap());
                                    return false;
                                }
                            }
                        }
                        true
                    },
                );

                if input.is_mouse_down {
                    if self.grab_joints.get(player_id).is_none() {
                        if let Some(object_handle) = hovered_object {
                            let object_body = &self.rigid_body_set[object_handle];
                            let joint = RevoluteJointBuilder::new().local_anchor1(Point2::origin()).local_anchor2(object_body.position().inverse() * grab_point).build();
                            let handle = self.impulse_joint_set.insert(char_body_handle, object_handle, joint, true);
                            self.grab_joints.insert(*player_id, handle);
                        }
                    }
                } else {
                    if let Some(handle) = self.grab_joints.remove(player_id) {
                        self.impulse_joint_set.remove(handle, true);
                    }
                }
            }
        }

        self.player_inputs.clear();
        self.query_pipeline.update(&self.rigid_body_set, &self.collider_set);
    }

    pub fn get_game_state(&self) -> GameState {
        let mut objects = Vec::new();
        for (handle, body) in self.rigid_body_set.iter() {
            if self.players.values().any(|&h| h.into_raw_parts() == body.colliders()[0].into_raw_parts()) {
                continue;
            }

            for collider_handle in body.colliders() {
                if let Some(collider) = self.collider_set.get(*collider_handle) {
                    let (shape, half_width, half_height, radius) = match collider.shape().as_typed_shape() {
                        TypedShape::Cuboid(c) => (ShapeType::Square, Some(c.half_extents.x), Some(c.half_extents.y), None),
                        TypedShape::Ball(b) => (ShapeType::Circle, None, None, Some(b.radius)),
                        _ => continue,
                    };

                    let position = collider.position();

                    objects.push(PhysicsObject {
                        id: handle.into_raw_parts().0,
                        x: position.translation.x, 
                        y: position.translation.y, 
                        rotation: position.rotation.angle(),
                        shape,
                        user_data: body.user_data,
                        half_width,
                        half_height,
                        radius,
                    });
                }
            }
        }

        let mut players = Vec::new();
        for (player_id, collider_handle) in &self.players {
            if let Some(collider) = self.collider_set.get(*collider_handle) {
                if let Some(body) = self.rigid_body_set.get(collider.parent().unwrap()) {
                    let is_grabbing = self.grab_joints.contains_key(player_id);
                    let player_pos = body.translation();
                    let grab_point = Point2::new(player_pos.x, player_pos.y);
                    let grab_filter = QueryFilter::default().groups(InteractionGroups::new(GROUP_PLAYER.into(), GROUP_SQUARE.into()));
                    let mut is_over_grabbable = false;
                    self.query_pipeline.intersections_with_point(
                        &self.rigid_body_set, &self.collider_set, &grab_point, grab_filter,
                        |handle| {
                            if let Some(collider) = self.collider_set.get(handle) {
                                if let Some(body) = self.rigid_body_set.get(collider.parent().unwrap()) {
                                    if body.user_data == GRABBABLE_USER_DATA {
                                        is_over_grabbable = true;
                                        return false;
                                    }
                                }
                            }
                            true
                        },
                    );

                    players.push(Player {
                        id: *player_id,
                        x: body.translation().x,
                        y: body.translation().y,
                        is_grabbing,
                        is_over_grabbable,
                    });
                }
            }
        }

        GameState { 
            players, 
            objects, 
            boundaries: self.boundaries.clone(),
        }
    }

    pub fn pause(&mut self) {
        self.paused = !self.paused;
    }

    pub fn restart(&mut self) {
        for (_, body) in self.rigid_body_set.iter_mut() {
            if body.is_dynamic() {
                body.set_position(Isometry::identity(), true);
                body.set_linvel(vector![0.0, 0.0], true);
                body.set_angvel(0.0, true);
            }
        }
    }
}