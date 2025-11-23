use std::ops::{Add, Div, Mul, Neg, Sub};

/// 右手坐标系, z 轴向上.
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
        Self::Output {
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
        Self::Output {
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

    #[inline]
    #[must_use]
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub const ZERO: Vec3 = Vec3::new(0., 0., 0.);
    pub const X: Vec3 = Vec3::new(1., 0., 0.);
    pub const Y: Vec3 = Vec3::new(0., 1., 0.);
    pub const Z: Vec3 = Vec3::new(0., 0., 1.);

    /// 是否是零向量.
    #[inline]
    #[must_use]
    pub const fn is_zero(self) -> bool {
        self.x.abs().max(self.y.abs()).max(self.z.abs()) < Self::TOLERANCE
    }

    /// 向量点乘
    #[inline]
    #[must_use]
    pub const fn dot(self, rhs: Self) -> f32 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }

    /// 向量叉乘
    #[inline]
    #[must_use]
    pub const fn cross(self, rhs: Self) -> Self {
        Self {
            x: self.y * rhs.z - self.z * rhs.y,
            y: self.z * rhs.x - self.x * rhs.z,
            z: self.x * rhs.y - self.y * rhs.x,
        }
    }

    /// 获取向量的模长
    #[inline]
    #[must_use]
    pub fn magnitude(self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    /// 是否是标准化的向量.
    #[inline]
    #[must_use]
    pub fn is_normalized(self) -> bool {
        self.magnitude().sub(1.0).abs() < Self::TOLERANCE
    }

    /// 标准化, 不做非 0 模长的保证.
    #[inline]
    #[must_use]
    pub fn normalize(self) -> Self {
        let mag = self.magnitude();
        Self {
            x: self.x / mag,
            y: self.y / mag,
            z: self.z / mag,
        }
    }

    /// 计算两个向量之间的余弦相似度.
    #[inline]
    #[must_use]
    pub fn cos(self, rhs: Self) -> f32 {
        let self_mag = self.magnitude();
        let rhs_mag = rhs.magnitude();
        self.dot(rhs) / self_mag / rhs_mag
    }
}
