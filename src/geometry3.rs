use std::{
    cmp::{max, min},
    ops::{Add, Index, IndexMut, Neg, Not, Rem, Sub},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Coordinate3<T>(pub T, pub T, pub T);

impl<T: Ord> Coordinate3<T> {
    /// Finds the [Coordinate3] of the minimum corner in the cuboid spanned by `a` and `b`.
    pub fn min(a: Coordinate3<T>, b: Coordinate3<T>) -> Coordinate3<T> {
        Coordinate3(min(a.0, b.0), min(a.1, b.1), min(a.2, b.2))
    }

    /// Finds the [Coordinate3] of the maximum corner in the cuboid spanned by `a` and `b`.
    pub fn max(a: Coordinate3<T>, b: Coordinate3<T>) -> Coordinate3<T> {
        Coordinate3(max(a.0, b.0), max(a.1, b.1), max(a.2, b.2))
    }
}

impl<T> Index<Axis3> for Coordinate3<T> {
    type Output = T;

    fn index(&self, index: Axis3) -> &Self::Output {
        match index {
            Axis3::X => &self.0,
            Axis3::Y => &self.1,
            Axis3::Z => &self.2,
        }
    }
}

impl<T> IndexMut<Axis3> for Coordinate3<T> {
    fn index_mut(&mut self, index: Axis3) -> &mut Self::Output {
        match index {
            Axis3::X => &mut self.0,
            Axis3::Y => &mut self.1,
            Axis3::Z => &mut self.2,
        }
    }
}

impl<T: Add> Add for Coordinate3<T> {
    type Output = Coordinate3<T::Output>;

    fn add(self, rhs: Self) -> Self::Output {
        Coordinate3(self.0.add(rhs.0), self.1.add(rhs.1), self.2.add(rhs.2))
    }
}

impl<T: Sub> Sub for Coordinate3<T> {
    type Output = Coordinate3<T::Output>;

    fn sub(self, rhs: Self) -> Self::Output {
        Coordinate3(self.0.sub(rhs.0), self.1.sub(rhs.1), self.2.sub(rhs.2))
    }
}

impl<T: Rem> Rem for Coordinate3<T> {
    type Output = Coordinate3<T::Output>;

    fn rem(self, rhs: Self) -> Self::Output {
        Coordinate3(self.0.rem(rhs.0), self.1.rem(rhs.1), self.2.rem(rhs.2))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Axis3 {
    X,
    Y,
    Z,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Direction3 {
    East,
    West,
    Up,
    Down,
    South,
    North,
}

impl Direction3 {
    pub fn axis(&self) -> Axis3 {
        match self {
            Direction3::East => Axis3::X,
            Direction3::West => Axis3::X,
            Direction3::Up => Axis3::Y,
            Direction3::Down => Axis3::Y,
            Direction3::South => Axis3::Z,
            Direction3::North => Axis3::Z,
        }
    }

    pub fn positive(&self) -> bool {
        match self {
            Direction3::East => true,
            Direction3::West => false,
            Direction3::Up => true,
            Direction3::Down => false,
            Direction3::South => true,
            Direction3::North => false,
        }
    }

    pub fn negative(&self) -> bool {
        !self.positive()
    }

    pub fn sign(&self) -> i8 {
        if self.positive() {
            1
        } else {
            -1
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(test, derive(strum::EnumIter))]
/// An [Orientation3] represents one of the 48 ways you can allign a structure to a grid in three
/// dimensions via rotation and mirroring. It consists of a
/// [primary](Orientation3::direction1()), a [secondary](Orientation3::direction2())
/// and a [tertiary](Orientation3::direction3()) [direction](Direction3) which are all
/// on different [axes](Axis3).
///
/// An [Orientation3] can also be represented by a left multiplied rotation matrix where the first
/// row represents the primary direction, the second row the secondary and so on. A value of 1
/// indicates a positive direction and a value of -1 indicates a negative direction. Each column
/// contains exactly one non zero value, as all directions are on different axes.
///
/// For example [Orientation3::XYZ] is represented by the identity matrix:
///
/// |       | 1 | 2 | 3 |
/// |-------|---|---|---|
/// | **x** | 1 | 0 | 0 |
/// | **y** | 0 | 1 | 0 |
/// | **z** | 0 | 0 | 1 |
///
/// And [Orientation3::Zyx] is represented by this matrix:
///
/// |       | 1 |  2 |  3 |
/// |-------|---|----|----|
/// | **x** | 0 |  0 | -1 |
/// | **y** | 0 | -1 |  0 |
/// | **z** | 1 |  0 |  0 |
#[allow(non_camel_case_types)]
pub enum Orientation3 {
    XYZ,
    XYz,
    XZY,
    XZy,
    XyZ,
    Xyz,
    XzY,
    Xzy,
    YXZ,
    YXz,
    YZX,
    YZx,
    YxZ,
    Yxz,
    YzX,
    Yzx,
    ZXY,
    ZXy,
    ZYX,
    ZYx,
    ZxY,
    Zxy,
    ZyX,
    Zyx,
    xYZ,
    xYz,
    xZY,
    xZy,
    xyZ,
    xyz,
    xzY,
    xzy,
    yXZ,
    yXz,
    yZX,
    yZx,
    yxZ,
    yxz,
    yzX,
    yzx,
    zXY,
    zXy,
    zYX,
    zYx,
    zxY,
    zxy,
    zyX,
    zyx,
}

impl Orientation3 {
    /// Orient a [Coordinate3] according to this [Orientation3].
    ///
    /// For example orienting the coordinate (1, 2, 3) to [Orientation3::YZX] results in (3, 1, 2),
    /// which is the equivalent of rotating it around an axis going diagonally through the origin.
    ///
    /// Some orientations are their own reverse, bot others are not. You can undo by orienting to
    /// the [opposite](Orientation3::opposite) orientation.
    // TODO consume to avoid clone
    pub fn orient<T: Clone + Neg<Output = T>>(&self, coordinate: Coordinate3<T>) -> Coordinate3<T> {
        let mut result = coordinate.clone();
        let Coordinate3(mut t1, mut t2, mut t3) = coordinate;
        if self.direction1().negative() {
            t1 = -t1;
        }
        if self.direction2().negative() {
            t2 = -t2;
        }
        if self.direction3().negative() {
            t3 = -t3;
        }
        result[self.direction1().axis()] = t1;
        result[self.direction2().axis()] = t2;
        result[self.direction3().axis()] = t3;
        result
    }

    /// Find the inverse [Orientation3]. This is equivalent to inverting the rotation matrix, which
    /// is the same as transposing it.
    pub fn inverse(&self) -> Orientation3 {
        match self {
            Orientation3::XYZ => Orientation3::XYZ,
            Orientation3::XYz => Orientation3::XYz,
            Orientation3::XZY => Orientation3::XZY,
            Orientation3::XZy => Orientation3::XzY,
            Orientation3::XyZ => Orientation3::XyZ,
            Orientation3::Xyz => Orientation3::Xyz,
            Orientation3::XzY => Orientation3::XZy,
            Orientation3::Xzy => Orientation3::Xzy,
            Orientation3::YXZ => Orientation3::YXZ,
            Orientation3::YXz => Orientation3::YXz,
            Orientation3::YZX => Orientation3::ZXY,
            Orientation3::YZx => Orientation3::zXY,
            Orientation3::YxZ => Orientation3::yXZ,
            Orientation3::Yxz => Orientation3::yXz,
            Orientation3::YzX => Orientation3::ZXy,
            Orientation3::Yzx => Orientation3::zXy,
            Orientation3::ZXY => Orientation3::YZX,
            Orientation3::ZXy => Orientation3::YzX,
            Orientation3::ZYX => Orientation3::ZYX,
            Orientation3::ZYx => Orientation3::zYX,
            Orientation3::ZxY => Orientation3::yZX,
            Orientation3::Zxy => Orientation3::yzX,
            Orientation3::ZyX => Orientation3::ZyX,
            Orientation3::Zyx => Orientation3::zyX,
            Orientation3::xYZ => Orientation3::xYZ,
            Orientation3::xYz => Orientation3::xYz,
            Orientation3::xZY => Orientation3::xZY,
            Orientation3::xZy => Orientation3::xzY,
            Orientation3::xyZ => Orientation3::xyZ,
            Orientation3::xyz => Orientation3::xyz,
            Orientation3::xzY => Orientation3::xZy,
            Orientation3::xzy => Orientation3::xzy,
            Orientation3::yXZ => Orientation3::YxZ,
            Orientation3::yXz => Orientation3::Yxz,
            Orientation3::yZX => Orientation3::ZxY,
            Orientation3::yZx => Orientation3::zxY,
            Orientation3::yxZ => Orientation3::yxZ,
            Orientation3::yxz => Orientation3::yxz,
            Orientation3::yzX => Orientation3::Zxy,
            Orientation3::yzx => Orientation3::zxy,
            Orientation3::zXY => Orientation3::YZx,
            Orientation3::zXy => Orientation3::Yzx,
            Orientation3::zYX => Orientation3::ZYx,
            Orientation3::zYx => Orientation3::zYx,
            Orientation3::zxY => Orientation3::yZx,
            Orientation3::zxy => Orientation3::yzx,
            Orientation3::zyX => Orientation3::Zyx,
            Orientation3::zyx => Orientation3::zyx,
        }
    }

    /// The primary [direction](Direction3) of this [orientation](Orientation3).
    pub fn direction1(&self) -> Direction3 {
        match self {
            Orientation3::XYZ => Direction3::East,
            Orientation3::XYz => Direction3::East,
            Orientation3::XZY => Direction3::East,
            Orientation3::XZy => Direction3::East,
            Orientation3::XyZ => Direction3::East,
            Orientation3::Xyz => Direction3::East,
            Orientation3::XzY => Direction3::East,
            Orientation3::Xzy => Direction3::East,
            Orientation3::YXZ => Direction3::Up,
            Orientation3::YXz => Direction3::Up,
            Orientation3::YZX => Direction3::Up,
            Orientation3::YZx => Direction3::Up,
            Orientation3::YxZ => Direction3::Up,
            Orientation3::Yxz => Direction3::Up,
            Orientation3::YzX => Direction3::Up,
            Orientation3::Yzx => Direction3::Up,
            Orientation3::ZXY => Direction3::South,
            Orientation3::ZXy => Direction3::South,
            Orientation3::ZYX => Direction3::South,
            Orientation3::ZYx => Direction3::South,
            Orientation3::ZxY => Direction3::South,
            Orientation3::Zxy => Direction3::South,
            Orientation3::ZyX => Direction3::South,
            Orientation3::Zyx => Direction3::South,
            Orientation3::xYZ => Direction3::West,
            Orientation3::xYz => Direction3::West,
            Orientation3::xZY => Direction3::West,
            Orientation3::xZy => Direction3::West,
            Orientation3::xyZ => Direction3::West,
            Orientation3::xyz => Direction3::West,
            Orientation3::xzY => Direction3::West,
            Orientation3::xzy => Direction3::West,
            Orientation3::yXZ => Direction3::Down,
            Orientation3::yXz => Direction3::Down,
            Orientation3::yZX => Direction3::Down,
            Orientation3::yZx => Direction3::Down,
            Orientation3::yxZ => Direction3::Down,
            Orientation3::yxz => Direction3::Down,
            Orientation3::yzX => Direction3::Down,
            Orientation3::yzx => Direction3::Down,
            Orientation3::zXY => Direction3::North,
            Orientation3::zXy => Direction3::North,
            Orientation3::zYX => Direction3::North,
            Orientation3::zYx => Direction3::North,
            Orientation3::zxY => Direction3::North,
            Orientation3::zxy => Direction3::North,
            Orientation3::zyX => Direction3::North,
            Orientation3::zyx => Direction3::North,
        }
    }

    /// The secondary [direction](Direction3) of this [orientation](Orientation3).
    pub fn direction2(&self) -> Direction3 {
        match self {
            Orientation3::XYZ => Direction3::Up,
            Orientation3::XYz => Direction3::Up,
            Orientation3::XZY => Direction3::South,
            Orientation3::XZy => Direction3::South,
            Orientation3::XyZ => Direction3::Down,
            Orientation3::Xyz => Direction3::Down,
            Orientation3::XzY => Direction3::North,
            Orientation3::Xzy => Direction3::North,
            Orientation3::YXZ => Direction3::East,
            Orientation3::YXz => Direction3::East,
            Orientation3::YZX => Direction3::South,
            Orientation3::YZx => Direction3::South,
            Orientation3::YxZ => Direction3::West,
            Orientation3::Yxz => Direction3::West,
            Orientation3::YzX => Direction3::North,
            Orientation3::Yzx => Direction3::North,
            Orientation3::ZXY => Direction3::East,
            Orientation3::ZXy => Direction3::East,
            Orientation3::ZYX => Direction3::Up,
            Orientation3::ZYx => Direction3::Up,
            Orientation3::ZxY => Direction3::West,
            Orientation3::Zxy => Direction3::West,
            Orientation3::ZyX => Direction3::Down,
            Orientation3::Zyx => Direction3::Down,
            Orientation3::xYZ => Direction3::Up,
            Orientation3::xYz => Direction3::Up,
            Orientation3::xZY => Direction3::South,
            Orientation3::xZy => Direction3::South,
            Orientation3::xyZ => Direction3::Down,
            Orientation3::xyz => Direction3::Down,
            Orientation3::xzY => Direction3::North,
            Orientation3::xzy => Direction3::North,
            Orientation3::yXZ => Direction3::East,
            Orientation3::yXz => Direction3::East,
            Orientation3::yZX => Direction3::South,
            Orientation3::yZx => Direction3::South,
            Orientation3::yxZ => Direction3::West,
            Orientation3::yxz => Direction3::West,
            Orientation3::yzX => Direction3::North,
            Orientation3::yzx => Direction3::North,
            Orientation3::zXY => Direction3::East,
            Orientation3::zXy => Direction3::East,
            Orientation3::zYX => Direction3::Up,
            Orientation3::zYx => Direction3::Up,
            Orientation3::zxY => Direction3::West,
            Orientation3::zxy => Direction3::West,
            Orientation3::zyX => Direction3::Down,
            Orientation3::zyx => Direction3::Down,
        }
    }

    /// The tertiary [direction](Direction3) of this [orientation](Orientation3).
    pub fn direction3(&self) -> Direction3 {
        match self {
            Orientation3::XYZ => Direction3::South,
            Orientation3::XYz => Direction3::North,
            Orientation3::XZY => Direction3::Up,
            Orientation3::XZy => Direction3::Down,
            Orientation3::XyZ => Direction3::South,
            Orientation3::Xyz => Direction3::North,
            Orientation3::XzY => Direction3::Up,
            Orientation3::Xzy => Direction3::Down,
            Orientation3::YXZ => Direction3::South,
            Orientation3::YXz => Direction3::North,
            Orientation3::YZX => Direction3::East,
            Orientation3::YZx => Direction3::West,
            Orientation3::YxZ => Direction3::South,
            Orientation3::Yxz => Direction3::North,
            Orientation3::YzX => Direction3::East,
            Orientation3::Yzx => Direction3::West,
            Orientation3::ZXY => Direction3::Up,
            Orientation3::ZXy => Direction3::Down,
            Orientation3::ZYX => Direction3::East,
            Orientation3::ZYx => Direction3::West,
            Orientation3::ZxY => Direction3::Up,
            Orientation3::Zxy => Direction3::Down,
            Orientation3::ZyX => Direction3::East,
            Orientation3::Zyx => Direction3::West,
            Orientation3::xYZ => Direction3::South,
            Orientation3::xYz => Direction3::North,
            Orientation3::xZY => Direction3::Up,
            Orientation3::xZy => Direction3::Down,
            Orientation3::xyZ => Direction3::South,
            Orientation3::xyz => Direction3::North,
            Orientation3::xzY => Direction3::Up,
            Orientation3::xzy => Direction3::Down,
            Orientation3::yXZ => Direction3::South,
            Orientation3::yXz => Direction3::North,
            Orientation3::yZX => Direction3::East,
            Orientation3::yZx => Direction3::West,
            Orientation3::yxZ => Direction3::South,
            Orientation3::yxz => Direction3::North,
            Orientation3::yzX => Direction3::East,
            Orientation3::yzx => Direction3::West,
            Orientation3::zXY => Direction3::Up,
            Orientation3::zXy => Direction3::Down,
            Orientation3::zYX => Direction3::East,
            Orientation3::zYx => Direction3::West,
            Orientation3::zxY => Direction3::Up,
            Orientation3::zxy => Direction3::Down,
            Orientation3::zyX => Direction3::East,
            Orientation3::zyx => Direction3::West,
        }
    }
}

impl Not for Orientation3 {
    type Output = Orientation3;

    fn not(self) -> Self::Output {
        self.inverse()
    }
}

#[cfg(test)]
mod tests {
    use crate::geometry3::{Coordinate3, Orientation3};
    use strum::IntoEnumIterator;

    #[test]
    fn test_inverse() {
        // given:
        let coordinate = Coordinate3(1, 2, 3);

        for orientation in Orientation3::iter() {
            // when:
            let inverse = orientation.inverse();
            let actual = inverse.orient(orientation.orient(coordinate));

            // then:
            assert_eq!(actual, coordinate, "orientation={:?}", orientation);
        }
    }
}
