/* ************************************************************************ **
** This file is part of rsp2, and is licensed under EITHER the MIT license  **
** or the Apache 2.0 license, at your option.                               **
**                                                                          **
**     http://www.apache.org/licenses/LICENSE-2.0                           **
**     http://opensource.org/licenses/MIT                                   **
**                                                                          **
** Be aware that not all of rsp2 is provided under this permissive license, **
** and that the project as a whole is licensed under the GPL 3.0.           **
** ************************************************************************ */

//! Small fixed-size matrix types, compatible with `V2`/`V3`/`V4`
//!
//! This library primarily uses a row-based formalism; matrices are conceptually
//! understood to be containers of row-vectors. This formalism is most useful when
//! most vectors used are row vectors (in which case most matrix-vector multiplication
//! has the matrix on the right)

use crate::traits::{Semiring, Ring, Field};
use crate::traits::internal::{PrimitiveSemiring, PrimitiveRing, PrimitiveFloat};
use crate::vee;
use crate::types::*;
use crate::{Unvee, Envee};
use num_traits::{Zero, One};

// ---------------------------------------------------------------------------
// ------------------------------ PUBLIC API ---------------------------------

/// Construct a matrix from a function on indices.
///
/// The shape of the matrix will be inferred solely from how it
/// is used.  There is also a static method form of this for
/// easily supplying a type hint. (e.g. `M33::from_fn`)
#[inline(always)]
pub(crate) fn from_fn<M: FromFn<F>, B, F>(f: F) -> M
where F: FnMut(usize, usize) -> B,
{ FromFn::from_fn(f) }

/// Construct a matrix from a 2D array (of rows).
///
/// This is intended to be used in places where an array of known
/// shape already exists, and needs to be wrapped into a matrix.
/// The signature is such that type inference will work in the
/// forward direction (deciding the output Matrix shape from
/// the input array).
#[inline(always)]
pub fn from_array<A: IntoMatrix>(arr: A) -> A::Matrix
{ arr.into_matrix() }

/// Matrix inverse.
#[inline(always)]
pub fn inv<M: Inv>(m: &M) -> M
where M: Inv,
{ Inv::inv(m) }

// Square matrices
gen_each!{
    @{Mnn_Mn_Vn_n}
    for_each!(
        {$Mnn:ident $Mn:ident $Vn:ident $n:tt}
    ) => {
        impl<X> $Mnn<X> {
            /// Construct the identity matrix.
            #[inline(always)]
            pub fn eye() -> Self
            where Self: One,
            { One::one() }

            /// Construct a diagonal matrix.
            #[inline(always)]
            pub fn from_diag(diag: DiagT<Self>) -> Self
            where Self: FromDiag,
            { FromDiag::from_diag(diag) }

            /// Matrix inverse.
            ///
            /// It is recommended to write this as `M33::inv(m)` or etc. rather than `m.inv()`.
            #[inline(always)]
            pub fn inv(&self) -> Self
            where Self: Inv,
            { Inv::inv(self) }

            /// Matrix determinant.
            #[inline(always)]
            pub fn det(&self) -> DetT<Self>
            where Self: Det,
            { Det::det(self) }
        }
    }
}

// General rectangular, for things that must be generic over V.
// (due to e.g. an associated type)
gen_each!{
    @{Mn_n}
    for_each!(
        {$Mr:ident $r:tt}
    ) => {
        impl<V> $Mr<V> {
            /// Matrix transpose. (does not conjugate)
            #[inline(always)]
            pub fn t(&self) -> TransposeT<Self>
            where Self: Transpose,
            { Transpose::t(self) }

            /// Cast into a plain `[[T; m]; n]`.
            #[inline(always)]
            pub fn into_array(self) -> ArrayT<Self>
            where Self: IntoArray,
            { IntoArray::into_array(self) }

            /// Cast into a plain `&[[T; m]; n]`.
            #[inline(always)]
            pub fn as_array(&self) -> &ArrayT<Self>
            where Self: IntoArray,
            { IntoArray::as_array(self) }

            /// Cast into a plain `&mut [[T; m]; n]`.
            #[inline(always)]
            pub fn as_array_mut(&mut self) -> &mut ArrayT<Self>
            where Self: IntoArray,
            { IntoArray::as_array_mut(self) }
        }
    }
}

// General rectangular.
gen_each!{
    @{Mnn_Mn_Vn_n}
    @{Vn_n}
    for_each!(
        {$_Mrr:ident $Mr:ident $Vr:ident $r:tt}
        {$Vc:ident $c:tt}
    ) => {
        impl<X> $Mr<$Vc<X>> {
            /// Construct the zero matrix.
            #[inline(always)]
            pub fn zero() -> Self
            where Self: Zero,
            { Zero::zero() }

            /// Construct a fixed-size matrix from a function on indices.
            #[inline(always)]
            pub fn from_fn<B, F>(f: F) -> Self
            where Self: FromFn<F>, F: FnMut(usize, usize) -> B,
            { FromFn::from_fn(f) }

            /// Map each scalar element of a matrix.
            #[inline(always)]
            pub fn map<B, F>(self, mut f: F) -> $Mr<$Vc<B>>
            where F: FnMut(X) -> B,
            { $Mr($Vr(self.0).map(|v| v.map(&mut f)).0) }

            /// Apply a fallible function to each scalar element, with short-circuiting.
            #[inline(always)]
            pub fn try_map<E, B, F>(self, mut f: F) -> Result<$Mr<$Vc<B>>, E>
            where F: FnMut(X) -> Result<B, E>,
            { Ok($Mr($Vr(self.0).try_map(|v| v.try_map(&mut f))?.0)) }

            /// Apply a fallible function to each scalar element, with short-circuiting.
            #[inline(always)]
            pub fn opt_map<B, F>(self, mut f: F) -> Option<$Mr<$Vc<B>>>
            where F: FnMut(X) -> Option<B>,
            { Some($Mr($Vr(self.0).opt_map(|v| v.opt_map(&mut f))?.0)) }
        }
    }
}

// -------------------------- END PUBLIC API ---------------------------------
// The rest is implementation and boiler boiler boiiiiler boiilerplaaaaate
// ---------------------------------------------------------------------------

/// Implementation detail of some free functions that defer to external traits.
///
/// > **_Fuggedaboudit._**
///
/// Its purpose is to prevent those functions from producing non-matrix types.
pub trait IsMatrix: Sized { }

// christ on a cracker, even THIS takes 10 lines to implement.
gen_each!{
    @{Mn_n}
    @{Vn_n}
    for_each!(
        {$Mr:ident $r:tt}
        {$Vc:ident $c:tt}
    ) => {
        impl<X> IsMatrix for $Mr<$Vc<X>> { }
    }
}

// ---------------------------------------------------------------------------

gen_each!{
    @{Mn_n}
    @{Vn_n}
    for_each!(
        {$Mr:ident $r:tt}
        {$Vc:ident $c:tt}
    ) => {
        impl<X: Semiring> Zero for $Mr<$Vc<X>>
        where X: PrimitiveSemiring,
        {
            #[inline]
            fn zero() -> Self
            { from_array([[X::zero(); $c]; $r]) }

            #[inline]
            fn is_zero(&self) -> bool
            { self.0.iter().all(|row| row.iter().all(|x| x.is_zero())) }
        }
    }
}

// ---------------------------------------------------------------------------

/// Implementation detail of the free function `mat::from_fn`.
///
/// > **_Fuggedaboudit._**
pub trait FromFn<F>: Sized {
    fn from_fn(f: F) -> Self;
}

gen_each!{
    @{Mn_n}
    @{Vn_n}
    for_each!(
        {$Mr:ident $r:tt}
        {$Vc:ident $c:tt}
    ) => {
        impl<X, F> FromFn<F> for $Mr<$Vc<X>>
          where F: FnMut(usize, usize) -> X,
        {
            #[inline]
            fn from_fn(mut f: F) -> Self {
                $Mr(<V![$r, _]>::from_fn(|r| {
                    <$Vc<_>>::from_fn(|c| f(r, c))
                }).0)
            }
        }
    }
}

// ---------------------------------------------------------------------------

/// Implementation detail of the free function `mat::from_array`.
///
/// > **_Fuggedaboudit._**
pub trait IntoMatrix: Sized {
    type Matrix;

    fn into_matrix(self) -> Self::Matrix;
}

pub type ArrayT<M> = <M as IntoArray>::Array;

/// Implementation detail of the inherent method `{M2,M3,M4}::into_array`.
///
/// > **_Fuggedaboudit._**
pub trait IntoArray: Sized {
    type Array;

    fn into_array(self) -> Self::Array;
    fn as_array(&self) -> &Self::Array;
    fn as_array_mut(&mut self) -> &mut Self::Array;
}

gen_each!{
    @{Mn_n}
    @{Vn_n}
    for_each!(
        {$Mr:ident $r:tt}
        {$Vc:ident $c:tt}
    ) => {
        impl<X> IntoMatrix for [[X; $c]; $r] {
            type Matrix = $Mr<$Vc<X>>;

            #[inline(always)]
            fn into_matrix(self) -> Self::Matrix
            { $Mr(self.envee()) }
        }

        impl<X> IntoArray for $Mr<$Vc<X>> {
            type Array = [[X; $c]; $r];

            #[inline(always)]
            fn into_array(self) -> Self::Array
            { self.0.unvee() }

            #[inline(always)]
            fn as_array(&self) -> &Self::Array
            { self.0.unvee_ref() }

            #[inline(always)]
            fn as_array_mut(&mut self) -> &mut Self::Array
            { self.0.unvee_mut() }
        }
    }
}

// ---------------------------------------------------------------------------

impl<X: Semiring> One for M22<X>
where X: PrimitiveSemiring,
{
    #[inline(always)]
    fn one() -> Self
    { M22::from_diag(V2([X::one(), X::one()])) }

    #[inline]
    fn is_one(&self) -> bool
    { is_one_impl(self) }
}

impl<X: Semiring> One for M33<X>
where X: PrimitiveSemiring,
{
    #[inline(always)]
    fn one() -> Self
    { M33::from_diag(V3([X::one(), X::one(), X::one()])) }

    #[inline]
    fn is_one(&self) -> bool
    { is_one_impl(self) }
}

impl<X: Semiring> One for M44<X>
where X: PrimitiveSemiring,
{
    #[inline(always)]
    fn one() -> Self
    { M44::from_diag(V4([X::one(), X::one(), X::one(), X::one()])) }

    #[inline]
    fn is_one(&self) -> bool
    { is_one_impl(self) }
}

// NOTE: I have no idea how well this optimizes.
#[inline(always)]
fn is_one_impl<M, V, X>(m: &M) -> bool
where
    for<'a> &'a M: IntoIterator<Item=&'a V>,
    for<'a> &'a V: IntoIterator<Item=&'a X>,
    X: Zero + One,
    X: PartialEq, // (can be removed when num_traits gets a semver bump)
{
    m.into_iter().enumerate().all(|(r, v)| {
        v.into_iter().enumerate().all(|(c, x)| {
            test_eye_element(r, c, x)
        })
    })
}

#[inline(always)]
fn test_eye_element<X>(r: usize, c: usize, x: &X) -> bool
where
    X: Zero + One,
    X: PartialEq, // (can be removed when num_traits gets a semver bump)
{
    match r == c {
        true => x.is_one(),
        false => x.is_zero(),
    }
}

// ---------------------------------------------------------------------------

/// Output of `det`. Probably a scalar type.
pub type DetT<A> = <A as Det>::Output;

/// Implementation detail of the inherent method `{M22,M33}::det`.
///
/// > **_Fuggedaboudit._**
pub trait Det {
    type Output;

    fn det(&self) -> Self::Output;
}

impl<T: Ring> Det for M22<T>
where T: PrimitiveRing,
{
    type Output = T;

    fn det(&self) -> T {
        let [[a, b], [c, d]] = self.unvee();
        a * d - b * c
    }
}

impl<T: Ring> Det for M33<T>
where T: PrimitiveRing,
{
    type Output = T;

    fn det(&self) -> T {
        let [
            [a0, a1, a2],
            [b0, b1, b2],
            [c0, c1, c2],
        ] = self.unvee();

        T::zero()
        + a0 * b1 * c2
        + a1 * b2 * c0
        + a2 * b0 * c1
        - a0 * b2 * c1
        - a1 * b0 * c2
        - a2 * b1 * c0
    }
}

// ---------------------------------------------------------------------------

/// Implementation detail of the inherent method `{M22,M33}::inv`.
///
/// > **_Fuggedaboudit._**
pub trait Inv {
    fn inv(&self) -> Self;
}

impl<T: Field> Inv for M22<T>
where T: PrimitiveFloat,
{
    fn inv(&self) -> Self {
        let [[a, b], [c, d]] = self.0.unvee();
        let rdet = T::one() / self.det();
        &[[d, -b], [-c, a]].into_matrix() * rdet
    }
}

impl<T: Field> Inv for M33<T>
where T: PrimitiveFloat,
{
    fn inv(&self) -> Self {
        let cofactors: M33<T> = from_fn(|r, c|
            T::zero()
            + self[(r+1) % 3][(c+1) % 3] * self[(r+2) % 3][(c+2) % 3]
            - self[(r+1) % 3][(c+2) % 3] * self[(r+2) % 3][(c+1) % 3]
        );
        let det = vee::dot(&self[0], &cofactors[0]);
        let rdet = T::one() / det;
        M33::from_fn(|r, c| rdet * cofactors[c][r])
    }
}

// ---------------------------------------------------------------------------

/// Output of `transpose`. Probably a matrix with the dimensions flipped.
pub type TransposeT<A> = <A as Transpose>::Output;

/// Implementation detail of the inherent method `{M2,M3,M4}::t`.
///
/// > **_Fuggedaboudit._**
pub trait Transpose {
    type Output;

    fn t(&self) -> Self::Output;
}

gen_each!{
    @{Mn_n}
    @{Vn_n}
    for_each!(
        {$Mr:ident $r:tt}
        {$Vc:ident $c:tt}
    ) => {
        // FIXME: unnecessary Copy bound
        impl<X: Copy> Transpose for $Mr<$Vc<X>> {
            type Output = M![$c, V![$r, X]];

            #[inline]
            fn t(&self) -> Self::Output
            { from_fn(|r, c| self[c][r]) }
        }
    }
}

// ---------------------------------------------------------------------------

// slice-of-array integration.

gen_each!{
    @{Mn_n}
    for_each!( {$Mn:ident $n:tt} ) => {
        unsafe impl<V> slice_of_array::IsSliceomorphic for $Mn<V> {
            type Element = V;
            const LEN: usize = $n;
        }
    }
}

// ---------------------------------------------------------------------------

/// Argument to `from_diag`. Probably a vector type like `V3<X>`.
pub type DiagT<A> = <A as FromDiag>::Diag;

/// Implementation detail of the inherent method `{M2,M3,M4}::from_diag`.
///
/// > **_Fuggedaboudit._**
pub trait FromDiag: Sized {
    type Diag;

    fn from_diag(diag: Self::Diag) -> Self;
}

impl<X: Semiring> FromDiag for M22<X>
where X: PrimitiveSemiring,
{
    type Diag = V2<X>;

    #[inline(always)]
    fn from_diag(V2([a, b]): V2<X>) -> Self { M2([
        V2([        a, X::zero()]),
        V2([X::zero(),         b]),
    ])}
}

impl<X: Semiring> FromDiag for M33<X>
where X: PrimitiveSemiring,
{
    type Diag = V3<X>;

    #[inline(always)]
    fn from_diag(V3([a, b, c]): V3<X>) -> Self { M3([
        V3([        a, X::zero(), X::zero()]),
        V3([X::zero(),         b, X::zero()]),
        V3([X::zero(), X::zero(),         c]),
    ])}
}

impl<X: Semiring> FromDiag for M44<X>
where X: PrimitiveSemiring,
{
    type Diag = V4<X>;

    #[inline(always)]
    fn from_diag(V4([a, b, c, d]): V4<X>) -> Self { M4([
        V4([        a, X::zero(), X::zero(), X::zero()]),
        V4([X::zero(),         b, X::zero(), X::zero()]),
        V4([X::zero(), X::zero(),         c, X::zero()]),
        V4([X::zero(), X::zero(), X::zero(),         d]),
    ])}
}

// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_eye() {
        assert_eq!(from_array([[0, 0], [0, 0]]), M22::zero());
        assert_eq!(from_array([[1, 0], [0, 1]]), M22::eye());
        assert_eq!(from_array([[0, 0, 0], [0, 0, 0], [0, 0, 0]]), M33::zero());
        assert_eq!(from_array([[1, 0, 0], [0, 1, 0], [0, 0, 1]]), M33::eye());
        assert_eq!(from_array([[0, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0]]), M44::zero());
        assert_eq!(from_array([[1, 0, 0, 0], [0, 1, 0, 0], [0, 0, 1, 0], [0, 0, 0, 1]]), M44::eye());
        assert!(from_array([[0, 0], [0, 0]]).is_zero());
        assert!(!from_array([[0, 1], [0, 0]]).is_zero());
        assert!(from_array([[1, 0], [0, 1]]).is_one());
        assert!(!from_array([[2, 0], [0, 1]]).is_one());
        assert!(!from_array([[1, -1], [0, 1]]).is_one());
    }

    #[test]
    fn test_inverse_2() {
        let actual = inv(&from_array([[7., 2.], [-11., 4.]]));
        let expected = from_array([
            [ 2./25., -1./25.],
            [11./50.,  7./50.],
        ]);

        assert_close!(abs=1e-12, expected.unvee(), actual.unvee());
    }

    #[test]
    fn test_inverse_3() {
        let actual = inv(&from_array([
            [1., 2., 4.],
            [5., 2., 1.],
            [3., 6., 3.],
        ]));

        let expected = from_array([
            [ 0./1.,  1./4., -1./12.],
            [-1./6., -1./8., 19./72.],
            [ 1./3.,  0./1., -1./9. ],
        ]);

        assert_close!(abs=1e-12, expected.unvee(), actual.unvee());
    }
}
