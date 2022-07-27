use crate::car::*;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use std::f32::consts::PI;

pub fn car_change_detection_system(
    query: Query<(Entity, &Car, &Velocity, &Transform), Changed<Car>>,
    mut front: Query<(&mut MultibodyJoint, With<WheelFront>)>,
    mut wheel_set: ParamSet<(
        Query<(&Wheel, &mut ExternalForce, &Transform, &Velocity), With<WheelFront>>,
        Query<(&Wheel, &mut ExternalForce, &Transform, &Velocity), With<WheelBack>>,
    )>,
) {
    for (_entity, car, velocity, transform) in query.iter() {
        let torque: f32;
        let car_mps = velocity.linvel.length();
        let braking = car.brake > 0.;

        let car_vector = transform.rotation.mul_vec3(Vec3::Z);
        let delta = velocity.linvel.normalize() - car_vector.normalize();
        let car_angle_slip_rad = Vec3::new(delta.x, 0., delta.z).length();
        let mut forward: bool = true;
        if car_angle_slip_rad > 1. {
            forward = false;
        }

        let break_max_torque = car.wheel_max_torque * 5.;
        if forward {
            if braking {
                torque = -car.brake * break_max_torque;
            } else {
                torque = car.gas * car.wheel_max_torque;
            }
        } else {
            if braking {
                torque = -car.brake * car.wheel_max_torque;
            } else {
                torque = car.gas * break_max_torque;
            }
        }
        let torque_vec = Vec3::new(0., torque, 0.);
        let torque_speed_x: f32 = match car_mps / 30. {
            x if x >= 1. => 0.,
            x => 1. - x,
        };
        let steering_speed_x: f32 = match car_mps / 10. {
            x if x >= 1. => 0.,
            x => 1. - x,
        }
        .powi(2);

        let max_angle = PI / 4.;
        let angle: f32 = max_angle * car.steering * (0.1 + 0.9 * steering_speed_x);
        let quat = Quat::from_axis_angle(Vec3::Y, -angle);
        let steering_torque_vec = quat.mul_vec3(torque_vec);
        let axis = quat.mul_vec3(Vec3::X);

        // let mut slip: Vec<f32> = vec![0.; 4];
        for (_i, wheel_entity) in car.wheels.iter().enumerate() {
            let mut q_front_wheels = wheel_set.p0();
            let wheel_result = q_front_wheels.get_mut(*wheel_entity);
            if let Ok((wheel, mut forces, transform, velocity)) = wheel_result {
                let radius_vel = velocity.angvel * wheel.radius;
                let velocity_slip = (
                    radius_vel[0] - velocity.linvel[2],
                    radius_vel[2] + velocity.linvel[0],
                );
                let slip_sq = (velocity_slip.0.powi(2) + velocity_slip.1.powi(2)).sqrt();
                let max_slip = 0.4;
                let slip_sq_x: f32 = match slip_sq / max_slip {
                    x if x >= 1. => {
                        // println!("max_slip {max_slip}, slip {slip_sq}");
                        0.
                    }
                    x => 1. - x,
                };
                // slip[i] = slip_sq_x;
                let total_torque = steering_torque_vec * slip_sq_x * torque_speed_x;
                forces.torque = (transform.rotation.mul_vec3(total_torque)).into();
            }

            if let Ok((wheel, mut forces, transform, velocity)) =
                wheel_set.p1().get_mut(*wheel_entity)
            {
                let radius_vel = velocity.angvel * wheel.radius;
                let velocity_slip = (
                    radius_vel[0] - velocity.linvel[2],
                    radius_vel[2] + velocity.linvel[0],
                );
                let slip_sq = (velocity_slip.0.powi(2) + velocity_slip.1.powi(2)).sqrt();
                let max_slip = 0.4;
                let slip_sq_x: f32 = match slip_sq / max_slip {
                    x if x >= 1. => {
                        // println!("max_slip {max_slip}, slip {slip_sq}");
                        0.
                    }
                    x => 1. - x,
                };
                // slip[i] = slip_sq_x;
                let total_torque = torque_vec * slip_sq_x * torque_speed_x;
                forces.torque = (transform.rotation.mul_vec3(total_torque)).into();
            }
            if let Ok((mut joint, _)) = front.get_mut(*wheel_entity) {
                joint.data.set_local_axis1(axis);
            }
        }
        // println!("slip {slip:?}");
    }
}