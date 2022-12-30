use winit::event::*;

pub struct Camera {
    pub position: glm::Vec3,
    pub world_up: glm::Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl Camera {
    pub fn new(
        position: glm::Vec3,
        world_up: glm::Vec3,
        target: glm::Vec3,
        aspect: f32,
        fovy: f32,
        znear: f32,
        zfar: f32,
    ) -> Self {
        let mut res = Camera {
            position,
            world_up,
            yaw: 0.0,
            pitch: 0.0,
            aspect: aspect,
            fovy,
            znear,
            zfar,
        };
        res.point_at(target);
        res
    }

    pub fn point_at(&mut self, target: glm::Vec3) {
        let dir = glm::normalize(&(target - self.position));
        self.yaw = dir.z.atan2(dir.x);
        self.pitch = dir.y.asin();
    }

    pub fn get_front(&self) -> glm::Vec3 {
        glm::vec3(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos()
        )
    }

    pub fn get_view_projection_matrix(&self) -> glm::Mat4 {
        use lazy_static::lazy_static;
        lazy_static! {
            pub static ref OPENGL_TO_WGPU_MATRIX: glm::Mat4 = glm::mat4(
                1.0, 0.0, 0.0, 0.0,
                0.0, 1.0, 0.0, 0.0,
                0.0, 0.0, 0.5, 0.0,
                0.0, 0.0, 0.5, 1.0,
            );
        }
        let view = glm::look_at(
            &self.position,
            &(self.position + self.get_front()),
            &self.world_up
        );
        let projection = glm::perspective(
            self.aspect,
            self.fovy,
            self.znear,
            self.zfar
        );
        *OPENGL_TO_WGPU_MATRIX * projection * view
    }
}

pub struct CameraController {
    move_speed: f32,
    mouse_sensitivity: f32,
    zoom_sensitivity: f32,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
    is_up_pressed: bool,
    is_down_pressed: bool,
    mouse_delta: (f32, f32),
    mouse_scroll_delta: f32,
}

impl CameraController {
    pub fn new(move_speed: f32, mouse_sensitivity: f32, zoom_sensitivity: f32) -> Self {
        CameraController {
            move_speed,
            mouse_sensitivity,
            zoom_sensitivity,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
            is_up_pressed: false,
            is_down_pressed: false,
            mouse_delta: (0.0, 0.0),
            mouse_scroll_delta: 0.0,
        }
    }

    pub fn process_event(&mut self, event: &DeviceEvent) -> bool {
        match event {
            DeviceEvent::Key(keyboard_input) => {
                let keycode = keyboard_input.virtual_keycode.unwrap();
                let is_pressed = keyboard_input.state == ElementState::Pressed;
                match keycode {
                    VirtualKeyCode::W => {
                        self.is_forward_pressed = is_pressed;
                        true
                    },
                    VirtualKeyCode::A => {
                        self.is_left_pressed = is_pressed;
                        true
                    },
                    VirtualKeyCode::S => {
                        self.is_backward_pressed = is_pressed;
                        true
                    },
                    VirtualKeyCode::D => {
                        self.is_right_pressed = is_pressed;
                        true
                    },
                    VirtualKeyCode::Space => {
                        self.is_up_pressed = is_pressed;
                        true
                    },
                    VirtualKeyCode::LControl => {
                        self.is_down_pressed = is_pressed;
                        true
                    },
                    _ => false,
                }
            },
            DeviceEvent::MouseMotion { delta } => {
                self.mouse_delta.0 += delta.0 as f32;
                self.mouse_delta.1 += delta.1 as f32;
                true
            },
            DeviceEvent::MouseWheel { delta } => {
                match delta {
                    MouseScrollDelta::LineDelta(_, delta_y) => {
                        self.mouse_scroll_delta += delta_y;
                        true
                    },
                    _ => false
                }
            }
            _ => false
        }
    }

    pub fn update_camera(&mut self, camera: &mut Camera) {
        let forward_dir = camera.get_front();
        let up_dir = camera.world_up;
        let right_dir = glm::cross(&forward_dir, &up_dir);
        
        // Translation
        let mut move_dir = glm::vec3(0.0, 0.0, 0.0);
        if self.is_forward_pressed {
            move_dir += forward_dir;
        }
        if self.is_backward_pressed {
            move_dir -= forward_dir;
        }
        if self.is_right_pressed {
            move_dir += right_dir;
        }
        if self.is_left_pressed {
            move_dir -= right_dir;
        }
        if glm::length(&move_dir) > 0.001 {
            move_dir = glm::normalize(&move_dir);
        }
        if self.is_up_pressed {
            move_dir += up_dir;
        }
        if self.is_down_pressed {
            move_dir -= up_dir;
        }
        camera.position += self.move_speed * move_dir;

        // Rotation
        camera.yaw += self.mouse_sensitivity * self.mouse_delta.0;
        camera.yaw %= 2.0 * std::f32::consts::PI;
        camera.pitch -= self.mouse_sensitivity * self.mouse_delta.1;
        camera.pitch = camera.pitch.clamp(-89.0_f32.to_radians(), 89.0_f32.to_radians());
        self.mouse_delta = (0.0, 0.0);

        // Zooming
        camera.fovy += self.zoom_sensitivity * self.mouse_scroll_delta;
        camera.fovy = camera.fovy.clamp(1.0_f32.to_radians(), 120.0_f32.to_radians());
        self.mouse_scroll_delta = 0.0;
    }
}
