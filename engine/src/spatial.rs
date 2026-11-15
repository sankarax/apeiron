pub fn wrapped_delta(from: f32, to: f32, length: f32) -> f32 {
    let delta = to - from;

    (delta + length / 2.0).rem_euclid(length) - length / 2.0
}

pub fn distance_squared(x1: f32, y1: f32, x2: f32, y2: f32, width: f32, height: f32) -> f32 {
    let dx = wrapped_delta(x1, x2, width);
    let dy = wrapped_delta(y1, y2, height);

    dx * dx + dy * dy
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrapped_distance_across_boundary() {
        let delta = wrapped_delta(1595.0, 5.0, 1600.0);

        assert!((delta - 10.0).abs() < 0.001);
    }

    #[test]
    fn wrapped_distance_reverse_direction() {
        let delta = wrapped_delta(5.0, 1595.0, 1600.0);

        assert!((delta + 10.0).abs() < 0.001);
    }

    #[test]
    fn normal_distance_squared() {
        let distance = distance_squared(10.0, 10.0, 13.0, 14.0, 1600.0, 900.0);

        assert!((distance - 25.0).abs() < 0.001);
    }
}
