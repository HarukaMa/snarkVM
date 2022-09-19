// Copyright (C) 2019-2022 Aleo Systems Inc.
// This file is part of the snarkVM library.

// The snarkVM library is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// The snarkVM library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with the snarkVM library. If not, see <https://www.gnu.org/licenses/>.

use crate::{templates::short_weierstrass_jacobian, PairingEngine};
use snarkvm_fields::{Field, FieldParameters, One, PrimeField, SquareRootField, Zero};
use snarkvm_utilities::{rand::Uniform, serialize::*, BigInteger384, FromBytes, ToBytes, ToMinimalBits};

use core::{
    fmt::{Debug, Display},
    hash::Hash,
    iter,
    ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};
use serde::{de::DeserializeOwned, Serialize};

/// Projective representation of an elliptic curve point guaranteed to be in the prime order subgroup.
pub trait ProjectiveCurve:
    CanonicalSerialize
    + CanonicalDeserialize
    + Copy
    + Clone
    + Debug
    + Display
    + Default
    + FromBytes
    + Send
    + Sync
    + 'static
    + Eq
    + Hash
    + Neg<Output = Self>
    + Uniform
    + Zero
    + Add<Self, Output = Self>
    + Sub<Self, Output = Self>
    + Mul<Self::ScalarField, Output = Self>
    + AddAssign<Self>
    + SubAssign<Self>
    + MulAssign<Self::ScalarField>
    + for<'a> Add<&'a Self, Output = Self>
    + for<'a> Sub<&'a Self, Output = Self>
    + for<'a> AddAssign<&'a Self>
    + for<'a> SubAssign<&'a Self>
    + PartialEq<Self::Affine>
    + Sized
    + ToBytes
    + iter::Sum
    + From<<Self as ProjectiveCurve>::Affine>
{
    type Affine: AffineCurve<Projective = Self, ScalarField = Self::ScalarField> + From<Self> + Into<Self>;
    type BaseField: Field;
    type ScalarField: PrimeField + SquareRootField + Into<<Self::ScalarField as PrimeField>::BigInteger>;

    /// Returns a fixed generator of unknown exponent.
    #[must_use]
    fn prime_subgroup_generator() -> Self;

    /// Normalizes a slice of projective elements so that
    /// conversion to affine is cheap.
    fn batch_normalization(v: &mut [Self]);

    /// Normalizes a slice of projective elements and outputs a vector
    /// containing the affine equivalents.
    fn batch_normalization_into_affine(mut v: Vec<Self>) -> Vec<Self::Affine> {
        Self::batch_normalization(&mut v);
        v.into_iter().map(|v| v.into()).collect()
    }

    /// Checks if the point is already "normalized" so that
    /// cheap affine conversion is possible.
    #[must_use]
    fn is_normalized(&self) -> bool;

    /// Adds an affine element to this element.
    fn add_assign_mixed(&mut self, other: &Self::Affine);

    /// Adds an affine element to this element.
    fn add_mixed(&self, other: &Self::Affine) -> Self {
        let mut copy = *self;
        copy.add_assign_mixed(other);
        copy
    }

    /// Adds an affine element to this element.
    fn sub_assign_mixed(&mut self, other: &Self::Affine) {
        self.add_assign_mixed(&-*other);
    }

    /// Returns `self + self`.
    #[must_use]
    fn double(&self) -> Self;

    /// Sets `self := self + self`.
    fn double_in_place(&mut self);

    /// Converts this element into its affine representation.
    #[must_use]
    #[allow(clippy::wrong_self_convention)]
    fn to_affine(&self) -> Self::Affine;
}

/// Affine representation of an elliptic curve point guaranteed to be
/// in the correct prime order subgroup.
#[allow(clippy::wrong_self_convention)]
pub trait AffineCurve:
    CanonicalSerialize
    + CanonicalDeserialize
    + Copy
    + Clone
    + Debug
    + Display
    + Default
    + FromBytes
    + Send
    + Sync
    + 'static
    + Eq
    + Hash
    + Neg<Output = Self>
    + Uniform
    + PartialEq<Self::Projective>
    + Mul<Self::ScalarField, Output = Self::Projective>
    + Sized
    + Serialize
    + DeserializeOwned
    + ToBytes
    + From<<Self as AffineCurve>::Projective>
    + ToMinimalBits
    + Zero
{
    type Projective: ProjectiveCurve<Affine = Self, ScalarField = Self::ScalarField> + From<Self> + Into<Self>;
    type BaseField: Field + SquareRootField;
    type ScalarField: PrimeField + SquareRootField + Into<<Self::ScalarField as PrimeField>::BigInteger>;
    type Coordinates;

    /// Initializes a new affine group element from the given coordinates.
    fn from_coordinates(coordinates: Self::Coordinates) -> Self;

    /// Returns the cofactor of the curve.
    fn cofactor() -> &'static [u64];

    /// Returns a fixed generator of unknown exponent.
    #[must_use]
    fn prime_subgroup_generator() -> Self;

    /// Attempts to construct an affine point given an x-coordinate. The
    /// point is not guaranteed to be in the prime order subgroup.
    ///
    /// If and only if `greatest` is set will the lexicographically
    /// largest y-coordinate be selected.
    fn from_x_coordinate(x: Self::BaseField, greatest: bool) -> Option<Self>;

    /// Attempts to construct an affine point given a y-coordinate. The
    /// point is not guaranteed to be in the prime order subgroup.
    ///
    /// If and only if `greatest` is set will the lexicographically
    /// largest y-coordinate be selected.
    fn from_y_coordinate(y: Self::BaseField, greatest: bool) -> Option<Self>;

    /// Multiply this element by the cofactor and output the
    /// resulting projective element.
    #[must_use]
    fn mul_by_cofactor_to_projective(&self) -> Self::Projective;

    /// Converts this element into its projective representation.
    #[must_use]
    fn to_projective(&self) -> Self::Projective;

    /// Returns a group element if the set of bytes forms a valid group element,
    /// otherwise returns None. This function is primarily intended for sampling
    /// random group elements from a hash-function or RNG output.
    fn from_random_bytes(bytes: &[u8]) -> Option<Self>;

    /// Multiply this element by a big-endian boolean representation of
    /// an integer.
    fn mul_bits(&self, bits: impl Iterator<Item = bool>) -> Self::Projective;

    /// Multiply this element by the cofactor.
    #[must_use]
    fn mul_by_cofactor(&self) -> Self {
        self.mul_by_cofactor_to_projective().into()
    }

    /// Multiply this element by the inverse of the cofactor modulo the size of
    /// `Self::ScalarField`.
    #[must_use]
    fn mul_by_cofactor_inv(&self) -> Self;

    /// Checks that the point is in the prime order subgroup given the point on the curve.
    #[must_use]
    fn is_in_correct_subgroup_assuming_on_curve(&self) -> bool;

    /// Returns the x-coordinate of the point.
    #[must_use]
    fn to_x_coordinate(&self) -> Self::BaseField;

    /// Returns the y-coordinate of the point.
    #[must_use]
    fn to_y_coordinate(&self) -> Self::BaseField;

    /// Checks that the current point is on the elliptic curve.
    fn is_on_curve(&self) -> bool;

    /// Performs the first half of batch addition in-place.
    fn batch_add_loop_1(
        a: &mut Self,
        b: &mut Self,
        half: &Self::BaseField, // The value 2.inverse().
        inversion_tmp: &mut Self::BaseField,
    );

    /// Performs the second half of batch addition in-place.
    fn batch_add_loop_2(a: &mut Self, b: Self, inversion_tmp: &mut Self::BaseField);
}

pub trait PairingCurve: AffineCurve {
    type Engine: PairingEngine<Fr = Self::ScalarField>;
    type Prepared: CanonicalSerialize
        + CanonicalDeserialize
        + ToBytes
        + FromBytes
        + PartialEq
        + Eq
        + Default
        + Clone
        + Send
        + Sync
        + Debug
        + 'static;
    type PairWith: PairingCurve<PairWith = Self>;
    type PairingResult: Field;

    /// Prepares this element for pairing purposes.
    #[must_use]
    fn prepare(&self) -> Self::Prepared;

    /// Perform a pairing
    #[must_use]
    fn pairing_with(&self, other: &Self::PairWith) -> Self::PairingResult;
}

pub trait ModelParameters: 'static + Copy + Clone + Debug + PartialEq + Eq + Hash + Send + Sync + Sized {
    type BaseField: Field + SquareRootField;
    type ScalarField: PrimeField + SquareRootField + Into<<Self::ScalarField as PrimeField>::BigInteger>;
}

pub trait ShortWeierstrassParameters: ModelParameters {
    /// The coefficient `A` of the short Weierstrass curve.
    const WEIERSTRASS_A: Self::BaseField;
    /// The coefficient `B` of the short Weierstrass curve.
    const WEIERSTRASS_B: Self::BaseField;
    /// The cofactor of the short Weierstrass curve.
    const COFACTOR: &'static [u64];
    /// The cofactor inverse of the short Weierstrass curve.
    const COFACTOR_INV: Self::ScalarField;
    /// The affine generator of the short Weierstrass curve.
    const AFFINE_GENERATOR_COEFFS: (Self::BaseField, Self::BaseField);

    // GLV parameters
    // /// The scalar multiplication window size.
    // const GLV_WINDOW_SIZE: usize = 4;
    // /// The GLV table length.
    // const L: usize = 1 << (Self::GLV_WINDOW_SIZE - 1);
    // /// The curve parameter `x`.
    // /// R / q
    // /// x^2 - 1
    // const B_1: Self::ScalarField = Self::X.square() - Self::ScalarField::one();
    // /// x^2
    // const B_2: Self::ScalarField = Self::X.square();
    // const LAMBDA: Self::ScalarField = Self::B_1;
    // const HALF_R: Self::ScalarField =
    //     Self::ScalarField::from_repr(<Self::ScalarField as PrimeField>::BigInteger::from(0x8000000000000000)).unwrap();
    // const R_128: Self::ScalarField =
    //     Self::ScalarField::from_repr(<Self::ScalarField as PrimeField>::BigInteger::from([
    //         0xffffffffffffffff,
    //         0xffffffffffffffff,
    //         0,
    //         0,
    //     ]))
    //     .unwrap();
    // const PHI_1: Self::BaseField = Field::from_base_prime_field(
    //     <Self::BaseField as Field>::BasePrimeField::from_repr(
    //         <<Self::BaseField as Field>::BasePrimeField as PrimeField>::BigInteger::from([
    //             9586122913090633729,
    //             4981570305181876224,
    //             14262076793150106624,
    //             7033126720376490667,
    //             699094806891394796,
    //             0,
    //         ]),
    //     )
    //     .unwrap(),
    // );
    // const PHI_2: Self::BaseField = Self::PHI_1.square();

    #[inline(always)]
    fn mul_by_a(elem: &Self::BaseField) -> Self::BaseField {
        let mut copy = *elem;
        copy *= &Self::WEIERSTRASS_A;
        copy
    }

    #[inline(always)]
    fn add_b(elem: &Self::BaseField) -> Self::BaseField {
        let mut copy = *elem;
        copy += &Self::WEIERSTRASS_B;
        copy
    }

    fn is_in_correct_subgroup_assuming_on_curve(p: &short_weierstrass_jacobian::Affine<Self>) -> bool;
}

pub trait TwistedEdwardsParameters: ModelParameters {
    /// The coefficient `A` of the twisted Edwards curve.
    const EDWARDS_A: Self::BaseField;
    /// The coefficient `D` of the twisted Edwards curve.
    const EDWARDS_D: Self::BaseField;
    /// The cofactor of the twisted Edwards curve.
    const COFACTOR: &'static [u64];
    /// The cofactor inverse of the twisted Edwards curve.
    const COFACTOR_INV: Self::ScalarField;
    /// The affine generator of the twisted Edwards curve.
    const AFFINE_GENERATOR_COEFFS: (Self::BaseField, Self::BaseField);

    type MontgomeryParameters: MontgomeryParameters<BaseField = Self::BaseField>;

    #[inline(always)]
    fn mul_by_a(elem: &Self::BaseField) -> Self::BaseField {
        let mut copy = *elem;
        copy *= &Self::EDWARDS_A;
        copy
    }
}

pub trait MontgomeryParameters: ModelParameters {
    /// The coefficient `A` of the Montgomery curve.
    const MONTGOMERY_A: Self::BaseField;
    /// The coefficient `B` of the Montgomery curve.
    const MONTGOMERY_B: Self::BaseField;

    type TwistedEdwardsParameters: TwistedEdwardsParameters<BaseField = Self::BaseField>;
}
