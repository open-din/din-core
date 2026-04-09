//! Scalar math/compare utilities backing native `math`, `compare`, `mix`, `clamp`, and `switch` nodes.

use serde::{Deserialize, Serialize};

/// Scalar operations supported by the runtime `math` node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MathOperation {
    /// Returns `a + b`.
    Add,
    /// Returns `a - b`.
    Subtract,
    /// Returns `a * b`.
    Multiply,
    /// Returns `a / b`, or `0.0` when `b == 0.0`.
    Divide,
    /// Returns fused multiply-add: `a * b + c`.
    MultiplyAdd,
    /// Returns `a.powf(b)`.
    Power,
    /// Returns logarithm of `a` in base `b`.
    Logarithm,
    /// Returns square root of `a` after clamping to non-negative.
    Sqrt,
    /// Returns inverse square root of `a`.
    InvSqrt,
    /// Returns absolute value of `a`.
    Abs,
    /// Returns `exp(a)`.
    Exp,
    /// Returns `min(a, b)`.
    Min,
    /// Returns `max(a, b)`.
    Max,
    /// Returns `1.0` when `a < b`, otherwise `0.0`.
    LessThan,
    /// Returns `1.0` when `a > b`, otherwise `0.0`.
    GreaterThan,
    /// Returns sign of `a`.
    Sign,
    /// Returns sign of `a - b`.
    Compare,
    /// Returns smooth minimum of `a` and `b` controlled by `c`.
    SmoothMin,
    /// Returns smooth maximum of `a` and `b` controlled by `c`.
    SmoothMax,
    /// Rounds to nearest integer.
    Round,
    /// Rounds down.
    Floor,
    /// Rounds up.
    Ceil,
    /// Truncates fractional component.
    Truncate,
    /// Returns fractional component.
    Fraction,
    /// Returns truncating modulo (`a % b`).
    TruncModulo,
    /// Returns positive modulo in `[0, b)`.
    FloorModulo,
    /// Wraps `a` in range `[b, c)`.
    Wrap,
    /// Quantizes `a` to a `b` step size.
    Snap,
    /// Triangle-wave fold in `[0, b]`.
    PingPong,
    /// Returns `sin(a)`.
    Sin,
    /// Returns `cos(a)`.
    Cos,
    /// Returns `tan(a)`.
    Tan,
    /// Returns `asin(a)` with domain clamp.
    Asin,
    /// Returns `acos(a)` with domain clamp.
    Acos,
    /// Returns `atan(a)`.
    Atan,
    /// Returns `atan2(a, b)`.
    Atan2,
    /// Returns `sinh(a)`.
    Sinh,
    /// Returns `cosh(a)`.
    Cosh,
    /// Returns `tanh(a)`.
    Tanh,
}

/// Comparison operations for the runtime `compare` node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompareOperation {
    /// Returns `a == b`.
    Equal,
    /// Returns `a != b`.
    NotEqual,
    /// Returns `a < b`.
    LessThan,
    /// Returns `a <= b`.
    LessThanOrEqual,
    /// Returns `a > b`.
    GreaterThan,
    /// Returns `a >= b`.
    GreaterThanOrEqual,
}

/// Boundary behavior used by the runtime `clamp` node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClampMode {
    /// Hard clamp into `[min, max]`.
    Clamp,
    /// Periodic wrap into `[min, max)`.
    Wrap,
    /// Reflect/fold at the edges.
    Fold,
}

/// Executes a scalar operation against up to three operands.
///
/// `c` is used by a subset of operations (for example `MultiplyAdd`, smoothing, and wrapping).
///
/// # Examples
///
/// ```
/// use din_core::{MathOperation, math};
///
/// assert_eq!(math(MathOperation::Add, 2.0, 3.0, 0.0), 5.0);
/// assert_eq!(math(MathOperation::MultiplyAdd, 2.0, 3.0, 4.0), 10.0);
/// ```
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

/// Executes a boolean comparison operation.
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

/// Linear interpolation from `a` to `b` using factor `t`.
///
/// When `clamp_t` is `true`, `t` is constrained to `[0.0, 1.0]`.
///
/// # Examples
///
/// ```
/// use din_core::mix;
///
/// assert_eq!(mix(0.0, 10.0, 0.25, true), 2.5);
/// assert_eq!(mix(0.0, 10.0, 2.0, true), 10.0);
/// ```
pub fn mix(a: f32, b: f32, t: f32, clamp_t: bool) -> f32 {
    let t = if clamp_t { t.clamp(0.0, 1.0) } else { t };
    a + (b - a) * t
}

/// Applies clamp/wrap/fold behavior for a value range.
///
/// Returns the input unchanged when `min > max`.
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

/// Selects one value by index from an input slice.
///
/// Returns `0.0` when the index is out of range.
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
