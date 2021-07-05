use crate::geometry3::{Coordinate3, Direction3};
use std::{cmp::max, convert::TryFrom};

pub trait Command {
    fn is_conditional(&self) -> bool;
}

pub struct CommandBlock<C: Command> {
    pub command: Option<C>,
    pub coordinate: Coordinate3<i32>,
    pub direction: Direction3,
}

pub fn place_commands<C: Command>(chain: Vec<C>) -> Vec<CommandBlock<C>> {
    let max_cond_len = max_cond_len(&chain);
    // The minimum size for the primary direction needed to place all conditional commands
    let min_prim_size = max_cond_len + 2;
    let estimated_side_length = (chain.len() as f32).cbrt().ceil() as usize;
    let prim_size = max(min_prim_size, estimated_side_length);

    let mut curve = QuaterSpiralCurve::new()
        .flat_map(|(x, z)| {
            let forwards = (x % 2 == 0) == (z % 2 == 0); // Alternate between forwards and backwards
            (0..prim_size)
                .map(move |y| if forwards { y } else { prim_size - 1 - y })
                .map(move |y| Coordinate3(x, y as i32, z))
        })
        .enumerate()
        .peekable();

    let mut next_coordinate = || {
        let (index, current) = curve.next().unwrap(); // curve is infinite, so unwrap is safe
        let (_, next) = curve.peek().unwrap(); // curve is infinite, so unwrap is safe
        let direction = Direction3::try_from(*next - current).unwrap(); // next is guaranteed to be right next to current
        (index, current, direction)
    };

    let mut chain = chain
        .into_iter()
        .rev() // Go backwards through the chain to calculate all conditional sequence lengths
        .scan(0, |cond_seq_len, command| {
            if command.is_conditional() {
                *cond_seq_len += 1;
            } else {
                let len = *cond_seq_len;
                if len != 0 {
                    *cond_seq_len = 0; // Reset

                    // The unconditional command before the first conditional is part of the sequence
                    return Some((command, len + 1));
                }
            }
            Some((command, *cond_seq_len))
        })
        .collect::<Vec<_>>(); // Collect to reverse again
    chain.reverse();

    let mut result = Vec::new();
    for (command, cond_seq_len) in chain {
        let (index, mut coordinate, mut direction) = next_coordinate();
        if cond_seq_len != 0 {
            let index = index % prim_size;
            let space_until_corner = prim_size - 1 - index;
            if cond_seq_len > space_until_corner {
                // Insert no operation commands
                let no_op_count = space_until_corner + 1;
                let mut i = 0;
                while i < no_op_count {
                    i += 1;
                    result.push(CommandBlock {
                        command: None,
                        coordinate,
                        direction,
                    });
                    // Update coordinate
                    // Use destructuring assignment (https://github.com/rust-lang/rust/issues/71126)
                    let (_, new_coordinate, new_direction) = next_coordinate();
                    coordinate = new_coordinate;
                    direction = new_direction;
                }
            }
        }
        result.push(CommandBlock {
            command: Some(command),
            coordinate,
            direction,
        });
    }
    result
}

/// Finds the length of the longest sequence of conditional commands
fn max_cond_len<'l, I, C: 'l>(chain: I) -> usize
where
    I: IntoIterator<Item = &'l C>,
    C: Command,
{
    let mut max_cond_len = 0;
    let mut cond_len = 0;
    for command in chain {
        if command.is_conditional() {
            cond_len += 1;
        } else {
            max_cond_len = max(max_cond_len, cond_len);
        }
    }
    max(max_cond_len, cond_len)
}

/// An infinite [Iterator] producing a space filling curve starting from the origin by alternating a
/// quater spiral back and forth. The following table illustrates the iteration scheme:
/// |  y\x  |  0 |  1 |  2 |  3 |
/// |-------|----|----|----|----|
/// | **0** |  1 |  2 |  9 | 10 |
/// | **1** |  4 |  3 |  8 | 11 |
/// | **2** |  5 |  6 |  7 | 12 |
/// | **3** | 16 | 15 | 14 | 13 |
pub struct QuaterSpiralCurve {
    next: (i32, i32),
}

impl QuaterSpiralCurve {
    pub fn new() -> QuaterSpiralCurve {
        QuaterSpiralCurve { next: (0, 0) }
    }
}

impl Iterator for QuaterSpiralCurve {
    type Item = (i32, i32);

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.next;
        if current.0 > current.1 || (current.0 == current.1 && current.0 % 2 == 0) {
            if current.0 % 2 == 1 {
                self.next.1 += 1;
            } else if current.1 == 0 {
                self.next.0 += 1;
            } else {
                self.next.1 -= 1;
            }
        } else {
            if current.1 % 2 == 0 {
                self.next.0 += 1;
            } else if current.0 == 0 {
                self.next.1 += 1;
            } else {
                self.next.0 -= 1;
            }
        }
        Some(current)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quarter_spiral_curve() {
        // given:
        let mut under_test = QuaterSpiralCurve::new();

        // when / then:
        assert_eq!(under_test.next(), Some((0, 0)));
        assert_eq!(under_test.next(), Some((1, 0)));
        assert_eq!(under_test.next(), Some((1, 1)));
        assert_eq!(under_test.next(), Some((0, 1)));
        assert_eq!(under_test.next(), Some((0, 2)));
        assert_eq!(under_test.next(), Some((1, 2)));
        assert_eq!(under_test.next(), Some((2, 2)));
        assert_eq!(under_test.next(), Some((2, 1)));
        assert_eq!(under_test.next(), Some((2, 0)));
        assert_eq!(under_test.next(), Some((3, 0)));
        assert_eq!(under_test.next(), Some((3, 1)));
        assert_eq!(under_test.next(), Some((3, 2)));
        assert_eq!(under_test.next(), Some((3, 3)));
        assert_eq!(under_test.next(), Some((2, 3)));
        assert_eq!(under_test.next(), Some((1, 3)));
        assert_eq!(under_test.next(), Some((0, 3)));
    }
}
