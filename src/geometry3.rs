// Minect is library that allows a program to connect to a running Minecraft instance without
// requiring any Minecraft mods.
//
// © Copyright (C) 2021-2023 Adrodoc <adrodoc55@googlemail.com> & skess42 <skagaros@gmail.com>
//
// This file is part of Minect.
//
// Minect is free software: you can redistribute it and/or modify it under the terms of the GNU
// General Public License as published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// Minect is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even
// the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General
// Public License for more details.
//
// You should have received a copy of the GNU General Public License along with Minect.
// If not, see <http://www.gnu.org/licenses/>.

#![allow(dead_code)]

use std::{
    cmp::{max, min},
    convert::TryFrom,
    fmt::{self, Display},
    ops::{Add, AddAssign, Index, IndexMut, Neg, Not, Rem, Sub},
};

use num_traits::Signed;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Coordinate3<T>(pub(crate) T, pub(crate) T, pub(crate) T);

impl<T> Coordinate3<T> {
    pub(crate) fn map<F, R>(self, f: F) -> Coordinate3<R>
    where
        F: Fn(T) -> R,
    {
        Coordinate3(f(self.0), f(self.1), f(self.2))
    }

    pub(crate) fn zip<F, U, R>(self, other: Coordinate3<U>, f: F) -> Coordinate3<R>
    where
        F: Fn(T, U) -> R,
    {
        Coordinate3(f(self.0, other.0), f(self.1, other.1), f(self.2, other.2))
    }
}

impl<T: Ord> Coordinate3<T> {
    /// Finds the [Coordinate3] of the minimum corner in the cuboid spanned by `a` and `b`.
    pub(crate) fn min(a: Coordinate3<T>, b: Coordinate3<T>) -> Coordinate3<T> {
        a.zip(b, min)
    }

    /// Finds the [Coordinate3] of the maximum corner in the cuboid spanned by `a` and `b`.
    pub(crate) fn max(a: Coordinate3<T>, b: Coordinate3<T>) -> Coordinate3<T> {
        a.zip(b, max)
    }
}

impl<T: Clone + Neg<Output = T>> Coordinate3<T> {
    pub(crate) fn get_in_direction(&self, direction: Direction3) -> T {
        let raw = self[(direction.axis())].clone();
        if direction.is_negative() {
            -raw
        } else {
            raw
        }
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

impl<T: Neg> Neg for Coordinate3<T> {
    type Output = Coordinate3<T::Output>;

    fn neg(self) -> Self::Output {
        self.map(Neg::neg)
    }
}

impl<T: Add> Add for Coordinate3<T> {
    type Output = Coordinate3<T::Output>;

    fn add(self, rhs: Self) -> Self::Output {
        self.zip(rhs, Add::add)
    }
}

impl<T: Clone + Add<Output = T>> AddAssign for Coordinate3<T> {
    fn add_assign(&mut self, rhs: Self) {
        *self = self.clone().add(rhs);
    }
}

impl<T: Sub> Sub for Coordinate3<T> {
    type Output = Coordinate3<T::Output>;

    fn sub(self, rhs: Self) -> Self::Output {
        self.zip(rhs, Sub::sub)
    }
}

impl<T: Rem> Rem for Coordinate3<T> {
    type Output = Coordinate3<T::Output>;

    fn rem(self, rhs: Self) -> Self::Output {
        self.zip(rhs, Rem::rem)
    }
}

impl<T> From<Coordinate3<T>> for Vec<T> {
    fn from(c: Coordinate3<T>) -> Self {
        vec![c.0, c.1, c.2]
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Axis3 {
    X,
    Y,
    Z,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Direction3 {
    /// +X
    East,
    /// -X
    West,
    /// +Y
    Up,
    /// -Y
    Down,
    /// +Z
    South,
    /// -Z
    North,
}

impl Direction3 {
    pub(crate) fn axis(&self) -> Axis3 {
        match self {
            Direction3::East => Axis3::X,
            Direction3::West => Axis3::X,
            Direction3::Up => Axis3::Y,
            Direction3::Down => Axis3::Y,
            Direction3::South => Axis3::Z,
            Direction3::North => Axis3::Z,
        }
    }

    pub(crate) fn is_positive(&self) -> bool {
        match self {
            Direction3::East => true,
            Direction3::West => false,
            Direction3::Up => true,
            Direction3::Down => false,
            Direction3::South => true,
            Direction3::North => false,
        }
    }

    pub(crate) fn is_negative(&self) -> bool {
        !self.is_positive()
    }

    pub(crate) fn signum(&self) -> i8 {
        if self.is_positive() {
            1
        } else {
            -1
        }
    }

    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Direction3::East => "east",
            Direction3::West => "west",
            Direction3::Up => "up",
            Direction3::Down => "down",
            Direction3::South => "south",
            Direction3::North => "north",
        }
    }

    pub(crate) fn as_coordinate<N>(&self, one: N, zero: N) -> Coordinate3<N>
    where
        N: Clone + Neg<Output = N>,
    {
        match self {
            Direction3::East => Coordinate3(one, zero.clone(), zero),
            Direction3::West => Coordinate3(-one, zero.clone(), zero),
            Direction3::Up => Coordinate3(zero.clone(), one, zero),
            Direction3::Down => Coordinate3(zero.clone(), -one, zero),
            Direction3::South => Coordinate3(zero.clone(), zero, one),
            Direction3::North => Coordinate3(zero.clone(), zero, -one),
        }
    }
}

impl Display for Direction3 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Neg for Direction3 {
    type Output = Direction3;

    fn neg(self) -> Self::Output {
        match self {
            Direction3::East => Self::West,
            Direction3::West => Self::East,
            Direction3::Up => Self::Down,
            Direction3::Down => Self::Up,
            Direction3::South => Self::North,
            Direction3::North => Self::South,
        }
    }
}

impl<S: Signed> TryFrom<Coordinate3<S>> for Direction3 {
    type Error = ();

    fn try_from(value: Coordinate3<S>) -> Result<Self, Self::Error> {
        let Coordinate3(s1, s2, s3) = value;
        let count = [
            s1.is_positive(),
            s1.is_negative(),
            s2.is_positive(),
            s2.is_negative(),
            s3.is_positive(),
            s3.is_negative(),
        ]
        .iter()
        .filter(|&&it| it)
        .count();

        if count != 1 {
            Err(())
        } else if s1.is_positive() {
            Ok(Direction3::East)
        } else if s1.is_negative() {
            Ok(Direction3::West)
        } else if s2.is_positive() {
            Ok(Direction3::Up)
        } else if s2.is_negative() {
            Ok(Direction3::Down)
        } else if s3.is_positive() {
            Ok(Direction3::South)
        } else if s3.is_negative() {
            Ok(Direction3::North)
        } else {
            Err(())
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(test, derive(strum::EnumIter))]
/// An [Orientation3] represents one of the 48 ways you can allign a structure to a grid in three
/// dimensions via rotation and mirroring. It consists of a
/// [primary](Orientation3::direction1()), a [secondary](Orientation3::direction2())
/// and a [tertiary](Orientation3::direction3()) [direction](Direction3) which are all
/// on different [axes](Axis3). Lower case axis names represent a negative direction.
///
/// An [Orientation3] can also be represented by a left multiplied rotation matrix where the first
/// column represents the primary direction, the second column the secondary and so on. A value of 1
/// indicates a positive direction and a value of -1 indicates a negative direction. Each column and
/// row contains exactly one non zero value, as all directions are on different axes.
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
pub(crate) enum Orientation3 {
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
    /// Orient a [Coordinate3] according to this [Orientation3]. This can be seen as renaming the
    /// axis of the first component to the [primary](Orientation3::direction1), the second to the
    /// secondary and so on. Lower case axis names represent a negative [Direction3], so there the
    /// component is inverted too.
    ///
    /// For example orienting the coordinate (x: 1, y: 2, z: 3) to [Orientation3::YzX] results in
    /// (y: 1, z: -2, x: 3) or short (3, 1, -2).
    ///
    /// Some orientations are their own reverse, bot others are not. You can undo by orienting to
    /// the [inverse](Orientation3::inverse) orientation.
    // TODO consume to avoid clone
    pub(crate) fn orient_coordinate<T: Clone + Neg<Output = T>>(
        &self,
        coordinate: Coordinate3<T>,
    ) -> Coordinate3<T> {
        let mut result = coordinate.clone();
        let Coordinate3(mut t1, mut t2, mut t3) = coordinate;
        if self.direction1().is_negative() {
            t1 = -t1;
        }
        if self.direction2().is_negative() {
            t2 = -t2;
        }
        if self.direction3().is_negative() {
            t3 = -t3;
        }
        result[self.direction1().axis()] = t1;
        result[self.direction2().axis()] = t2;
        result[self.direction3().axis()] = t3;
        result
    }

    pub(crate) fn orient_direction(&self, direction: Direction3) -> Direction3 {
        let target_direction = match direction.axis() {
            Axis3::X => self.direction1(),
            Axis3::Y => self.direction2(),
            Axis3::Z => self.direction3(),
        };
        if direction.is_positive() {
            target_direction
        } else {
            -target_direction
        }
    }

    /// Find the inverse [Orientation3]. This is equivalent to inverting the rotation matrix, which
    /// is the same as transposing it.
    pub(crate) const fn inverse(&self) -> Orientation3 {
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
    pub(crate) fn direction1(&self) -> Direction3 {
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
    pub(crate) fn direction2(&self) -> Direction3 {
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
    pub(crate) fn direction3(&self) -> Direction3 {
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
            let actual = inverse.orient_coordinate(orientation.orient_coordinate(coordinate));

            // then:
            assert_eq!(actual, coordinate, "orientation={:?}", orientation);
        }
    }
}
