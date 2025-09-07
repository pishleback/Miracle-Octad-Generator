pub mod finite_field_4 {
    use std::ops::{Add, Mul};

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum Point {
        Zero,
        One,
        Alpha,
        Beta,
    }

    impl Add<Self> for Point {
        type Output = Self;

        fn add(self, other: Self) -> Self::Output {
            match (self, other) {
                (x, Point::Zero) => x,
                (Point::Zero, x) => x,
                (Point::One, Point::One) => Point::Zero,
                (Point::One, Point::Alpha) => Point::Beta,
                (Point::One, Point::Beta) => Point::Alpha,
                (Point::Alpha, Point::One) => Point::Beta,
                (Point::Alpha, Point::Alpha) => Point::Zero,
                (Point::Alpha, Point::Beta) => Point::One,
                (Point::Beta, Point::One) => Point::Alpha,
                (Point::Beta, Point::Alpha) => Point::One,
                (Point::Beta, Point::Beta) => Point::Zero,
            }
        }
    }

    impl Mul<Self> for Point {
        type Output = Self;

        fn mul(self, other: Self) -> Self::Output {
            match (self, other) {
                (_, Point::Zero) => Point::Zero,
                (Point::Zero, _) => Point::Zero,
                (x, Point::One) => x,
                (Point::One, x) => x,
                (Point::Alpha, Point::Alpha) => Point::Beta,
                (Point::Alpha, Point::Beta) => Point::One,
                (Point::Beta, Point::Alpha) => Point::One,
                (Point::Beta, Point::Beta) => Point::Alpha,
            }
        }
    }

    impl Point {
        pub fn conjugate(self) -> Self {
            match self {
                Point::Zero => Point::Zero,
                Point::One => Point::One,
                Point::Alpha => Point::Beta,
                Point::Beta => Point::Alpha,
            }
        }
    }
}

pub mod hexacode {
    use crate::logic::finite_field_4;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum Side {
        Left,
        Right,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum Pair {
        Left,
        Middle,
        Right,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Point {
        pub side: Side,
        pub pair: Pair,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct Vector {
        components: [finite_field_4::Point; 6],
    }

    impl Vector {
        /*
        Numbering:
        0 1  2 3  4 5
        */
        pub fn usize_to_point(i: usize) -> Point {
            debug_assert!(i < 6);
            Point {
                side: match i % 2 {
                    0 => Side::Left,
                    1 => Side::Right,
                    _ => unreachable!(),
                },
                pair: match i / 2 {
                    0 => Pair::Left,
                    1 => Pair::Middle,
                    2 => Pair::Right,
                    _ => unreachable!(),
                },
            }
        }
        pub fn point_to_usize(p: Point) -> usize {
            (match p.side {
                Side::Left => 0,
                Side::Right => 1,
            }) + 2 * match p.pair {
                Pair::Left => 0,
                Pair::Middle => 1,
                Pair::Right => 2,
            }
        }

        fn from_fn(components: impl Fn(Point) -> finite_field_4::Point) -> Self {
            Self {
                components: std::array::from_fn(|i| components(Self::usize_to_point(i))),
            }
        }

        fn component(&self, p: Point) -> finite_field_4::Point {
            self.components[Self::point_to_usize(p)]
        }
    }
}

pub mod miracle_octad_generator {
    use crate::logic::hexacode;

    use super::finite_field_4;
    use std::{collections::HashSet, ops::Add, vec};

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Point {
        pub col: hexacode::Point,
        pub row: finite_field_4::Point,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
    pub struct Vector {
        components: [bool; 24],
    }

    impl Add<&Vector> for &Vector {
        type Output = Vector;

        fn add(self, other: &Vector) -> Self::Output {
            Vector {
                components: std::array::from_fn(|i| self.components[i] ^ other.components[i]),
            }
        }
    }

    impl Vector {
        pub fn contains(&self, other: &Self) -> bool {
            for i in 0..24 {
                if !self.components[i] && other.components[i] {
                    return false;
                }
            }
            true
        }
    }

    impl Vector {
        /*
        MOG numbering:
        0  |  0  1   2  3   4  5
        1  |  6  7   8  9   10 11
        a  |  12 13  14 15  16 17
        b  |  18 19  20 21  22 23
        */
        pub fn usize_to_point(i: usize) -> Point {
            debug_assert!(i < 24);
            Point {
                col: hexacode::Vector::usize_to_point(i % 6),
                row: match i / 6 {
                    0 => finite_field_4::Point::Zero,
                    1 => finite_field_4::Point::One,
                    2 => finite_field_4::Point::Alpha,
                    3 => finite_field_4::Point::Beta,
                    _ => unreachable!(),
                },
            }
        }
        pub fn point_to_usize(p: Point) -> usize {
            hexacode::Vector::point_to_usize(p.col)
                + 6 * match p.row {
                    finite_field_4::Point::Zero => 0,
                    finite_field_4::Point::One => 1,
                    finite_field_4::Point::Alpha => 2,
                    finite_field_4::Point::Beta => 3,
                }
        }

        pub fn from_fn(components: impl Fn(Point) -> bool) -> Self {
            Self {
                components: std::array::from_fn(|i| components(Self::usize_to_point(i))),
            }
        }

        pub fn zero() -> Self {
            Self {
                components: std::array::from_fn(|i| false),
            }
        }

        pub fn component(&self, p: Point) -> bool {
            self.components[Self::point_to_usize(p)]
        }

        pub fn set_component(&mut self, p: Point, value: bool) {
            self.components[Self::point_to_usize(p)] = value
        }

        pub fn weight(&self) -> usize {
            let mut w = 0;
            for i in 0..24 {
                if self.components[i] {
                    w += 1;
                }
            }
            w
        }

        pub fn points(&self) -> impl Iterator<Item = Point> {
            (0..24).filter_map(|i| {
                if self.components[i] {
                    Some(Self::usize_to_point(i))
                } else {
                    None
                }
            })
        }
    }

    pub struct BinaryGolayCode {
        basis: Vec<Vector>,
        codewords: HashSet<Vector>,
    }

    impl Default for BinaryGolayCode {
        fn default() -> Self {
            // Construct a basis for the binary golay code on the 6x4 MOG grid
            let basis = {
                let mut basis = vec![];

                let col1 = (hexacode::Side::Left, hexacode::Pair::Left);
                for col2 in [
                    (hexacode::Side::Right, hexacode::Pair::Left),
                    (hexacode::Side::Left, hexacode::Pair::Middle),
                    (hexacode::Side::Right, hexacode::Pair::Middle),
                    (hexacode::Side::Left, hexacode::Pair::Right),
                    (hexacode::Side::Right, hexacode::Pair::Right),
                ] {
                    basis.push(Vector::from_fn(|p| {
                        (p.col.side, p.col.pair) == col1 || (p.col.side, p.col.pair) == col2
                    }));
                }

                for val in [
                    finite_field_4::Point::One,
                    finite_field_4::Point::Alpha,
                    finite_field_4::Point::Beta,
                ] {
                    basis.push(Vector::from_fn(|p| match p.col.pair {
                        hexacode::Pair::Left => match p.col.side {
                            hexacode::Side::Left => p.row != finite_field_4::Point::Zero,
                            hexacode::Side::Right => p.row == finite_field_4::Point::Zero,
                        },
                        hexacode::Pair::Middle | hexacode::Pair::Right => p.row == val,
                    }))
                }

                basis.push(Vector::from_fn(|p| match (p.col.side, p.col.pair) {
                    (hexacode::Side::Left, hexacode::Pair::Left) => {
                        p.row != finite_field_4::Point::Zero
                    }
                    (hexacode::Side::Right, hexacode::Pair::Left) => {
                        p.row == finite_field_4::Point::One
                    }
                    (hexacode::Side::Left, hexacode::Pair::Middle) => {
                        p.row == finite_field_4::Point::Zero
                    }
                    (hexacode::Side::Right, hexacode::Pair::Middle) => {
                        p.row == finite_field_4::Point::One
                    }
                    (hexacode::Side::Left, hexacode::Pair::Right) => {
                        p.row == finite_field_4::Point::Alpha
                    }
                    (hexacode::Side::Right, hexacode::Pair::Right) => {
                        p.row == finite_field_4::Point::Beta
                    }
                }));

                basis.push(Vector::from_fn(|p| match (p.col.side, p.col.pair) {
                    (hexacode::Side::Left, hexacode::Pair::Left) => {
                        p.row != finite_field_4::Point::Zero
                    }
                    (hexacode::Side::Right, hexacode::Pair::Left) => {
                        p.row == finite_field_4::Point::Alpha
                    }
                    (hexacode::Side::Left, hexacode::Pair::Middle) => {
                        p.row == finite_field_4::Point::Zero
                    }
                    (hexacode::Side::Right, hexacode::Pair::Middle) => {
                        p.row == finite_field_4::Point::Alpha
                    }
                    (hexacode::Side::Left, hexacode::Pair::Right) => {
                        p.row == finite_field_4::Point::Beta
                    }
                    (hexacode::Side::Right, hexacode::Pair::Right) => {
                        p.row == finite_field_4::Point::One
                    }
                }));

                basis.push(Vector::from_fn(|p| match (p.col.side, p.col.pair) {
                    (hexacode::Side::Left, hexacode::Pair::Left) => {
                        p.row == finite_field_4::Point::One
                    }
                    (hexacode::Side::Right, hexacode::Pair::Left) => {
                        p.row != finite_field_4::Point::Zero
                    }
                    (hexacode::Side::Left, hexacode::Pair::Middle) => {
                        p.row == finite_field_4::Point::Zero
                    }
                    (hexacode::Side::Right, hexacode::Pair::Middle) => {
                        p.row == finite_field_4::Point::One
                    }
                    (hexacode::Side::Left, hexacode::Pair::Right) => {
                        p.row == finite_field_4::Point::Beta
                    }
                    (hexacode::Side::Right, hexacode::Pair::Right) => {
                        p.row == finite_field_4::Point::Alpha
                    }
                }));

                basis.push(Vector::from_fn(|p| match (p.col.side, p.col.pair) {
                    (hexacode::Side::Left, hexacode::Pair::Left) => {
                        p.row == finite_field_4::Point::Alpha
                    }
                    (hexacode::Side::Right, hexacode::Pair::Left) => {
                        p.row != finite_field_4::Point::Zero
                    }
                    (hexacode::Side::Left, hexacode::Pair::Middle) => {
                        p.row == finite_field_4::Point::Zero
                    }
                    (hexacode::Side::Right, hexacode::Pair::Middle) => {
                        p.row == finite_field_4::Point::Alpha
                    }
                    (hexacode::Side::Left, hexacode::Pair::Right) => {
                        p.row == finite_field_4::Point::One
                    }
                    (hexacode::Side::Right, hexacode::Pair::Right) => {
                        p.row == finite_field_4::Point::Beta
                    }
                }));

                basis
            };

            // Compute the span of the basis to obtain the codewords in the binary golay code
            let mut codewords = HashSet::new();
            for b in 0usize..(1 << basis.len()) {
                let mut codeword = Vector::zero();
                for i in 0..basis.len() {
                    if b & (1usize << i) != 0 {
                        codeword = &codeword + &basis[i];
                    }
                }
                debug_assert!(!codewords.contains(&codeword));
                codewords.insert(codeword);
            }

            // Sanity checks
            for vector in &codewords {
                debug_assert!(
                    vector.weight() == 0
                        || vector.weight() == 8
                        || vector.weight() == 12
                        || vector.weight() == 16
                        || vector.weight() == 24
                );
            }
            debug_assert_eq!(basis.len(), 12);
            debug_assert_eq!(codewords.len(), 1usize << 12);

            Self { basis, codewords }
        }
    }

    impl BinaryGolayCode {
        pub fn is_codeword(&self, vector: Vector) -> bool {
            self.codewords.contains(&vector)
        }

        pub fn is_octad(&self, vector: Vector) -> bool {
            vector.weight() == 8 && self.codewords.contains(&vector)
        }

        // If the input vector has weight 5, return the unique octad containing it
        // Otherwise, return an Err
        pub fn complete_octad(&self, vector: Vector) -> Result<Vector, ()> {
            if vector.weight() != 5 {
                return Err(());
            }
            for codeword in &self.codewords {
                if codeword.weight() == 8 && codeword.contains(&vector) {
                    return Ok(codeword.clone());
                }
            }
            unreachable!()
        }

        pub fn complete_sextet(&self, vector: Vector) -> Result<HashSet<Vector>, ()> {
            if vector.weight() != 4 {
                return Err(());
            }
            let mut sextet = HashSet::new();
            for codeword in &self.codewords {
                let offset = codeword + &vector;
                if offset.weight() == 4 {
                    sextet.insert(offset);
                }
            }
            debug_assert_eq!(sextet.len(), 6);
            Ok(sextet)
        }
    }
}
