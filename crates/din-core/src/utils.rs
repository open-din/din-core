//! Shared internal utilities for runtime/domain helpers.

/// Returns a finite, positive value or a default fallback.
pub(crate) fn finite_positive_f32(value: f32, fallback: f32) -> f32 {
    if value.is_finite() && value > 0.0 {
        value
    } else {
        fallback
    }
}

/// Returns a finite, positive value or a default fallback.
pub(crate) fn finite_positive_f64(value: f64, fallback: f64) -> f64 {
    if value.is_finite() && value > 0.0 {
        value
    } else {
        fallback
    }
}

/// Returns a strictly positive `u32`, replacing zero with a fallback.
pub(crate) fn positive_u32(value: u32, fallback: u32) -> u32 {
    value.max(fallback.max(1))
}

/// Clamps a value to the inclusive unit range.
pub(crate) fn unit_interval(value: f32) -> f32 {
    value.clamp(0.0, 1.0)
}
