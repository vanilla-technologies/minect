use crate::geometry3::{Coordinate3, Orientation3};

/// Return a [Vec] of [coordinates](Coordinate3) that completely fill the cubiod spanned by `min`
/// and `max` as a space filling curve with the specified [orientation][Orientation3].
///
/// Furthermore the [Vec] has the following properties:
/// * The distance between two successive coordinates is always 1.
/// * It contains only distinct coordinates. No two coordinates are equal.
/// * The first and the last coordinates are corners of the cuboid (not neccessarily `min` or
///   `max` depending on `orientation`).
///
/// `min` has to be smaller or equal to `max` on all axes, but both can be positive or negative. If
/// you have two arbitrary [coordinates](Coordinate3) you can find `min` with [Coordinate3::min] and
/// `max` with [Coordinate3::max]
pub fn space_filling_curve(
    min: Coordinate3<i32>,
    max: Coordinate3<i32>,
    orientation: Orientation3,
) -> Vec<Coordinate3<i32>> {
    // The offset needed to move negative values to be positive so the result is between min and max
    let mut in_place_offset = max - min;
    if orientation.direction1().is_positive() {
        in_place_offset[orientation.direction1().axis()] = 0;
    }
    if orientation.direction2().is_positive() {
        in_place_offset[orientation.direction2().axis()] = 0;
    }
    if orientation.direction3().is_positive() {
        in_place_offset[orientation.direction3().axis()] = 0;
    }
    let offset = in_place_offset + min;

    let size = max - min + Coordinate3(1, 1, 1); // +1 because min and max are inclusive
    let size = orientation.inverse().orient(size);
    let size = size.map(i32::abs); // Size is always positive
    snake_curve(size)
        .into_iter()
        .map(|c| orientation.orient(c) + offset)
        .collect()
}

fn snake_curve(size: Coordinate3<i32>) -> Vec<Coordinate3<i32>> {
    let mut result = Vec::new();
    let mut y_backwards = false;
    let mut x_backwards = false;
    for z in 0..size.2 {
        for mut y in 0..size.1 {
            if y_backwards {
                y = size.1 - 1 - y;
            }
            for mut x in 0..size.0 {
                if x_backwards {
                    x = size.0 - 1 - x;
                }
                result.push(Coordinate3(x, y, z));
            }
            x_backwards = !x_backwards;
        }
        y_backwards = !y_backwards;
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

    #[test]
    #[allow(non_snake_case)]
    fn test_space_filling_curve_Zxy() {
        // given:
        let min = Coordinate3(2, 3, 5);
        let max = Coordinate3(3, 5, 6);

        // when:
        let actual = space_filling_curve(min, max, Orientation3::Zxy);

        // then:
        assert_eq!(
            actual,
            vec![
                Coordinate3(3, 5, 5),
                Coordinate3(3, 5, 6),
                Coordinate3(2, 5, 6),
                Coordinate3(2, 5, 5),
                Coordinate3(2, 4, 5),
                Coordinate3(2, 4, 6),
                Coordinate3(3, 4, 6),
                Coordinate3(3, 4, 5),
                Coordinate3(3, 3, 5),
                Coordinate3(3, 3, 6),
                Coordinate3(2, 3, 6),
                Coordinate3(2, 3, 5),
            ]
        );
    }
}
