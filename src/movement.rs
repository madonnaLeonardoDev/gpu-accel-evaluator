const CAM_X: usize = 0;
const CAM_Y: usize = 1;
const CAM_Z: usize = 2;
const CAM_YAW: usize = 3;
const CAM_PITCH: usize = 4;

const MAX_PITCH: f32 = std::f32::consts::FRAC_PI_2 - 0.01;

use crate::Scene;
pub trait Movement {
    fn forward_xz(&self) -> (f32, f32);

    fn right_xz(&self) -> (f32, f32);

    fn move_forward(&mut self, speed: f32, dt: f32);

    fn move_backward(&mut self, speed: f32, dt: f32);

    fn move_right(&mut self, speed: f32, dt: f32);
    fn move_left(&mut self, speed: f32, dt: f32);
    fn move_up(&mut self, speed: f32, dt: f32);
    fn move_down(&mut self, speed: f32, dt: f32);
    fn turn_left(&mut self, turn_speed: f32, dt: f32);
    fn turn_right(&mut self, turn_speed: f32, dt: f32);
    fn look_up(&mut self, turn_speed: f32, dt: f32);
    fn look_down(&mut self, turn_speed: f32, dt: f32);
}


impl Movement for Vec<f32> {
    fn forward_xz(&self) -> (f32, f32) {
    let yaw = self[CAM_YAW];
    (yaw.sin(), yaw.cos())
}

    fn right_xz(&self) -> (f32, f32) {
        let yaw = self[CAM_YAW];
        (yaw.cos(), -yaw.sin())
    }

    fn move_forward(&mut self, speed: f32, dt: f32) {
        let (fx, fz) = self.forward_xz();
        self[CAM_X] += fx * speed * dt;
        self[CAM_Z] += fz * speed * dt;
    }

    fn move_backward(&mut self, speed: f32, dt: f32) {
        self.move_forward(-speed, dt);
    }

    fn move_right(&mut self, speed: f32, dt: f32) {
        let (rx, rz) = self.right_xz();
        self[CAM_X] += rx * speed * dt;
        self[CAM_Z] += rz * speed * dt;
    }

    fn move_left(&mut self, speed: f32, dt: f32) {
        self.move_right(-speed, dt);
    }

    fn move_up(&mut self, speed: f32, dt: f32) {
        self[CAM_Y] += speed * dt;
    }

    fn move_down(&mut self, speed: f32, dt: f32) {
        self[CAM_Y] -= speed * dt;
    }

    fn turn_left(&mut self, turn_speed: f32, dt: f32) {
        self[CAM_YAW] -= turn_speed * dt;
    }

    fn turn_right(&mut self, turn_speed: f32, dt: f32) {
        self[CAM_YAW] += turn_speed * dt;
    }

    fn look_up(&mut self, turn_speed: f32, dt: f32) {
        self[CAM_PITCH] = (self[CAM_PITCH] + turn_speed * dt).clamp(-MAX_PITCH, MAX_PITCH);
    }

    fn look_down(&mut self, turn_speed: f32, dt: f32) {
        self[CAM_PITCH] = (self[CAM_PITCH] - turn_speed * dt).clamp(-MAX_PITCH, MAX_PITCH);
    }
}

