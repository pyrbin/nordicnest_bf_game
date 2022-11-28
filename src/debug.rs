use crate::prelude::*;
use bevy_prototype_debug_lines::DebugLines;
use std::f32::consts::{FRAC_PI_2, PI};

pub trait DebugLinesExt {
    fn circle(&mut self, origin: Vec3, radius: f32, duration: f32, color: Color);
    fn square(&mut self, origin: Vec3, size: f32, duration: f32, color: Color);
}

impl DebugLinesExt for DebugLines {
    fn circle(&mut self, origin: Vec3, radius: f32, duration: f32, color: Color) {
        add_circle(self, origin, radius, duration, color);
    }
    fn square(&mut self, origin: Vec3, size: f32, duration: f32, color: Color) {
        add_square(self, origin, size, duration, color);
    }
}

fn add_square(lines: &mut DebugLines, origin: Vec3, size: f32, duration: f32, color: Color) {
    let half_size = size / 2.0;
    let p1 = origin + Vec3::new(-half_size, 0.0, -half_size);
    let p2 = origin + Vec3::new(half_size, 0.0, -half_size);
    let p3 = origin + Vec3::new(half_size, 0.0, half_size);
    let p4 = origin + Vec3::new(-half_size, 0.0, half_size);
    lines.line_colored(p1, p2, duration, color);
    lines.line_colored(p2, p3, duration, color);
    lines.line_colored(p3, p4, duration, color);
    lines.line_colored(p4, p1, duration, color);
}

fn add_circle(lines: &mut DebugLines, origin: Vec3, radius: f32, duration: f32, color: Color) {
    let x_rotate = Quat::from_rotation_x(PI);
    add_semicircle(lines, origin, Quat::IDENTITY, radius, duration, color);
    add_semicircle(
        lines,
        origin,
        Quat::IDENTITY * x_rotate,
        radius,
        duration,
        color,
    );
}

fn add_semicircle(
    lines: &mut DebugLines,
    origin: Vec3,
    rot: Quat,
    radius: f32,
    duration: f32,
    color: Color,
) {
    let x_rotate = Quat::from_rotation_y(PI);
    add_quartercircle(lines, origin, rot, radius, duration, color);
    add_quartercircle(lines, origin, rot * x_rotate, radius, duration, color);
}

fn add_quartercircle(
    lines: &mut DebugLines,
    origin: Vec3,
    rot: Quat,
    radius: f32,
    duration: f32,
    color: Color,
) {
    let quarter_circle_segments = 4;
    let angle = FRAC_PI_2 / quarter_circle_segments as f32;
    let mut current_point = rot.mul_vec3(Vec3::X * radius);
    let direction = Quat::from_axis_angle(rot.mul_vec3(Vec3::Y), angle);
    for _ in 0..quarter_circle_segments {
        let next_point = direction.mul_vec3(current_point);
        lines.line_colored(origin + current_point, origin + next_point, duration, color);
        current_point = next_point;
    }
}
