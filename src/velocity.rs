use bevy::prelude::*;

// ===== components =====

/// The velocity of an entity.
/// The position of the entity is updated based on the value of the velocity
/// and the value of the [`Speed`] component.
#[derive(Default)]
pub struct Velocity(pub Vec3);

// ===== systems =====

/// Update the position of entities based on their [`Velocity`] and [`Speed`] components.
pub fn update_velocities(time: Res<Time>, mut q: Query<(&mut Transform, &Velocity)>) {
    for (mut transform, Velocity(velocity)) in q.iter_mut() {
        // calculate the new position of the agent
        transform.translation += *velocity * time.delta_seconds();
    }
}
