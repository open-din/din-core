//! Scalar math/compare utilities backing native `math`, `compare`, `mix`, `clamp`, and `switch` nodes.
#![allow(missing_docs)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MathOperation {
    Add,
    Subtract,
    Multiply,
    Divide,
    MultiplyAdd,
    Power,
    Logarithm,
    Sqrt,
    InvSqrt,
    Abs,
    Exp,
    Min,
    Max,
    LessThan,
    GreaterThan,
    Sign,
    Compare,
    SmoothMin,
    SmoothMax,
    Round,
    Floor,
    Ceil,
    Truncate,
    Fraction,
    TruncModulo,
    FloorModulo,
    Wrap,
    Snap,
    PingPong,
    Sin,
    Cos,
    Tan,
    Asin,
    Acos,
    Atan,
    Atan2,
    Sinh,
    Cosh,
    Tanh,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompareOperation {
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClampMode {
    Clamp,
    Wrap,
    Fold,
}

pub fn math(operation: MathOperation, a: f32, b: f32, c: f32) -> f32 {
    match operation {
        MathOperation::Add => a + b,
        MathOperation::Subtract => a - b,
        MathOperation::Multiply => a * b,
        MathOperation::Divide => {
            if b == 0.0 {
                0.0
            } else {
                a / b
            }
        }
        MathOperation::MultiplyAdd => a.mul_add(b, c),
        MathOperation::Power => a.powf(b),
        MathOperation::Logarithm => {
            if a <= 0.0 || b <= 0.0 || b == 1.0 {
                0.0
            } else {
                a.log(b)
            }
        }
        MathOperation::Sqrt => a.max(0.0).sqrt(),
        MathOperation::InvSqrt => {
            let sqrt = a.max(0.0).sqrt();
            if sqrt == 0.0 { 0.0 } else { 1.0 / sqrt }
        }
        MathOperation::Abs => a.abs(),
        MathOperation::Exp => a.exp(),
        MathOperation::Min => a.min(b),
        MathOperation::Max => a.max(b),
        MathOperation::LessThan => (a < b) as u8 as f32,
        MathOperation::GreaterThan => (a > b) as u8 as f32,
        MathOperation::Sign => a.signum(),
        MathOperation::Compare => (a - b).signum(),
        MathOperation::SmoothMin => smooth_min(a, b, c.max(0.000_1)),
        MathOperation::SmoothMax => smooth_max(a, b, c.max(0.000_1)),
        MathOperation::Round => a.round(),
        MathOperation::Floor => a.floor(),
        MathOperation::Ceil => a.ceil(),
        MathOperation::Truncate => a.trunc(),
        MathOperation::Fraction => a.fract(),
        MathOperation::TruncModulo => {
            if b == 0.0 {
                0.0
            } else {
                a % b
            }
        }
        MathOperation::FloorModulo => {
            if b == 0.0 {
                0.0
            } else {
                ((a % b) + b) % b
            }
        }
        MathOperation::Wrap => wrap(a, b, c),
        MathOperation::Snap => {
            if b == 0.0 {
                a
            } else {
                (a / b).round() * b
            }
        }
        MathOperation::PingPong => ping_pong(a, b),
        MathOperation::Sin => a.sin(),
        MathOperation::Cos => a.cos(),
        MathOperation::Tan => a.tan(),
        MathOperation::Asin => a.clamp(-1.0, 1.0).asin(),
        MathOperation::Acos => a.clamp(-1.0, 1.0).acos(),
        MathOperation::Atan => a.atan(),
        MathOperation::Atan2 => a.atan2(b),
        MathOperation::Sinh => a.sinh(),
        MathOperation::Cosh => a.cosh(),
        MathOperation::Tanh => a.tanh(),
    }
}

pub fn compare(operation: CompareOperation, a: f32, b: f32) -> bool {
    match operation {
        CompareOperation::Equal => a == b,
        CompareOperation::NotEqual => a != b,
        CompareOperation::LessThan => a < b,
        CompareOperation::LessThanOrEqual => a <= b,
        CompareOperation::GreaterThan => a > b,
        CompareOperation::GreaterThanOrEqual => a >= b,
    }
}

pub fn mix(a: f32, b: f32, t: f32, clamp_t: bool) -> f32 {
    let t = if clamp_t { t.clamp(0.0, 1.0) } else { t };
    a + (b - a) * t
}

pub fn clamp(value: f32, min: f32, max: f32, mode: ClampMode) -> f32 {
    if min > max {
        return value;
    }

    match mode {
        ClampMode::Clamp => value.clamp(min, max),
        ClampMode::Wrap => wrap(value, min, max),
        ClampMode::Fold => {
            let range = max - min;
            if range == 0.0 {
                return min;
            }
            let mut wrapped = (value - min) % (range * 2.0);
            if wrapped < 0.0 {
                wrapped += range * 2.0;
            }
            if wrapped > range {
                max - (wrapped - range)
            } else {
                min + wrapped
            }
        }
    }
}

pub fn switch_value(index: usize, inputs: &[f32]) -> f32 {
    inputs.get(index).copied().unwrap_or(0.0)
}

fn wrap(value: f32, min: f32, max: f32) -> f32 {
    if min >= max {
        return min;
    }
    let range = max - min;
    let mut wrapped = (value - min) % range;
    if wrapped < 0.0 {
        wrapped += range;
    }
    min + wrapped
}

fn ping_pong(value: f32, length: f32) -> f32 {
    if length <= 0.0 {
        return 0.0;
    }
    let wrapped = wrap(value, 0.0, length * 2.0);
    if wrapped > length {
        length - (wrapped - length)
    } else {
        wrapped
    }
}

fn smooth_min(a: f32, b: f32, k: f32) -> f32 {
    let h = ((k - (a - b).abs()).max(0.0)) / k;
    a.min(b) - h * h * h * k * (1.0 / 6.0)
}

fn smooth_max(a: f32, b: f32, k: f32) -> f32 {
    -smooth_min(-a, -b, k)
}
