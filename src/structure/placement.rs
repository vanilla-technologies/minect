use crate::_3d::{Coordinate3, Orientation3};

/// Return a [Vec] of [coordinates](Coordinate3) that completely fill the cubiod between `min` and
/// `max` as a space filling curve with the specified [orientation][Orientation3].
///
/// Furthermore the [Vec] has the following properties:
/// * The distance between two successive coordinates is always 1.
/// * It contains only distinct coordinates. No two coordinates are equal.
/// * The first and the last coordinates are corners of the cuboid (not neccessarily `min` or
///   `max` depending on `orientation`).
///
/// `min` has to be strictly smaller than `max`, but both can be positive or negative. If you have
/// two arbitrary [coordinates](Coordinate3) you can find `min` with [Coordinate3::min] and `max`
/// with [Coordinate3::max]
pub fn space_filling_curve(
    min: Coordinate3<i32>,
    max: Coordinate3<i32>,
    orientation: Orientation3,
) -> Vec<Coordinate3<i32>> {
    let delta = max - min;

    let axis3 = orientation.direction3().axis();
    let delta3 = delta[axis3];

    let axis2 = orientation.direction2().axis();
    let delta2 = delta[axis2];

    let axis1 = orientation.direction1().axis();
    let delta1 = delta[axis1];

    let backwards3 = orientation.direction3().negative();
    let mut backwards2 = orientation.direction2().negative();
    let mut backwards1 = orientation.direction1().negative();

    let mut result = Vec::new();
    for mut i3 in 0..=delta3 {
        if backwards3 {
            i3 = delta3 - i3;
        }
        if orientation.direction3().negative() {
            i3 *= -1;
        }
        for mut i2 in 0..=delta2 {
            if backwards2 {
                i2 = delta2 - i2;
            }
            if orientation.direction2().negative() {
                i2 *= -1;
            }
            for mut i1 in 0..=delta1 {
                if backwards1 {
                    i1 = delta1 - i1;
                }
                if orientation.direction1().negative() {
                    i1 *= -1;
                }
                let c = orientation.opposite().orient(Coordinate3(i1, i2, i3));
                let coordinate = c + min;
                result.push(coordinate);
            }
            backwards1 = !backwards1;
        }
        backwards2 = !backwards2;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(non_snake_case)]
    fn test_space_filling_curve_XYZ() {
        // given:
        let min = Coordinate3(2, 3, 5);
        let max = Coordinate3(3, 5, 6);

        // when:
        let actual = space_filling_curve(min, max, Orientation3::XYZ);

        // then:
        assert_eq!(
            actual,
            vec![
                Coordinate3(2, 3, 5),
                Coordinate3(3, 3, 5),
                Coordinate3(3, 4, 5),
                Coordinate3(2, 4, 5),
                Coordinate3(2, 5, 5),
                Coordinate3(3, 5, 5),
                Coordinate3(3, 5, 6),
                Coordinate3(2, 5, 6),
                Coordinate3(2, 4, 6),
                Coordinate3(3, 4, 6),
                Coordinate3(3, 3, 6),
                Coordinate3(2, 3, 6),
            ]
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_space_filling_curve_YXZ() {
        // given:
        let min = Coordinate3(2, 3, 5);
        let max = Coordinate3(3, 5, 6);

        // when:
        let actual = space_filling_curve(min, max, Orientation3::YXZ);

        // then:
        assert_eq!(
            actual,
            vec![
                Coordinate3(2, 3, 5),
                Coordinate3(2, 4, 5),
                Coordinate3(2, 5, 5),
                Coordinate3(3, 5, 5),
                Coordinate3(3, 4, 5),
                Coordinate3(3, 3, 5),
                Coordinate3(3, 3, 6),
                Coordinate3(3, 4, 6),
                Coordinate3(3, 5, 6),
                Coordinate3(2, 5, 6),
                Coordinate3(2, 4, 6),
                Coordinate3(2, 3, 6),
            ]
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_space_filling_curve_Zyx() {
        // given:
        let min = Coordinate3(2, 3, 5);
        let max = Coordinate3(3, 5, 6);

        // when:
        let actual = space_filling_curve(min, max, Orientation3::Zyx);

        // then:
        assert_eq!(
            actual,
            vec![
                Coordinate3(3, 5, 5),
                Coordinate3(3, 5, 6),
                Coordinate3(3, 4, 6),
                Coordinate3(3, 4, 5),
                Coordinate3(3, 3, 5),
                Coordinate3(3, 3, 6),
                Coordinate3(2, 3, 6),
                Coordinate3(2, 3, 5),
                Coordinate3(2, 4, 5),
                Coordinate3(2, 4, 6),
                Coordinate3(2, 5, 6),
                Coordinate3(2, 5, 5),
            ]
        );
    }
}
