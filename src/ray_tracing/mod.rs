use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
#[cfg(feature = "rayon")]
use rayon::iter::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};
use std::f32;
use std::ops::{Div, Rem};
use std::panic;
use std::time::Duration;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::ray_tracing::action::ActionManager;
use crate::ray_tracing::vector::Vec3;
use crate::time::Instant;

pub mod action;
pub mod vector;

#[wasm_bindgen(start)]
fn main() {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
}

#[inline]
#[must_use]
pub const fn rgb(r: u8, g: u8, b: u8) -> u32 {
    0xFF000000 | (((r as u32) << 16) + ((g as u32) << 8) + b as u32)
}

#[inline]
#[must_use]
pub const fn rgbf(r: f32, g: f32, b: f32) -> u32 {
    rgb((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
}

/// 输入为 0xAARRGGBB, 输出为 [0xRR, 0xGG, 0xBB, 0xAA]
#[cfg(target_arch = "wasm32")]
#[inline]
#[must_use]
const fn to_web_color(color: u32) -> [u8; 4] {
    [
        (color >> 16) as u8,
        (color >> 8) as u8,
        (color & 0xff) as u8,
        (color >> 24) as u8,
    ]
}

#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntersectKind {
    Sky,
    Ground,
    Sphere,
}

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct Intersect {
    distance: f32,
    hit_point: Option<Vec3>,
    normal: Option<Vec3>,
    kind: IntersectKind,
}

/// 会镜面反射的球体.
#[wasm_bindgen]
#[derive(Debug, Clone, Copy)]
pub struct Sphere {
    center: Vec3,
    /// 球体半径 (米)
    radius: f32,
}

#[wasm_bindgen]
impl Sphere {
    #[must_use]
    pub fn new(center: Vec3, radius: f32) -> Self {
        Self { center, radius }
    }

    /// 球体和从某个点射出的光线求交. 起点在球面内上外估计都能正常计算.
    #[must_use]
    pub fn intersect(self, origin: Vec3, direction: Vec3) -> Option<Intersect> {
        let v = origin - self.center;
        let b = direction.dot(v);
        let c = v.dot(v) - self.radius * self.radius;
        let frac_descriminant_4 = b * b - c;
        if frac_descriminant_4 > 0.0 {
            let mut distance = -b - frac_descriminant_4.sqrt();
            let intersect_point = direction * distance + origin;
            let mut normal = (intersect_point - self.center).normalize();
            if distance < 0.0 {
                // 从球体内部射出的光线, 上面计算的距离可能是负数.
                distance = -b + frac_descriminant_4.sqrt();
                if distance < 0.0 {
                    return None;
                }
                normal = -normal; // 法向量向内.
            }
            let intersect = Intersect {
                distance,
                hit_point: Some(intersect_point),
                normal: Some(normal),
                kind: IntersectKind::Sphere,
            };
            Some(intersect)
        } else {
            None
        }
    }
}

/// 点光源.
#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct Light {
    /// 光源位置.
    pos: Vec3,
    /// 光照强度 (0.0 ~ 1.0).
    strength: f32,
}

#[wasm_bindgen]
impl Light {
    #[must_use]
    pub fn new(pos: Vec3, strength: f32) -> Self {
        Self { pos, strength }
    }
}

/// 渲染一个 3D 场景(光线追踪), 地面为 z = 0.
#[wasm_bindgen]
#[derive(Debug)]
pub struct RayTracing {
    width: usize,
    height: usize,
    last_frame_time: Option<Instant>,
    /// 摄像机世界坐标.
    camera_pos: Vec3,
    /// 摄像机视线, 始终是标准化的.
    camera_gaze: Vec3,
    /// 球体.
    spheres: Vec<Sphere>,
    /// 光源.
    lights: Vec<Light>,
    /// 按键管理器.
    am: ActionManager,
    rng: SmallRng,
    withdraw_actions_on_render: bool,
}

impl RayTracing {
    /// 相机移动速度 (米/s).
    const CAMERA_SPEED: f32 = 1.0;
    /// 相机转向速度 (rad/s).
    const CAMERA_ROTATION_SPEED: f32 = 30f32.to_radians();
    /// 天空颜色.
    const SKY_COLOR: Vec3 = Vec3::new_const(0.7, 0.6, 1.0);
    /// 地面颜色 1.
    const GROUND_COLOR_1: Vec3 = Vec3::new_const(0.9, 0.1, 0.1);
    /// 地面颜色 2.
    const GROUND_COLOR_2: Vec3 = Vec3::new_const(0.9, 0.9, 0.9);
    /// 地面颜色格子大小 (米).
    const GROUND_GRID_SIZE: f32 = 0.3;
    /// 反射的亮度损耗.
    const REFLECTION_DECAY: f32 = 0.4;
    /// 高光幂次.
    const SPECULAR_POW: f32 = 80.;
    /// 焦平面大小 (米).
    ///
    /// 焦平面的一半长度和 gaze 长度组成直角三角形两边,
    /// 这个直角三角形以 gaze 边为轴进行对称, 得到的二倍角就是视场角:
    /// `tan(FOV / 2) = FOCAL_SIZE / camera_gaze.magnitude()`
    const FOCAL_SIZE: f32 = 2.5;
    /// 抗锯齿采样次数.
    const AA_SAMPLES: u16 = 5;
    /// 最大的反射次数.
    const MAX_REFLECTION: u32 = 3;
}

#[wasm_bindgen]
impl RayTracing {
    /// 第一帧绘制 (render) 的时候会返回 `Some(...)`.
    pub fn new(width: usize, height: usize, seed: u32) -> Self {
        let mut self_ = Self {
            width,
            height,
            last_frame_time: None,
            camera_pos: Vec3::new(0., 0., 0.),
            camera_gaze: Vec3::new(1., 0., 0.).normalize(),
            spheres: Vec::new(),
            lights: Vec::new(),
            am: ActionManager::new(),
            rng: SmallRng::seed_from_u64(seed as u64),
            withdraw_actions_on_render: true,
        };
        self_.trigger_action(action::Action::RequestRender);
        self_
    }

    fn delta_time(&self) -> Option<Duration> {
        self.last_frame_time.map(|x| x.elapsed())
    }

    pub fn put_sphere(&mut self, sphere: Sphere) {
        self.spheres.push(sphere);
    }

    pub fn put_light(&mut self, light: Light) {
        self.lights.push(light);
    }

    pub fn move_camera_to(&mut self, pos: Vec3) {
        self.camera_pos = pos;
    }

    pub fn rotate_camera_to(&mut self, gaze: Vec3) {
        self.camera_gaze = gaze.normalize();
    }

    /// 在此处传入 [`RequestRender`](action::Action::RequestRender) 来让强制绘制一帧.
    ///
    /// 触发的 action 在 render 之后就会被清空, 如果不想清空, 那么 `set_withdraw_actions_on_render(false)`.
    pub fn trigger_action(&mut self, action: action::Action) {
        self.am.trigger(action);
    }

    pub fn withdraw_action(&mut self, action: action::Action) {
        self.am.withdraw(action);
    }

    pub fn set_withdraw_actions_on_render(&mut self, value: bool) {
        self.withdraw_actions_on_render = value;
    }

    fn handle_actions(&mut self) {
        debug_assert!(self.camera_gaze.is_normalized());
        let delta_time = self.delta_time().map_or(f32::EPSILON, |x| x.as_secs_f32());

        // 计算相机转向.
        // yaw 表示和正 x 轴在 xy 平面内的夹角, 就像普通的数学角度一样.
        let mut delta_angle = Vec3::ZERO; // x: yaw, y: pitch
        if self.am.is_triggerred(action::Action::CameraRotationDown) {
            delta_angle = delta_angle - Vec3::Y;
        }
        if self.am.is_triggerred(action::Action::CameraRotationUp) {
            delta_angle = delta_angle + Vec3::Y;
        }
        if self.am.is_triggerred(action::Action::CameraRotationCCW) {
            delta_angle = delta_angle - Vec3::X;
        }
        if self.am.is_triggerred(action::Action::CameraRotationCW) {
            delta_angle = delta_angle + Vec3::X; // 右转是增大 yaw (在右手坐标系中) ?
        }
        if !delta_angle.is_zero() {
            let horizontal_gaze = Vec3::new(self.camera_gaze.x, self.camera_gaze.y, 0.);
            let current_yaw = self.camera_gaze.y.atan2(self.camera_gaze.x);
            let current_pitch = self.camera_gaze.z.atan2(horizontal_gaze.magnitude());
            let Vec3 {
                x: delta_yaw,
                y: delta_pitch,
                z: _,
            } = delta_angle.normalize() * Self::CAMERA_ROTATION_SPEED * delta_time;

            // 两个弧度
            let yaw = (current_yaw + delta_yaw).rem(2.0 * f32::consts::PI);
            let pitch =
                (current_pitch + delta_pitch).clamp(-80f32.to_radians(), 80f32.to_radians()); // 限制角度防止万向轴问题.
            // 弧度转方向向量
            self.camera_gaze = Vec3::new(
                yaw.cos() * pitch.cos(),
                yaw.sin() * pitch.cos(),
                pitch.sin(),
            );
        }

        // 计算相机坐标偏移.
        let delta_distance = Self::CAMERA_SPEED * delta_time;
        let mut direction = Vec3::ZERO;
        // todo 判断这个叉积的方向是否正确.
        let right_direction = Vec3::Z.cross(self.camera_gaze);
        if self.am.is_triggerred(action::Action::CameraMoveForward) {
            direction = direction + self.camera_gaze;
        }
        if self.am.is_triggerred(action::Action::CameraMoveBackward) {
            direction = direction - self.camera_gaze;
        }
        if self.am.is_triggerred(action::Action::CameraMoveLeft) {
            direction = direction - right_direction;
        }
        if self.am.is_triggerred(action::Action::CameraMoveRight) {
            direction = direction + right_direction;
        }
        if self.am.is_triggerred(action::Action::CameraMoveUp) {
            direction = direction + Vec3::Z;
        }
        if self.am.is_triggerred(action::Action::CameraMoveDown) {
            direction = direction - Vec3::Z;
        }
        if !direction.is_zero() {
            self.camera_pos = self.camera_pos + direction.normalize() * delta_distance;
        }
    }

    /// 从一个点开始沿着指定方向进行相交检测, 返回相交结果.
    fn intersect(&self, origin: Vec3, direction: Vec3) -> Intersect {
        let mut min_distance_intersect = Intersect {
            distance: f32::INFINITY,
            hit_point: None,
            normal: None,
            kind: IntersectKind::Sky,
        };

        // 检测是否将会在某个远处相交于地面.
        let t = -origin.z / direction.z;
        if t > 0.0 {
            min_distance_intersect = Intersect {
                distance: t,
                normal: Some(Vec3::Z),
                hit_point: Some(origin + direction * t),
                kind: IntersectKind::Ground,
            };
        }

        // 和所有球体进行相交检测.
        // if let Some(mdi_candidate) = self
        //     .spheres
        //     .iter()
        //     .filter_map(|sphere| sphere.intersect(origin, direction))
        //     .min_by(|ia, ib| ia.distance.partial_cmp(&ib.distance).unwrap())
        //     && mdi_candidate.distance < min_distance_intersect.distance
        // {
        //     min_distance_intersect = mdi_candidate;
        // }
        // --- 上面是迭代器的写法, 下面是直接 for 的写法, 我发现下面更快一点. ---
        for sphere in &self.spheres {
            if let Some(intersect) = sphere.intersect(origin, direction)
                && matches!(
                    intersect
                        .distance
                        .partial_cmp(&min_distance_intersect.distance),
                    Some(std::cmp::Ordering::Less)
                )
            {
                min_distance_intersect = intersect;
            }
        }

        min_distance_intersect
    }

    /// 着色, 返回颜色 rgb (0.0 ~ 1.0).
    fn radiance(&self, origin: Vec3, direction: Vec3, reflection_count: u32) -> Vec3 {
        let intersect = self.intersect(origin, direction);
        if intersect.kind == IntersectKind::Sky {
            return Self::SKY_COLOR * (1.0 - direction.z.abs()).powf(4.0);
        }
        let intersect_point = intersect.hit_point.unwrap();
        let normal = intersect.normal.unwrap();

        // 计算各个点光源产生的兰伯特漫反射系数平均数.
        let lambert = self
            .lights
            .iter()
            .map(|l| {
                let to_light_direction = (l.pos - intersect_point).normalize();
                let lambert = to_light_direction.dot(normal);
                // 地上的点到点光源进行遮挡检测.
                let it = self.intersect(intersect_point, to_light_direction);
                if matches!(it.kind, IntersectKind::Sphere) {
                    // 被遮挡了.
                    0.
                } else {
                    lambert * l.strength
                }
            })
            .sum::<f32>()
            .div(self.lights.len() as f32)
            .max(0.);

        match intersect.kind {
            IntersectKind::Ground => {
                // 根据格子坐标选择颜色.
                let Vec3 { x, y, z: _ } = intersect_point;
                let ground_color = if ((x / Self::GROUND_GRID_SIZE).floor() as i32
                    + (y / Self::GROUND_GRID_SIZE).floor() as i32)
                    % 2
                    == 0
                {
                    Self::GROUND_COLOR_1
                } else {
                    Self::GROUND_COLOR_2
                };
                ground_color * (lambert + 0.1)
            }
            IntersectKind::Sphere => {
                let reflect_direction = normal * (2.0 * normal.dot(-direction)) + direction;
                // 计算高光(所有亮度产生的高光总和).
                let specular = if lambert > 0.0 {
                    self.lights
                        .iter()
                        .map(|l| {
                            let to_light_direction = (l.pos - intersect_point).normalize();
                            reflect_direction
                                .dot(to_light_direction)
                                .powf(Self::SPECULAR_POW)
                                * l.strength
                        })
                        .sum::<f32>()
                        .clamp(0.0, 1.0)
                } else {
                    0.
                };
                Vec3::new(specular, specular, specular)
                    + if reflection_count <= Self::MAX_REFLECTION {
                        // + normal * 0.01 防止又检测到此球体.
                        self.radiance(
                            intersect_point + normal * 0.01,
                            reflect_direction,
                            reflection_count + 1,
                        ) * (1.0 - Self::REFLECTION_DECAY)
                    } else {
                        Vec3::ZERO
                    }
            }
            IntersectKind::Sky => unreachable!(),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn render_pixel(
        &self,
        i: usize,
        right: Vec3,
        down: Vec3,
        top_left: Vec3,
        interval_x: f32,
        interval_y: f32,
        rng: &mut SmallRng,
    ) -> u32 {
        let x = i % self.width;
        let y = i / self.width;
        let rx = x as f32 / self.width as f32;
        let ry = y as f32 / self.height as f32;
        let mut pixel_color = Vec3::ZERO;
        for _ in 0..Self::AA_SAMPLES {
            // todo 不知道这个景深随机值取什么范围比较好.
            let dof_src =
                right * rng.random_range(0.0..0.001) + down * rng.random_range(0.0..0.001);
            let direction = top_left - dof_src
                + right * (rx * Self::FOCAL_SIZE + rng.random_range(-interval_x..interval_x))
                + down * (ry * Self::FOCAL_SIZE + rng.random_range(-interval_y..interval_y));
            let direction = direction.normalize();
            pixel_color = pixel_color + self.radiance(self.camera_pos + dof_src, direction, 0);
        }
        pixel_color = pixel_color / Self::AA_SAMPLES.into();
        rgbf(pixel_color.x, pixel_color.y, pixel_color.z)
    }

    /// 渲染画面, 当没有任何操作 ([`Action`](action::Action)) 的时候, 画面没变, 返回 None.
    pub fn render(&mut self) -> Option<Vec<u32>> {
        if !self.am.has_actions() {
            // 没操作, 那么场景没有变化, 不渲染.
            self.last_frame_time = Some(Instant::now()); // 假装渲染了一帧便于后面的时间计算.
            return None;
        }
        self.handle_actions();
        if self.withdraw_actions_on_render {
            self.am.clear();
        }
        self.last_frame_time = Some(Instant::now());
        // todo 判断这个叉积的方向是否正确.
        let right = Vec3::Z.cross(self.camera_gaze).normalize();
        let down = right.cross(self.camera_gaze).normalize();
        // 焦平面左上.
        let top_left =
            -right * Self::FOCAL_SIZE / 2.0 - down * Self::FOCAL_SIZE / 2.0 + self.camera_gaze;

        let interval_x = 0.5 / self.width as f32;
        let interval_y = 0.5 / self.height as f32;

        #[cfg(feature = "rayon")]
        {
            Some(
                (0..self.height * self.width)
                    .into_par_iter()
                    .chunks(1000)
                    .map_init(
                        || self.rng.clone(),
                        |rng, chunk| {
                            chunk
                                .into_iter()
                                .map(|i| {
                                    self.render_pixel(
                                        i, right, down, top_left, interval_x, interval_y, rng,
                                    )
                                })
                                .collect::<Vec<_>>()
                        },
                    )
                    .flatten()
                    .collect(),
            )
        }
        #[cfg(not(feature = "rayon"))]
        {
            let rng = &mut self.rng.clone();
            Some(
                (0..self.height * self.width)
                    .map(|i| {
                        self.render_pixel(i, right, down, top_left, interval_x, interval_y, rng)
                    })
                    .collect(),
            )
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn render_to_web_color(&mut self) -> Option<Vec<u8>> {
        // todo 找一个零拷贝传递给 js 的方法.
        Some(self.render()?.into_iter().flat_map(to_web_color).collect())
    }
}
