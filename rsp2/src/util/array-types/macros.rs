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

/// Higher-order macro that iterates over a cartesian product.
///
/// Useful for generating impls involving opaque traits of the sort
/// sometimes seen used as bounds in public APIs.
///
/// It takes a number of groups of token trees and a suitable definition
/// for a callback macro, and it calls the macro with one token tree from
/// each group in order.
///
/// See the examples module in the source for example usage.
macro_rules! cartesian {
    (
        $([$($groups:tt)*])*
        for_each!($($mac_match:tt)*)
        => {$($mac_body:tt)*}$(;)*
    )
    => {
        // (this fixed name gets overwritten on each use.  Technically, using a fixed name
        //  makes this macro not "re-entrant", in the sense that you can't call it in a nested
        //  fashion... but there is no reason to do so, because the macro has a monolithic design)
        macro_rules! __cartesian__user_macro {
            ($($mac_match)*) => {$($mac_body)*};
        }
        cartesian__!{ @product::next($([$($groups)*])*) -> (__cartesian__user_macro!()) }
    };
}

/// implementation detail, go away
macro_rules! cartesian__ {

    (@product::next([$($token:tt)+] $($rest:tt)*) -> $cb:tt)
    => { cartesian__!{ @product::unpack([$($token)+] $($rest)*) -> $cb } };
    // base case; direct product of no arguments
    (@product::next() -> ($mac:ident!($($args:tt)*)))
    => {$mac!{$($args)*}};

    // Each direct product in the invocation incurs a fixed number of recursions
    //  as we replicate the macro.  First, we must smash anything we want to replicate
    //  into a single tt that can be matched without repetitions.  Do this to `rest`.
    (@product::unpack([$($token:tt)*] $($rest:tt)*) -> $cb:tt)
    => {cartesian__!{ @product::unpack_2([$($token)*] [$($rest)*]) -> $cb }};

    // Replicate macro for each token.
    (@product::unpack_2([$($token:tt)*] $rest:tt) -> $cb:tt)
    => { $( cartesian__!{ @product::unpack_3($token $rest) -> $cb } )* };

    // Expand the unparsed arguments back to normal;
    // add the token into the macro call
    (@product::unpack_3($token:tt [$($rest:tt)*]) -> ($mac:ident!($($args:tt)*)))
    => {cartesian__!{ @product::next($($rest)*) -> ($mac!($($args)*$token)) }};
}

/// `cartesian!` with some predefined groups used in the library's public API.
///
/// See the examples module in the source for example usage.
macro_rules! gen_each {
    ($($arg:tt)*) => { gen_each__!{[$($arg)*] -> []} };
}

macro_rules! gen_each__ {
    //----------------------------
    // Groups using the standard syntax supported by cartesian

    ([[$($alternatives:tt)*] $($rest:tt)*] -> [$($done:tt)*])
    => { gen_each__!{[$($rest)*] -> [$($done)* [
        $($alternatives)*
    ]] }};

    //----------------------------
    // Special groups of the form @{...}

    // NOTE: This macro is what truly defines the set of members
    //       for each opaque trait.

    // Types that implement Field
    ([@{field} $($rest:tt)*] -> [$($done:tt)*])
    => { gen_each__!{[$($rest)*] -> [$($done)* [
        {f32} {f64}
    ]] }};

    // Types that implement Ring
    ([@{ring} $($rest:tt)*] -> [$($done:tt)*])
    => { gen_each__!{[$($rest)*] -> [$($done)* [
        {f32} {f64}
        {i8} {i16} {i32} {i64} {isize}
    ]] }};

    // Types that implement Semiring
    ([@{semiring} $($rest:tt)*] -> [$($done:tt)*])
    => { gen_each__!{[$($rest)*] -> [$($done)* [
        {f32} {f64}
        {i8} {i16} {i32} {i64} {isize}
        {u8} {u16} {u32} {u64} {usize}
    ]] }};

    // PrimitiveFloat types
    ([@{float} $($rest:tt)*] -> [$($done:tt)*])
    => { gen_each__!{[$($rest)*] -> [$($done)* [
        {f32} {f64}
    ]] }};

    // Fixed sized vector types
    ([@{Vn} $($rest:tt)*] -> [$($done:tt)*])
    => { gen_each__!{[$($rest)*] -> [$($done)* [
        {V2} {V3} {V4}
    ]] }};

    // ...along with their size
    ([@{Vn_n} $($rest:tt)*] -> [$($done:tt)*])
    => { gen_each__!{[$($rest)*] -> [$($done)* [
        {V2 2} {V3 3} {V4 4}
    ]] }};

    // Matrices of generic width, and their number of rows
    ([@{Mn_n} $($rest:tt)*] -> [$($done:tt)*])
    => { gen_each__!{[$($rest)*] -> [$($done)* [
        {M2 2} {M3 3} {M4 4}
    ]] }};

    // Square matrices, their vector types, and size
    ([@{Mnn_Mn_Vn_n} $($rest:tt)*] -> [$($done:tt)*])
    => { gen_each__!{[$($rest)*] -> [$($done)* [
        {M22 M2 V2 2} {M33 M3 V3 3} {M44 M4 V4 4}
    ]] }};

    ([@{0...4} $($rest:tt)*] -> [$($done:tt)*])
    => { gen_each__!{[$($rest)*] -> [$($done)* [
        { 0} { 1} { 2} { 3} { 4}
    ]] }};

    ([@{1...4} $($rest:tt)*] -> [$($done:tt)*])
    => { gen_each__!{[$($rest)*] -> [$($done)* [
             { 1} { 2} { 3} { 4}
    ]] }};

    ([@{0...8} $($rest:tt)*] -> [$($done:tt)*])
    => { gen_each__!{[$($rest)*] -> [$($done)* [
        { 0} { 1} { 2} { 3} { 4} { 5} { 6} { 7} { 8}
    ]] }};

    ([@{1...8} $($rest:tt)*] -> [$($done:tt)*])
    => { gen_each__!{[$($rest)*] -> [$($done)* [
             { 1} { 2} { 3} { 4} { 5} { 6} { 7} { 8}
    ]] }};

    ([@{0...16} $($rest:tt)*] -> [$($done:tt)*])
    => { gen_each__!{[$($rest)*] -> [$($done)* [
        { 0} { 1} { 2} { 3} { 4} { 5} { 6} { 7} { 8} { 9}
        {10} {11} {12} {13} {14} {15} {16}
    ]] }};

    ([@{1...16} $($rest:tt)*] -> [$($done:tt)*])
    => { gen_each__!{[$($rest)*] -> [$($done)* [
             { 1} { 2} { 3} { 4} { 5} { 6} { 7} { 8} { 9}
        {10} {11} {12} {13} {14} {15} {16}
    ]] }};

    // Finally: Delegate to `cartesian`
    ([$mac:ident!$($defn_args:tt)*] -> [$($groups:tt)*])
    => {
        cartesian!{
            $($groups)*
            $mac!$($defn_args)*
        }
    };
}

/// Synthesize a `Vn` vector type from a `tt` of the size.
macro_rules! V {
    (2, $X:ty) => { V2<$X> };
    (3, $X:ty) => { V3<$X> };
    (4, $X:ty) => { V4<$X> };
}

/// Synthesize an `Mn` matrix type from a `tt` of the size.
macro_rules! M {
    (2, $V:ty) => { M2<$V> };
    (3, $V:ty) => { M3<$V> };
    (4, $V:ty) => { M4<$V> };
}

#[cfg(test)]
mod examples {
    mod cartesian {
        trait Trait { }
        // NOTE: Braces around the alternatives are not strictly necessary
        //       (cartesian simply iterates over token trees), but they tend
        //       to help group tokens and resolve any would-be ambiguities in
        //       the callback's match pattern.
        cartesian!{
            [{i32} {u32}]
            [{0} {1} {2} {3}]
            for_each!({$T:ty} {$n:expr})
            => {
                impl Trait for [$T; $n] { }
            }
        }

        #[test]
        fn example_works() {
            fn assert_trait<T:Trait>() {}
            assert_trait::<[u32; 0]>();
            assert_trait::<[i32; 2]>();
        }
    }

    mod gen_each {
        trait Trait { }
        gen_each!{
            @{field}      // equivalent to [{f32} {f64}]
            @{0...4}      // equivalent to [{0} {1} {2} {3} {4}]
            [{i32} {u32}] // explicitly defined groups are still allowed
            for_each!({$A:ty} {$n:expr} {$B:ty})
            => {
                impl Trait for ([$A; $n], $B) { }
            }
        }

        #[test]
        fn example_works() {
            fn assert_trait<T:Trait>() {}
            assert_trait::<([f64; 3], i32)>();
        }
    }
}
