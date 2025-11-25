use std::ops::{Add, Div, Mul, Neg, Sub};

use wasm_bindgen::prelude::wasm_bindgen;
#[cfg(feature = "simd")]
use wide::{f32x4, f32x8};

/// 右手坐标系, z 轴向上.
#[wasm_bindgen]
#[derive(Debug, Clone, Copy)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// 向量乘标量
impl Mul<f32> for Vec3 {
    type Output = Vec3;

    fn mul(self, rhs: f32) -> Self::Output {
        #[cfg(feature = "simd")]
        {
            #[allow(clippy::suspicious_arithmetic_impl)]
            let [x, y, z, _] =
                (f32x4::new([self.x, self.y, self.z, 0.]) * f32x4::splat(rhs)).to_array();
            Self { x, y, z }
        }
        #[cfg(not(feature = "simd"))]
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

/// 向量除以标量
impl Div<f32> for Vec3 {
    type Output = Vec3;

    fn div(self, rhs: f32) -> Self::Output {
        #[cfg(feature = "simd")]
        {
            #[allow(clippy::suspicious_arithmetic_impl)]
            let [x, y, z, _] =
                (f32x4::new([self.x, self.y, self.z, 0.]) * f32x4::splat(rhs).recip()).to_array();
            Self { x, y, z }
        }
        #[cfg(not(feature = "simd"))]
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
        }
    }
}

/// 反向
impl Neg for Vec3 {
    type Output = Vec3;

    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

/// 向量相加
impl Add for Vec3 {
    type Output = Vec3;

    fn add(self, rhs: Self) -> Self::Output {
        #[cfg(feature = "simd")]
        {
            let self_simd = f32x4::new([self.x, self.y, self.z, 0.]);
            let rhs_simd = f32x4::new([rhs.x, rhs.y, rhs.z, 0.]);
            let rst = self_simd + rhs_simd;
            let [x, y, z, _] = rst.to_array();
            Self::Output { x, y, z }
        }
        #[cfg(not(feature = "simd"))]
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

/// 向量相减
impl Sub for Vec3 {
    type Output = Vec3;

    fn sub(self, rhs: Self) -> Self::Output {
        #[cfg(feature = "simd")]
        {
            let self_simd = f32x4::new([self.x, self.y, self.z, 0.]);
            let rhs_simd = f32x4::new([rhs.x, rhs.y, rhs.z, 0.]);
            let rst = self_simd - rhs_simd;
            let [x, y, z, _] = rst.to_array();
            Self::Output { x, y, z }
        }
        #[cfg(not(feature = "simd"))]
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl From<(f32, f32, f32)> for Vec3 {
    fn from(value: (f32, f32, f32)) -> Self {
        Vec3 {
            x: value.0,
            y: value.1,
            z: value.2,
        }
    }
}

impl Vec3 {
    const TOLERANCE: f32 = 1e-6;
    pub const ZERO: Vec3 = Vec3::new_const(0., 0., 0.);
    pub const X: Vec3 = Vec3::new_const(1., 0., 0.);
    pub const Y: Vec3 = Vec3::new_const(0., 1., 0.);
    pub const Z: Vec3 = Vec3::new_const(0., 0., 1.);

    #[inline]
    #[must_use]
    pub const fn new_const(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

#[wasm_bindgen]
impl Vec3 {
    #[must_use]
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// 是否是零向量.
    #[must_use]
    pub fn is_zero(self) -> bool {
        self.x.abs().max(self.y.abs()).max(self.z.abs()) < Self::TOLERANCE
    }

    /// 向量点乘
    #[must_use]
    pub fn dot(self, rhs: Self) -> f32 {
        // 这里的 wide simd 好像没有明显的优化.
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }

    /// 向量叉乘
    #[must_use]
    pub fn cross(self, rhs: Self) -> Self {
        #[cfg(feature = "simd")]
        {
            let self_simd = f32x8::new([self.y, self.z, self.x, self.z, self.x, self.y, 0., 0.]);
            let rhs_simd = f32x8::new([rhs.z, rhs.x, rhs.y, rhs.y, rhs.z, rhs.x, 0., 0.]);
            let rst = self_simd * rhs_simd;
            let [x1, y1, z1, x2, y2, z2, _, _] = rst.to_array();
            let rst = f32x4::new([x1, y1, z1, 0.]) - f32x4::new([x2, y2, z2, 0.]);
            let [x, y, z, _] = rst.to_array();
            Self { x, y, z }
        }
        #[cfg(not(feature = "simd"))]
        Self {
            x: self.y * rhs.z - self.z * rhs.y,
            y: self.z * rhs.x - self.x * rhs.z,
            z: self.x * rhs.y - self.y * rhs.x,
        }
    }

    /// 获取向量的模长
    #[must_use]
    pub fn magnitude(self) -> f32 {
        // 这里的 wide simd 好像没有明显的优化.
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    /// 是否是标准化的向量.
    #[must_use]
    pub fn is_normalized(self) -> bool {
        (self.x * self.x + self.y * self.y + self.z * self.z - 1.).abs() < Self::TOLERANCE
    }

    /// 标准化, 不做非 0 模长的保证.
    #[must_use]
    pub fn normalize(self) -> Self {
        #[cfg(feature = "simd")]
        {
            let self_simd1 = f32x4::new([self.x, self.y, self.z, 0.]);
            let self_simd2 = f32x4::new([self.y, self.z, self.x, 0.]);
            let self_simd3 = f32x4::new([self.z, self.x, self.y, 0.]);
            let [x, y, z, _] = (self_simd1
                * (self_simd1 * self_simd1 + self_simd2 * self_simd2 + self_simd3 * self_simd3)
                    .recip_sqrt())
            .to_array();
            Self { x, y, z }
        }
        #[cfg(not(feature = "simd"))]
        {
            let mag = self.magnitude();
            Self {
                x: self.x / mag,
                y: self.y / mag,
                z: self.z / mag,
            }
        }
    }

    /// 计算两个向量之间的余弦相似度.
    #[must_use]
    pub fn cos(self, rhs: Self) -> f32 {
        let self_mag = self.magnitude();
        let rhs_mag = rhs.magnitude();
        self.dot(rhs) / self_mag / rhs_mag
    }
}
