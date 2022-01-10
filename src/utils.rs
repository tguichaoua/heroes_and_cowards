use bevy::prelude::*;
use bevy_prototype_debug_lines::DebugLines;

pub trait DebugLinesExt {
    fn arrow_colored(&mut self, start: Vec3, end: Vec3, duration: f32, color: Color);
    fn circle_colored(&mut self, center: Vec3, radius: f32, duration: f32, color: Color);
}

impl DebugLinesExt for DebugLines {
    fn arrow_colored(&mut self, start: Vec3, end: Vec3, duration: f32, color: Color) {
        let d = (start - end).normalize_or_zero() * 20.0;
        let right = Quat::from_rotation_z(std::f32::consts::FRAC_PI_6) * d;
        let left = Quat::from_rotation_z(-std::f32::consts::FRAC_PI_6) * d;

        self.line_colored(start, end, duration, color);
        self.line_colored(end, end + right, duration, color);
        self.line_colored(end, end + left, duration, color);
    }
    fn circle_colored(&mut self, center: Vec3, radius: f32, duration: f32, color: Color) {
        const ANGLES: [f32; 21] = [
            0.0,
            0.3141592653589793,
            0.6283185307179586,
            0.9424777960769379,
            1.2566370614359172,
            1.5707963267948966,
            1.8849555921538759,
            2.199114857512855,
            2.5132741228718345,
            2.827433388230814,
            3.141592653589793,
            3.4557519189487724,
            3.7699111843077517,
            4.084070449666731,
            4.39822971502571,
            4.71238898038469,
            5.026548245743669,
            5.340707511102648,
            5.654866776461628,
            5.969026041820607,
            6.283185307179586,
        ];

        let p = radius * Vec3::X;
        for i in 0..20 {
            let a = center + Quat::from_rotation_z(ANGLES[i]) * p;
            let b = center + Quat::from_rotation_z(ANGLES[i + 1]) * p;
            self.line_colored(a, b, duration, color);
        }
    }
}
