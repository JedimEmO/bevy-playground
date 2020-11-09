use std::f32::consts::PI;
use std::ops::{Add, Sub};

use bevy::input::mouse::{MouseButtonInput, MouseMotion};
use bevy::prelude::*;

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup.system())
        .add_system(input_system.system())
        .add_system(velocity_system.system())
        .add_system(friction_system.system())
        .add_system(mouse_system.system())
        .add_system(fire_system.system())
        .add_system(kill_system.system())
        .run();
}

struct Lifespan {
    kill_at: f64,
}

struct Velocity {
    magnitude: Vec3,
    last_change: f64,
    no_friction: bool,
}

struct Shooter {
    pew_handle: Handle<ColorMaterial>,
    shoot_direction: Vec2,
    shoot_angle: f32,
    last_shot_at: f64,
}

fn setup(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>, asset_server: Res<AssetServer>) {
    let texture = asset_server.load("dude.png");
    let pew = asset_server.load("pew.png");

    commands
        .spawn(Camera2dComponents::default())
        .spawn(SpriteComponents {
            material: materials.add(texture.into()),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            ..Default::default()
        })
        .with(Velocity { magnitude: Default::default(), last_change: 0.0, no_friction: false })
        .with(Shooter {
            pew_handle: materials.add(pew.into()),
            shoot_direction: Default::default(),
            shoot_angle: 0.0,
            last_shot_at: 0.0,
        });
}

fn mouse_system(mut state: Local<EventReader<CursorMoved>>, events: Res<Events<CursorMoved>>, mut query: Query<(&mut Transform, &Velocity, &mut Shooter)>) {
    for event in state.iter(&events) {
        for (mut t, _, mut shooter) in query.iter_mut() {
            let view_dir_vec: Vec2 = (event.position - Vec2::new(1280.0 / 2.0, 400.0)) - Vec2::new(t.translation.x(), t.translation.y());
            let angle = view_dir_vec.angle_between(Vec2::new(1.0, 0.0));

            t.rotation = Quat::from_rotation_z(-angle - PI / 2.0);
            shooter.shoot_direction = view_dir_vec;
            shooter.shoot_angle = -angle;
        }
    }
}

fn kill_system(mut commands: Commands, time: Res<Time>, query: Query<(Entity, &Lifespan)>) {
    for (entity, lifespan) in query.iter() {
        if time.seconds_since_startup >= lifespan.kill_at {
            commands.despawn(entity);
        }
    }
}

fn fire_system(mut commands: Commands, time: Res<Time>, mut state: Local<EventReader<MouseButtonInput>>, events: Res<Events<MouseButtonInput>>, mut query: Query<(&mut Transform, &mut Shooter)>) {
    for event in state.iter(&events) {
        if event.button == MouseButton::Left {
            for (mut t, mut shooter) in query.iter_mut() {
                if time.seconds_since_startup - shooter.last_shot_at > 0.1 {
                    shooter.last_shot_at = time.seconds_since_startup;

                    let mut transform = Transform::from_rotation(Quat::from_rotation_z(shooter.shoot_angle));
                    let dir = Vec3::new(shooter.shoot_direction.x(), shooter.shoot_direction.y(), 0.0);

                    transform.translation = t.translation.clone() + dir.normalize() * 50.0;

                    commands.spawn(SpriteComponents {
                        material: shooter.pew_handle.clone(),
                        transform,
                        ..Default::default()
                    })
                        .with(Velocity { magnitude: dir.normalize() * 2000.0, last_change: 0.0, no_friction: true })
                        .with(Lifespan { kill_at: time.seconds_since_startup + 0.5 });
                }
            }
        }
    }
}

fn velocity_system(time: Res<Time>, mut query: Query<(&Velocity, &mut Transform)>) {
    for (velocity, mut transform) in query.iter_mut() {
        *transform.translation.x_mut() += velocity.magnitude.x() * time.delta_seconds;
        *transform.translation.y_mut() += velocity.magnitude.y() * time.delta_seconds;
    }
}

fn friction_system(time: Res<Time>, mut query: Query<(&mut Velocity)>) {
    for mut velocity in query.iter_mut() {
        if time.seconds_since_startup - velocity.last_change > 0.1 && !velocity.no_friction {
            if velocity.magnitude.length() > 0.0 {
                velocity.magnitude *= 0.7;
            }
        }
    }
}

fn input_system(time: Res<Time>, keyboard_input: Res<Input<KeyCode>>, mut query: Query<(&mut Velocity, &Shooter)>) {
    for (mut velocity, _) in query.iter_mut() {
        let mut dir = Vec3::zero();

        let accel = 5000.0;
        let max_speed = 500.0;

        if keyboard_input.pressed(KeyCode::A) {
            *dir.x_mut() -= 1.0 * time.delta_seconds;
        }
        if keyboard_input.pressed(KeyCode::D) {
            *dir.x_mut() += 1.0 * time.delta_seconds;
        }
        if keyboard_input.pressed(KeyCode::W) {
            *dir.y_mut() += 1.0 * time.delta_seconds;
        }
        if keyboard_input.pressed(KeyCode::S) {
            *dir.y_mut() -= 1.0 * time.delta_seconds;
        }

        if dir.length() > 0.0 {
            dir = dir.normalize() * accel * time.delta_seconds;

            *velocity.magnitude.x_mut() += dir.x();
            *velocity.magnitude.y_mut() += dir.y();

            if velocity.magnitude.length() > max_speed {
                velocity.magnitude = velocity.magnitude.normalize() * max_speed;
            }

            velocity.last_change = time.seconds_since_startup
        }
    }
}
