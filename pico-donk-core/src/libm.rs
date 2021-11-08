// This file is a copy of a lot of code from the libm crate.
// However, it has all been edited to be const.

macro_rules! i {
    ($array:expr, $index:expr) => {
        $array[$index]
    };
    ($array:expr, $index:expr, = , $rhs:expr) => {
        $array[$index] = $rhs;
    };
    ($array:expr, $index:expr, -= , $rhs:expr) => {
        $array[$index] -= $rhs;
    };
    ($array:expr, $index:expr, += , $rhs:expr) => {
        $array[$index] += $rhs;
    };
    ($array:expr, $index:expr, &= , $rhs:expr) => {
        $array[$index] &= $rhs;
    };
    ($array:expr, $index:expr, == , $rhs:expr) => {
        $array[$index] == $rhs
    };
}

// Temporary macro to avoid panic codegen for division (in debug mode too). At
// the time of this writing this is only used in a few places, and once
// rust-lang/rust#72751 is fixed then this macro will no longer be necessary and
// the native `/` operator can be used and panics won't be codegen'd.
#[cfg(any(debug_assertions, not(feature = "unstable")))]
macro_rules! div {
    ($a:expr, $b:expr) => {
        $a / $b
    };
}

#[cfg(all(not(debug_assertions), feature = "unstable"))]
macro_rules! div {
    ($a:expr, $b:expr) => {
        unsafe { core::intrinsics::unchecked_div($a, $b) }
    };
}

macro_rules! llvm_intrinsically_optimized {
    (#[cfg($($clause:tt)*)] $e:expr) => {
        #[cfg(all(feature = "unstable", $($clause)*))]
        {
            if true { // thwart the dead code lint
                $e
            }
        }
    };
}

const S1: f64 = -1.66666666666666324348e-01; /* 0xBFC55555, 0x55555549 */
const S2: f64 = 8.33333333332248946124e-03; /* 0x3F811111, 0x1110F8A6 */
const S3: f64 = -1.98412698298579493134e-04; /* 0xBF2A01A0, 0x19C161D5 */
const S4: f64 = 2.75573137070700676789e-06; /* 0x3EC71DE3, 0x57B1FE7D */
const S5: f64 = -2.50507602534068634195e-08; /* 0xBE5AE5E6, 0x8A2B9CEB */
const S6: f64 = 1.58969099521155010221e-10; /* 0x3DE5D93A, 0x5ACFD57C */

// kernel sin function on ~[-pi/4, pi/4] (except on -0), pi/4 ~ 0.7854
// Input x is assumed to be bounded by ~pi/4 in magnitude.
// Input y is the tail of x.
// Input iy indicates whether y is 0. (if iy=0, y assume to be 0).
//
// Algorithm
//      1. Since sin(-x) = -sin(x), we need only to consider positive x.
//      2. Callers must return sin(-0) = -0 without calling here since our
//         odd polynomial is not evaluated in a way that preserves -0.
//         Callers may do the optimization sin(x) ~ x for tiny x.
//      3. sin(x) is approximated by a polynomial of degree 13 on
//         [0,pi/4]
//                               3            13
//              sin(x) ~ x + S1*x + ... + S6*x
//         where
//
//      |sin(x)         2     4     6     8     10     12  |     -58
//      |----- - (1+S1*x +S2*x +S3*x +S4*x +S5*x  +S6*x   )| <= 2
//      |  x                                               |
//
//      4. sin(x+y) = sin(x) + sin'(x')*y
//                  ~ sin(x) + (1-x*x/2)*y
//         For better accuracy, let
//                   3      2      2      2      2
//              r = x *(S2+x *(S3+x *(S4+x *(S5+x *S6))))
//         then                   3    2
//              sin(x) = x + (S1*x + (x *(r-y/2)+y))
#[cfg_attr(all(test, assert_no_panic), no_panic::no_panic)]
pub const fn k_sin(x: f64, y: f64, iy: i32) -> f64 {
    let z = x * x;
    let w = z * z;
    let r = S2 + z * (S3 + z * S4) + z * w * (S5 + z * S6);
    let v = z * x;
    if iy == 0 {
        x + v * (S1 + z * r)
    } else {
        x - ((z * (0.5 * y - v * r) - y) - v * S1)
    }
}
// origin: FreeBSD /usr/src/lib/msun/src/k_cos.c
//
// ====================================================
// Copyright (C) 1993 by Sun Microsystems, Inc. All rights reserved.
//
// Developed at SunSoft, a Sun Microsystems, Inc. business.
// Permission to use, copy, modify, and distribute this
// software is freely granted, provided that this notice
// is preserved.
// ====================================================

const C1: f64 = 4.16666666666666019037e-02; /* 0x3FA55555, 0x5555554C */
const C2: f64 = -1.38888888888741095749e-03; /* 0xBF56C16C, 0x16C15177 */
const C3: f64 = 2.48015872894767294178e-05; /* 0x3EFA01A0, 0x19CB1590 */
const C4: f64 = -2.75573143513906633035e-07; /* 0xBE927E4F, 0x809C52AD */
const C5: f64 = 2.08757232129817482790e-09; /* 0x3E21EE9E, 0xBDB4B1C4 */
const C6: f64 = -1.13596475577881948265e-11; /* 0xBDA8FAE9, 0xBE8838D4 */

// kernel cos function on [-pi/4, pi/4], pi/4 ~ 0.785398164
// Input x is assumed to be bounded by ~pi/4 in magnitude.
// Input y is the tail of x.
//
// Algorithm
//      1. Since cos(-x) = cos(x), we need only to consider positive x.
//      2. if x < 2^-27 (hx<0x3e400000 0), return 1 with inexact if x!=0.
//      3. cos(x) is approximated by a polynomial of degree 14 on
//         [0,pi/4]
//                                       4            14
//              cos(x) ~ 1 - x*x/2 + C1*x + ... + C6*x
//         where the remez error is
//
//      |              2     4     6     8     10    12     14 |     -58
//      |cos(x)-(1-.5*x +C1*x +C2*x +C3*x +C4*x +C5*x  +C6*x  )| <= 2
//      |                                                      |
//
//                     4     6     8     10    12     14
//      4. let r = C1*x +C2*x +C3*x +C4*x +C5*x  +C6*x  , then
//             cos(x) ~ 1 - x*x/2 + r
//         since cos(x+y) ~ cos(x) - sin(x)*y
//                        ~ cos(x) - x*y,
//         a correction term is necessary in cos(x) and hence
//              cos(x+y) = 1 - (x*x/2 - (r - x*y))
//         For better accuracy, rearrange to
//              cos(x+y) ~ w + (tmp + (r-x*y))
//         where w = 1 - x*x/2 and tmp is a tiny correction term
//         (1 - x*x/2 == w + tmp exactly in infinite precision).
//         The exactness of w + tmp in infinite precision depends on w
//         and tmp having the same precision as x.  If they have extra
//         precision due to compiler bugs, then the extra precision is
//         only good provided it is retained in all terms of the final
//         expression for cos().  Retention happens in all cases tested
//         under FreeBSD, so don't pessimize things by forcibly clipping
//         any extra precision in w.
#[cfg_attr(all(test, assert_no_panic), no_panic::no_panic)]
pub const fn k_cos(x: f64, y: f64) -> f64 {
    let z = x * x;
    let w = z * z;
    let r = z * (C1 + z * (C2 + z * C3)) + w * w * (C4 + z * (C5 + z * C6));
    let hz = 0.5 * z;
    let w = 1.0 - hz;
    w + (((1.0 - w) - hz) + (z * r - x * y))
}

/* origin: FreeBSD /usr/src/lib/msun/src/k_rem_pio2.c */
/*
 * ====================================================
 * Copyright (C) 1993 by Sun Microsystems, Inc. All rights reserved.
 *
 * Developed at SunSoft, a Sun Microsystems, Inc. business.
 * Permission to use, copy, modify, and distribute this
 * software is freely granted, provided that this notice
 * is preserved.
 * ====================================================
 */

use core::f64;

const TOINT: f64 = 1. / f64::EPSILON;

/// Floor (f64)
///
/// Finds the nearest integer less than or equal to `x`.
#[cfg_attr(all(test, assert_no_panic), no_panic::no_panic)]
pub const fn floor(x: f64) -> f64 {
    // On wasm32 we know that LLVM's intrinsic will compile to an optimized
    // `f64.floor` native instruction, so we can leverage this for both code size
    // and speed.
    llvm_intrinsically_optimized! {
        #[cfg(target_arch = "wasm32")] {
            return unsafe { ::core::intrinsics::floorf64(x) }
        }
    }
    let ui = x.to_bits();
    let e = ((ui >> 52) & 0x7ff) as i32;

    if (e >= 0x3ff + 52) || (x == 0.) {
        return x;
    }
    /* y = int(x) - x, where int(x) is an integer neighbor of x */
    let y = if (ui >> 63) != 0 {
        x - TOINT + TOINT - x
    } else {
        x + TOINT - TOINT - x
    };
    /* special case because of non-nearest rounding modes */
    if e < 0x3ff {
        return if (ui >> 63) != 0 { -1. } else { 0. };
    }
    if y > 0. {
        x + y - 1.
    } else {
        x + y
    }
}

#[cfg_attr(all(test, assert_no_panic), no_panic::no_panic)]
pub const fn scalbn(x: f64, mut n: i32) -> f64 {
    let x1p1023 = f64::from_bits(0x7fe0000000000000); // 0x1p1023 === 2 ^ 1023
    let x1p53 = f64::from_bits(0x4340000000000000); // 0x1p53 === 2 ^ 53
    let x1p_1022 = f64::from_bits(0x0010000000000000); // 0x1p-1022 === 2 ^ (-1022)

    let mut y = x;

    if n > 1023 {
        y *= x1p1023;
        n -= 1023;
        if n > 1023 {
            y *= x1p1023;
            n -= 1023;
            if n > 1023 {
                n = 1023;
            }
        }
    } else if n < -1022 {
        /* make sure final n < -53 to avoid double
        rounding in the subnormal range */
        y *= x1p_1022 * x1p53;
        n += 1022 - 53;
        if n < -1022 {
            y *= x1p_1022 * x1p53;
            n += 1022 - 53;
            if n < -1022 {
                n = -1022;
            }
        }
    }
    y * f64::from_bits(((0x3ff + n) as u64) << 52)
}

// initial value for jk
const INIT_JK: [usize; 4] = [3, 4, 4, 6];

// Table of constants for 2/pi, 396 Hex digits (476 decimal) of 2/pi
//
//              integer array, contains the (24*i)-th to (24*i+23)-th
//              bit of 2/pi after binary point. The corresponding
//              floating value is
//
//                      ipio2[i] * 2^(-24(i+1)).
//
// NB: This table must have at least (e0-3)/24 + jk terms.
//     For quad precision (e0 <= 16360, jk = 6), this is 686.
#[cfg(target_pointer_width = "32")]
const IPIO2: [i32; 66] = [
    0xA2F983, 0x6E4E44, 0x1529FC, 0x2757D1, 0xF534DD, 0xC0DB62, 0x95993C, 0x439041, 0xFE5163,
    0xABDEBB, 0xC561B7, 0x246E3A, 0x424DD2, 0xE00649, 0x2EEA09, 0xD1921C, 0xFE1DEB, 0x1CB129,
    0xA73EE8, 0x8235F5, 0x2EBB44, 0x84E99C, 0x7026B4, 0x5F7E41, 0x3991D6, 0x398353, 0x39F49C,
    0x845F8B, 0xBDF928, 0x3B1FF8, 0x97FFDE, 0x05980F, 0xEF2F11, 0x8B5A0A, 0x6D1F6D, 0x367ECF,
    0x27CB09, 0xB74F46, 0x3F669E, 0x5FEA2D, 0x7527BA, 0xC7EBE5, 0xF17B3D, 0x0739F7, 0x8A5292,
    0xEA6BFB, 0x5FB11F, 0x8D5D08, 0x560330, 0x46FC7B, 0x6BABF0, 0xCFBC20, 0x9AF436, 0x1DA9E3,
    0x91615E, 0xE61B08, 0x659985, 0x5F14A0, 0x68408D, 0xFFD880, 0x4D7327, 0x310606, 0x1556CA,
    0x73A8C9, 0x60E27B, 0xC08C6B,
];

#[cfg(target_pointer_width = "64")]
const IPIO2: [i32; 690] = [
    0xA2F983, 0x6E4E44, 0x1529FC, 0x2757D1, 0xF534DD, 0xC0DB62, 0x95993C, 0x439041, 0xFE5163,
    0xABDEBB, 0xC561B7, 0x246E3A, 0x424DD2, 0xE00649, 0x2EEA09, 0xD1921C, 0xFE1DEB, 0x1CB129,
    0xA73EE8, 0x8235F5, 0x2EBB44, 0x84E99C, 0x7026B4, 0x5F7E41, 0x3991D6, 0x398353, 0x39F49C,
    0x845F8B, 0xBDF928, 0x3B1FF8, 0x97FFDE, 0x05980F, 0xEF2F11, 0x8B5A0A, 0x6D1F6D, 0x367ECF,
    0x27CB09, 0xB74F46, 0x3F669E, 0x5FEA2D, 0x7527BA, 0xC7EBE5, 0xF17B3D, 0x0739F7, 0x8A5292,
    0xEA6BFB, 0x5FB11F, 0x8D5D08, 0x560330, 0x46FC7B, 0x6BABF0, 0xCFBC20, 0x9AF436, 0x1DA9E3,
    0x91615E, 0xE61B08, 0x659985, 0x5F14A0, 0x68408D, 0xFFD880, 0x4D7327, 0x310606, 0x1556CA,
    0x73A8C9, 0x60E27B, 0xC08C6B, 0x47C419, 0xC367CD, 0xDCE809, 0x2A8359, 0xC4768B, 0x961CA6,
    0xDDAF44, 0xD15719, 0x053EA5, 0xFF0705, 0x3F7E33, 0xE832C2, 0xDE4F98, 0x327DBB, 0xC33D26,
    0xEF6B1E, 0x5EF89F, 0x3A1F35, 0xCAF27F, 0x1D87F1, 0x21907C, 0x7C246A, 0xFA6ED5, 0x772D30,
    0x433B15, 0xC614B5, 0x9D19C3, 0xC2C4AD, 0x414D2C, 0x5D000C, 0x467D86, 0x2D71E3, 0x9AC69B,
    0x006233, 0x7CD2B4, 0x97A7B4, 0xD55537, 0xF63ED7, 0x1810A3, 0xFC764D, 0x2A9D64, 0xABD770,
    0xF87C63, 0x57B07A, 0xE71517, 0x5649C0, 0xD9D63B, 0x3884A7, 0xCB2324, 0x778AD6, 0x23545A,
    0xB91F00, 0x1B0AF1, 0xDFCE19, 0xFF319F, 0x6A1E66, 0x615799, 0x47FBAC, 0xD87F7E, 0xB76522,
    0x89E832, 0x60BFE6, 0xCDC4EF, 0x09366C, 0xD43F5D, 0xD7DE16, 0xDE3B58, 0x929BDE, 0x2822D2,
    0xE88628, 0x4D58E2, 0x32CAC6, 0x16E308, 0xCB7DE0, 0x50C017, 0xA71DF3, 0x5BE018, 0x34132E,
    0x621283, 0x014883, 0x5B8EF5, 0x7FB0AD, 0xF2E91E, 0x434A48, 0xD36710, 0xD8DDAA, 0x425FAE,
    0xCE616A, 0xA4280A, 0xB499D3, 0xF2A606, 0x7F775C, 0x83C2A3, 0x883C61, 0x78738A, 0x5A8CAF,
    0xBDD76F, 0x63A62D, 0xCBBFF4, 0xEF818D, 0x67C126, 0x45CA55, 0x36D9CA, 0xD2A828, 0x8D61C2,
    0x77C912, 0x142604, 0x9B4612, 0xC459C4, 0x44C5C8, 0x91B24D, 0xF31700, 0xAD43D4, 0xE54929,
    0x10D5FD, 0xFCBE00, 0xCC941E, 0xEECE70, 0xF53E13, 0x80F1EC, 0xC3E7B3, 0x28F8C7, 0x940593,
    0x3E71C1, 0xB3092E, 0xF3450B, 0x9C1288, 0x7B20AB, 0x9FB52E, 0xC29247, 0x2F327B, 0x6D550C,
    0x90A772, 0x1FE76B, 0x96CB31, 0x4A1679, 0xE27941, 0x89DFF4, 0x9794E8, 0x84E6E2, 0x973199,
    0x6BED88, 0x365F5F, 0x0EFDBB, 0xB49A48, 0x6CA467, 0x427271, 0x325D8D, 0xB8159F, 0x09E5BC,
    0x25318D, 0x3974F7, 0x1C0530, 0x010C0D, 0x68084B, 0x58EE2C, 0x90AA47, 0x02E774, 0x24D6BD,
    0xA67DF7, 0x72486E, 0xEF169F, 0xA6948E, 0xF691B4, 0x5153D1, 0xF20ACF, 0x339820, 0x7E4BF5,
    0x6863B2, 0x5F3EDD, 0x035D40, 0x7F8985, 0x295255, 0xC06437, 0x10D86D, 0x324832, 0x754C5B,
    0xD4714E, 0x6E5445, 0xC1090B, 0x69F52A, 0xD56614, 0x9D0727, 0x50045D, 0xDB3BB4, 0xC576EA,
    0x17F987, 0x7D6B49, 0xBA271D, 0x296996, 0xACCCC6, 0x5414AD, 0x6AE290, 0x89D988, 0x50722C,
    0xBEA404, 0x940777, 0x7030F3, 0x27FC00, 0xA871EA, 0x49C266, 0x3DE064, 0x83DD97, 0x973FA3,
    0xFD9443, 0x8C860D, 0xDE4131, 0x9D3992, 0x8C70DD, 0xE7B717, 0x3BDF08, 0x2B3715, 0xA0805C,
    0x93805A, 0x921110, 0xD8E80F, 0xAF806C, 0x4BFFDB, 0x0F9038, 0x761859, 0x15A562, 0xBBCB61,
    0xB989C7, 0xBD4010, 0x04F2D2, 0x277549, 0xF6B6EB, 0xBB22DB, 0xAA140A, 0x2F2689, 0x768364,
    0x333B09, 0x1A940E, 0xAA3A51, 0xC2A31D, 0xAEEDAF, 0x12265C, 0x4DC26D, 0x9C7A2D, 0x9756C0,
    0x833F03, 0xF6F009, 0x8C402B, 0x99316D, 0x07B439, 0x15200C, 0x5BC3D8, 0xC492F5, 0x4BADC6,
    0xA5CA4E, 0xCD37A7, 0x36A9E6, 0x9492AB, 0x6842DD, 0xDE6319, 0xEF8C76, 0x528B68, 0x37DBFC,
    0xABA1AE, 0x3115DF, 0xA1AE00, 0xDAFB0C, 0x664D64, 0xB705ED, 0x306529, 0xBF5657, 0x3AFF47,
    0xB9F96A, 0xF3BE75, 0xDF9328, 0x3080AB, 0xF68C66, 0x15CB04, 0x0622FA, 0x1DE4D9, 0xA4B33D,
    0x8F1B57, 0x09CD36, 0xE9424E, 0xA4BE13, 0xB52333, 0x1AAAF0, 0xA8654F, 0xA5C1D2, 0x0F3F0B,
    0xCD785B, 0x76F923, 0x048B7B, 0x721789, 0x53A6C6, 0xE26E6F, 0x00EBEF, 0x584A9B, 0xB7DAC4,
    0xBA66AA, 0xCFCF76, 0x1D02D1, 0x2DF1B1, 0xC1998C, 0x77ADC3, 0xDA4886, 0xA05DF7, 0xF480C6,
    0x2FF0AC, 0x9AECDD, 0xBC5C3F, 0x6DDED0, 0x1FC790, 0xB6DB2A, 0x3A25A3, 0x9AAF00, 0x9353AD,
    0x0457B6, 0xB42D29, 0x7E804B, 0xA707DA, 0x0EAA76, 0xA1597B, 0x2A1216, 0x2DB7DC, 0xFDE5FA,
    0xFEDB89, 0xFDBE89, 0x6C76E4, 0xFCA906, 0x70803E, 0x156E85, 0xFF87FD, 0x073E28, 0x336761,
    0x86182A, 0xEABD4D, 0xAFE7B3, 0x6E6D8F, 0x396795, 0x5BBF31, 0x48D784, 0x16DF30, 0x432DC7,
    0x356125, 0xCE70C9, 0xB8CB30, 0xFD6CBF, 0xA200A4, 0xE46C05, 0xA0DD5A, 0x476F21, 0xD21262,
    0x845CB9, 0x496170, 0xE0566B, 0x015299, 0x375550, 0xB7D51E, 0xC4F133, 0x5F6E13, 0xE4305D,
    0xA92E85, 0xC3B21D, 0x3632A1, 0xA4B708, 0xD4B1EA, 0x21F716, 0xE4698F, 0x77FF27, 0x80030C,
    0x2D408D, 0xA0CD4F, 0x99A520, 0xD3A2B3, 0x0A5D2F, 0x42F9B4, 0xCBDA11, 0xD0BE7D, 0xC1DB9B,
    0xBD17AB, 0x81A2CA, 0x5C6A08, 0x17552E, 0x550027, 0xF0147F, 0x8607E1, 0x640B14, 0x8D4196,
    0xDEBE87, 0x2AFDDA, 0xB6256B, 0x34897B, 0xFEF305, 0x9EBFB9, 0x4F6A68, 0xA82A4A, 0x5AC44F,
    0xBCF82D, 0x985AD7, 0x95C7F4, 0x8D4D0D, 0xA63A20, 0x5F57A4, 0xB13F14, 0x953880, 0x0120CC,
    0x86DD71, 0xB6DEC9, 0xF560BF, 0x11654D, 0x6B0701, 0xACB08C, 0xD0C0B2, 0x485551, 0x0EFB1E,
    0xC37295, 0x3B06A3, 0x3540C0, 0x7BDC06, 0xCC45E0, 0xFA294E, 0xC8CAD6, 0x41F3E8, 0xDE647C,
    0xD8649B, 0x31BED9, 0xC397A4, 0xD45877, 0xC5E369, 0x13DAF0, 0x3C3ABA, 0x461846, 0x5F7555,
    0xF5BDD2, 0xC6926E, 0x5D2EAC, 0xED440E, 0x423E1C, 0x87C461, 0xE9FD29, 0xF3D6E7, 0xCA7C22,
    0x35916F, 0xC5E008, 0x8DD7FF, 0xE26A6E, 0xC6FDB0, 0xC10893, 0x745D7C, 0xB2AD6B, 0x9D6ECD,
    0x7B723E, 0x6A11C6, 0xA9CFF7, 0xDF7329, 0xBAC9B5, 0x5100B7, 0x0DB2E2, 0x24BA74, 0x607DE5,
    0x8AD874, 0x2C150D, 0x0C1881, 0x94667E, 0x162901, 0x767A9F, 0xBEFDFD, 0xEF4556, 0x367ED9,
    0x13D9EC, 0xB9BA8B, 0xFC97C4, 0x27A831, 0xC36EF1, 0x36C594, 0x56A8D8, 0xB5A8B4, 0x0ECCCF,
    0x2D8912, 0x34576F, 0x89562C, 0xE3CE99, 0xB920D6, 0xAA5E6B, 0x9C2A3E, 0xCC5F11, 0x4A0BFD,
    0xFBF4E1, 0x6D3B8E, 0x2C86E2, 0x84D4E9, 0xA9B4FC, 0xD1EEEF, 0xC9352E, 0x61392F, 0x442138,
    0xC8D91B, 0x0AFC81, 0x6A4AFB, 0xD81C2F, 0x84B453, 0x8C994E, 0xCC2254, 0xDC552A, 0xD6C6C0,
    0x96190B, 0xB8701A, 0x649569, 0x605A26, 0xEE523F, 0x0F117F, 0x11B5F4, 0xF5CBFC, 0x2DBC34,
    0xEEBC34, 0xCC5DE8, 0x605EDD, 0x9B8E67, 0xEF3392, 0xB817C9, 0x9B5861, 0xBC57E1, 0xC68351,
    0x103ED8, 0x4871DD, 0xDD1C2D, 0xA118AF, 0x462C21, 0xD7F359, 0x987AD9, 0xC0549E, 0xFA864F,
    0xFC0656, 0xAE79E5, 0x362289, 0x22AD38, 0xDC9367, 0xAAE855, 0x382682, 0x9BE7CA, 0xA40D51,
    0xB13399, 0x0ED7A9, 0x480569, 0xF0B265, 0xA7887F, 0x974C88, 0x36D1F9, 0xB39221, 0x4A827B,
    0x21CF98, 0xDC9F40, 0x5547DC, 0x3A74E1, 0x42EB67, 0xDF9DFE, 0x5FD45E, 0xA4677B, 0x7AACBA,
    0xA2F655, 0x23882B, 0x55BA41, 0x086E59, 0x862A21, 0x834739, 0xE6E389, 0xD49EE5, 0x40FB49,
    0xE956FF, 0xCA0F1C, 0x8A59C5, 0x2BFA94, 0xC5C1D3, 0xCFC50F, 0xAE5ADB, 0x86C547, 0x624385,
    0x3B8621, 0x94792C, 0x876110, 0x7B4C2A, 0x1A2C80, 0x12BF43, 0x902688, 0x893C78, 0xE4C4A8,
    0x7BDBE5, 0xC23AC4, 0xEAF426, 0x8A67F7, 0xBF920D, 0x2BA365, 0xB1933D, 0x0B7CBD, 0xDC51A4,
    0x63DD27, 0xDDE169, 0x19949A, 0x9529A8, 0x28CE68, 0xB4ED09, 0x209F44, 0xCA984E, 0x638270,
    0x237C7E, 0x32B90F, 0x8EF5A7, 0xE75614, 0x08F121, 0x2A9DB5, 0x4D7E6F, 0x5119A5, 0xABF9B5,
    0xD6DF82, 0x61DD96, 0x023616, 0x9F3AC4, 0xA1A283, 0x6DED72, 0x7A8D39, 0xA9B882, 0x5C326B,
    0x5B2746, 0xED3400, 0x7700D2, 0x55F4FC, 0x4D5901, 0x8071E0,
];

const PIO2: [f64; 8] = [
    1.57079625129699707031e+00, /* 0x3FF921FB, 0x40000000 */
    7.54978941586159635335e-08, /* 0x3E74442D, 0x00000000 */
    5.39030252995776476554e-15, /* 0x3CF84698, 0x80000000 */
    3.28200341580791294123e-22, /* 0x3B78CC51, 0x60000000 */
    1.27065575308067607349e-29, /* 0x39F01B83, 0x80000000 */
    1.22933308981111328932e-36, /* 0x387A2520, 0x40000000 */
    2.73370053816464559624e-44, /* 0x36E38222, 0x80000000 */
    2.16741683877804819444e-51, /* 0x3569F31D, 0x00000000 */
];

// fn rem_pio2_large(x : &[f64], y : &mut [f64], e0 : i32, prec : usize) -> i32
//
// Input parameters:
//      x[]     The input value (must be positive) is broken into nx
//              pieces of 24-bit integers in double precision format.
//              x[i] will be the i-th 24 bit of x. The scaled exponent
//              of x[0] is given in input parameter e0 (i.e., x[0]*2^e0
//              match x's up to 24 bits.
//
//              Example of breaking a double positive z into x[0]+x[1]+x[2]:
//                      e0 = ilogb(z)-23
//                      z  = scalbn(z,-e0)
//              for i = 0,1,2
//                      x[i] = floor(z)
//                      z    = (z-x[i])*2**24
//
//      y[]     ouput result in an array of double precision numbers.
//              The dimension of y[] is:
//                      24-bit  precision       1
//                      53-bit  precision       2
//                      64-bit  precision       2
//                      113-bit precision       3
//              The actual value is the sum of them. Thus for 113-bit
//              precison, one may have to do something like:
//
//              long double t,w,r_head, r_tail;
//              t = (long double)y[2] + (long double)y[1];
//              w = (long double)y[0];
//              r_head = t+w;
//              r_tail = w - (r_head - t);
//
//      e0      The exponent of x[0]. Must be <= 16360 or you need to
//              expand the ipio2 table.
//
//      prec    an integer indicating the precision:
//                      0       24  bits (single)
//                      1       53  bits (double)
//                      2       64  bits (extended)
//                      3       113 bits (quad)
//
// Here is the description of some local variables:
//
//      jk      jk+1 is the initial number of terms of ipio2[] needed
//              in the computation. The minimum and recommended value
//              for jk is 3,4,4,6 for single, double, extended, and quad.
//              jk+1 must be 2 larger than you might expect so that our
//              recomputation test works. (Up to 24 bits in the integer
//              part (the 24 bits of it that we compute) and 23 bits in
//              the fraction part may be lost to cancelation before we
//              recompute.)
//
//      jz      local integer variable indicating the number of
//              terms of ipio2[] used.
//
//      jx      nx - 1
//
//      jv      index for pointing to the suitable ipio2[] for the
//              computation. In general, we want
//                      ( 2^e0*x[0] * ipio2[jv-1]*2^(-24jv) )/8
//              is an integer. Thus
//                      e0-3-24*jv >= 0 or (e0-3)/24 >= jv
//              Hence jv = max(0,(e0-3)/24).
//
//      jp      jp+1 is the number of terms in PIo2[] needed, jp = jk.
//
//      q[]     double array with integral value, representing the
//              24-bits chunk of the product of x and 2/pi.
//
//      q0      the corresponding exponent of q[0]. Note that the
//              exponent for q[i] would be q0-24*i.
//
//      PIo2[]  double precision array, obtained by cutting pi/2
//              into 24 bits chunks.
//
//      f[]     ipio2[] in floating point
//
//      iq[]    integer array by breaking up q[] in 24-bits chunk.
//
//      fq[]    final product of x*(2/pi) in fq[0],..,fq[jk]
//
//      ih      integer. If >0 it indicates q[] is >= 0.5, hence
//              it also indicates the *sign* of the result.

/// Return the last three digits of N with y = x - N*pi/2
/// so that |y| < pi/2.
///
/// The method is to compute the integer (mod 8) and fraction parts of
/// (2/pi)*x without doing the full multiplication. In general we
/// skip the part of the product that are known to be a huge integer (
/// more accurately, = 0 mod 8 ). Thus the number of operations are
/// independent of the exponent of the input.
#[cfg_attr(all(test, assert_no_panic), no_panic::no_panic)]
pub const fn rem_pio2_large(x: &[f64], y: &mut [f64], e0: i32, prec: usize) -> i32 {
    let x1p24 = f64::from_bits(0x4170000000000000); // 0x1p24 === 2 ^ 24
    let x1p_24 = f64::from_bits(0x3e70000000000000); // 0x1p_24 === 2 ^ (-24)

    #[cfg(all(target_pointer_width = "64", feature = "checked"))]
    assert!(e0 <= 16360);

    let nx = x.len();

    let mut fw: f64;
    let mut n: i32;
    let mut ih: i32;
    let mut z: f64;
    let mut f: [f64; 20] = [0.; 20];
    let mut fq: [f64; 20] = [0.; 20];
    let mut q: [f64; 20] = [0.; 20];
    let mut iq: [i32; 20] = [0; 20];

    /* initialize jk*/
    let jk = i!(INIT_JK, prec);
    let jp = jk;

    /* determine jx,jv,q0, note that 3>q0 */
    let jx = nx - 1;
    let mut jv = div!(e0 - 3, 24);
    if jv < 0 {
        jv = 0;
    }
    let mut q0 = e0 - 24 * (jv + 1);
    let jv = jv as usize;

    /* set up f[0] to f[jx+jk] where f[jx+jk] = ipio2[jv+jk] */
    let mut j = (jv as i32) - (jx as i32);
    let m = jx + jk;
    let mut i = 0;
    while i <= m {
        i!(f, i, =, if j < 0 {
            0.
        } else {
            i!(IPIO2, j as usize) as f64
        });
        j += 1;
        i += 1;
    }

    /* compute q[0],q[1],...q[jk] */
    let mut i = 0;
    while i <= jk {
        fw = 0f64;
        let mut j = 0;
        while j <= jx {
            fw += i!(x, j) * i!(f, jx + i - j);
            j += 1;
        }
        i!(q, i, =, fw);
        i += 1;
    }

    let mut jz = jk;

    'recompute: loop {
        /* distill q[] into iq[] reversingly */
        let mut i = 0i32;
        z = i!(q, jz);
        let mut j = jz;
        while j > 0 {
            fw = (x1p_24 * z) as i32 as f64;
            i!(iq, i as usize, =, (z - x1p24 * fw) as i32);
            z = i!(q, j - 1) + fw;
            i += 1;
            j -= 1;
        }

        /* compute n */
        z = scalbn(z, q0); /* actual value of z */
        z -= 8.0 * floor(z * 0.125); /* trim off integer >= 8 */
        n = z as i32;
        z -= n as f64;
        ih = 0;
        if q0 > 0 {
            /* need iq[jz-1] to determine n */
            i = i!(iq, jz - 1) >> (24 - q0);
            n += i;
            i!(iq, jz - 1, -=, i << (24 - q0));
            ih = i!(iq, jz - 1) >> (23 - q0);
        } else if q0 == 0 {
            ih = i!(iq, jz - 1) >> 23;
        } else if z >= 0.5 {
            ih = 2;
        }

        if ih > 0 {
            /* q > 0.5 */
            n += 1;
            let mut carry = 0i32;
            let mut i = 0;
            while i < jz {
                /* compute 1-q */
                let j = i!(iq, i);
                if carry == 0 {
                    if j != 0 {
                        carry = 1;
                        i!(iq, i, =, 0x1000000 - j);
                    }
                } else {
                    i!(iq, i, =, 0xffffff - j);
                }
                i += 1
            }
            if q0 > 0 {
                /* rare case: chance is 1 in 12 */
                match q0 {
                    1 => {
                        i!(iq, jz - 1, &=, 0x7fffff);
                    }
                    2 => {
                        i!(iq, jz - 1, &=, 0x3fffff);
                    }
                    _ => {}
                }
            }
            if ih == 2 {
                z = 1. - z;
                if carry != 0 {
                    z -= scalbn(1., q0);
                }
            }
        }

        /* check if recomputation is needed */
        if z == 0. {
            let mut j = 0;
            let mut i = jz - 1;
            while i >= jk {
                j |= i!(iq, i);
                i -= 1;
            }
            if j == 0 {
                /* need recomputation */
                let mut k = 1;
                while i!(iq, jk - k, ==, 0) {
                    k += 1; /* k = no. of terms needed */
                }

                let mut i = jz + 1;
                while i <= (jz + k) {
                    /* add q[jz+1] to q[jz+k] */
                    i!(f, jx + i, =, i!(IPIO2, jv + i) as f64);
                    fw = 0f64;
                    let mut j = 0;
                    while j <= jx {
                        fw += i!(x, j) * i!(f, jx + i - j);
                        j += 1;
                    }
                    i!(q, i, =, fw);
                    i += 1;
                }
                jz += k;
                continue 'recompute;
            }
        }

        break;
    }

    /* chop off zero terms */
    if z == 0. {
        jz -= 1;
        q0 -= 24;
        while i!(iq, jz) == 0 {
            jz -= 1;
            q0 -= 24;
        }
    } else {
        /* break z into 24-bit if necessary */
        z = scalbn(z, -q0);
        if z >= x1p24 {
            fw = (x1p_24 * z) as i32 as f64;
            i!(iq, jz, =, (z - x1p24 * fw) as i32);
            jz += 1;
            q0 += 24;
            i!(iq, jz, =, fw as i32);
        } else {
            i!(iq, jz, =, z as i32);
        }
    }

    /* convert integer "bit" chunk to floating-point value */
    fw = scalbn(1., q0);
    let mut i = jz;
    loop {
        i!(q, i, =, fw * (i!(iq, i) as f64));
        fw *= x1p_24;
        i -= 1;
        if i == 0 {
            break;
        }
    }

    /* compute PIo2[0,...,jp]*q[jz,...,0] */
    let mut i = jz;
    loop {
        fw = 0f64;
        let mut k = 0;
        while (k <= jp) && (k <= jz - i) {
            fw += i!(PIO2, k) * i!(q, i + k);
            k += 1;
        }
        i!(fq, jz - i, =, fw);
        i -= 1;
        if i == 0 {
            break;
        }
    }

    /* compress fq[] into y[] */
    match prec {
        0 => {
            fw = 0f64;
            let mut i = jz;
            loop {
                fw += i!(fq, i);
                i -= 1;
                if i == 0 {
                    break;
                }
            }
            i!(y, 0, =, if ih == 0 { fw } else { -fw });
        }
        1 | 2 => {
            fw = 0f64;
            let mut i = jz;
            loop {
                fw += i!(fq, i);
                i -= 1;
                if i == 0 {
                    break;
                }
            }
            // TODO: drop excess precision here once double_t is used
            fw = fw as f64;
            i!(y, 0, =, if ih == 0 { fw } else { -fw });
            fw = i!(fq, 0) - fw;
            let mut i = 1;
            while i <= jz {
                fw += i!(fq, i);
                i += 1;
            }
            i!(y, 1, =, if ih == 0 { fw } else { -fw });
        }
        3 => {
            /* painful */
            let mut i = jz;
            while i >= 1 {
                fw = i!(fq, i - 1) + i!(fq, i);
                i!(fq, i, +=, i!(fq, i - 1) - fw);
                i!(fq, i - 1, =, fw);
                i -= 1;
            }
            let mut i = jz;
            while i >= 2 {
                fw = i!(fq, i - 1) + i!(fq, i);
                i!(fq, i, +=, i!(fq, i - 1) - fw);
                i!(fq, i - 1, =, fw);
                i -= 1;
            }
            fw = 0f64;
            let mut i = jz;
            while i >= 2 {
                fw += i!(fq, i);
                i -= 1;
            }
            if ih == 0 {
                i!(y, 0, =, i!(fq, 0));
                i!(y, 1, =, i!(fq, 1));
                i!(y, 2, =, fw);
            } else {
                i!(y, 0, =, -i!(fq, 0));
                i!(y, 1, =, -i!(fq, 1));
                i!(y, 2, =, -fw);
            }
        }
        #[cfg(debug_assertions)]
        _ => unreachable!(),
        #[cfg(not(debug_assertions))]
        _ => {}
    }
    n & 7
}

// #if FLT_EVAL_METHOD==0 || FLT_EVAL_METHOD==1
// #define EPS DBL_EPSILON
const EPS: f64 = 2.2204460492503131e-16;
// #elif FLT_EVAL_METHOD==2
// #define EPS LDBL_EPSILON
// #endif

// TODO: Support FLT_EVAL_METHOD?

const TO_INT: f64 = 1.5 / EPS;
/// 53 bits of 2/pi
const INV_PIO2: f64 = 6.36619772367581382433e-01; /* 0x3FE45F30, 0x6DC9C883 */
/// first 33 bits of pi/2
const PIO2_1: f64 = 1.57079632673412561417e+00; /* 0x3FF921FB, 0x54400000 */
/// pi/2 - PIO2_1
const PIO2_1T: f64 = 6.07710050650619224932e-11; /* 0x3DD0B461, 0x1A626331 */
/// second 33 bits of pi/2
const PIO2_2: f64 = 6.07710050630396597660e-11; /* 0x3DD0B461, 0x1A600000 */
/// pi/2 - (PIO2_1+PIO2_2)
const PIO2_2T: f64 = 2.02226624879595063154e-21; /* 0x3BA3198A, 0x2E037073 */
/// third 33 bits of pi/2
const PIO2_3: f64 = 2.02226624871116645580e-21; /* 0x3BA3198A, 0x2E000000 */
/// pi/2 - (PIO2_1+PIO2_2+PIO2_3)
const PIO2_3T: f64 = 8.47842766036889956997e-32; /* 0x397B839A, 0x252049C1 */

// return the remainder of x rem pi/2 in y[0]+y[1]
// use rem_pio2_large() for large x
//
// caller must handle the case when reduction is not needed: |x| ~<= pi/4 */
#[cfg_attr(all(test, assert_no_panic), no_panic::no_panic)]
pub const fn rem_pio2(x: f64) -> (i32, f64, f64) {
    let x1p24 = f64::from_bits(0x4170000000000000);

    let sign = (f64::to_bits(x) >> 63) as i32;
    let ix = (f64::to_bits(x) >> 32) as u32 & 0x7fffffff;

    const fn medium(x: f64, ix: u32) -> (i32, f64, f64) {
        /* rint(x/(pi/2)), Assume round-to-nearest. */
        let f_n = x as f64 * INV_PIO2 + TO_INT - TO_INT;
        let n = f_n as i32;
        let mut r = x - f_n * PIO2_1;
        let mut w = f_n * PIO2_1T; /* 1st round, good to 85 bits */
        let mut y0 = r - w;
        let ui = f64::to_bits(y0);
        let ey = (ui >> 52) as i32 & 0x7ff;
        let ex = (ix >> 20) as i32;
        if ex - ey > 16 {
            /* 2nd round, good to 118 bits */
            let t = r;
            w = f_n * PIO2_2;
            r = t - w;
            w = f_n * PIO2_2T - ((t - r) - w);
            y0 = r - w;
            let ey = (f64::to_bits(y0) >> 52) as i32 & 0x7ff;
            if ex - ey > 49 {
                /* 3rd round, good to 151 bits, covers all cases */
                let t = r;
                w = f_n * PIO2_3;
                r = t - w;
                w = f_n * PIO2_3T - ((t - r) - w);
                y0 = r - w;
            }
        }
        let y1 = (r - y0) - w;
        (n, y0, y1)
    }

    if ix <= 0x400f6a7a {
        /* |x| ~<= 5pi/4 */
        if (ix & 0xfffff) == 0x921fb {
            /* |x| ~= pi/2 or 2pi/2 */
            return medium(x, ix); /* cancellation -- use medium case */
        }
        if ix <= 0x4002d97c {
            /* |x| ~<= 3pi/4 */
            if sign == 0 {
                let z = x - PIO2_1; /* one round good to 85 bits */
                let y0 = z - PIO2_1T;
                let y1 = (z - y0) - PIO2_1T;
                return (1, y0, y1);
            } else {
                let z = x + PIO2_1;
                let y0 = z + PIO2_1T;
                let y1 = (z - y0) + PIO2_1T;
                return (-1, y0, y1);
            }
        } else if sign == 0 {
            let z = x - 2.0 * PIO2_1;
            let y0 = z - 2.0 * PIO2_1T;
            let y1 = (z - y0) - 2.0 * PIO2_1T;
            return (2, y0, y1);
        } else {
            let z = x + 2.0 * PIO2_1;
            let y0 = z + 2.0 * PIO2_1T;
            let y1 = (z - y0) + 2.0 * PIO2_1T;
            return (-2, y0, y1);
        }
    }
    if ix <= 0x401c463b {
        /* |x| ~<= 9pi/4 */
        if ix <= 0x4015fdbc {
            /* |x| ~<= 7pi/4 */
            if ix == 0x4012d97c {
                /* |x| ~= 3pi/2 */
                return medium(x, ix);
            }
            if sign == 0 {
                let z = x - 3.0 * PIO2_1;
                let y0 = z - 3.0 * PIO2_1T;
                let y1 = (z - y0) - 3.0 * PIO2_1T;
                return (3, y0, y1);
            } else {
                let z = x + 3.0 * PIO2_1;
                let y0 = z + 3.0 * PIO2_1T;
                let y1 = (z - y0) + 3.0 * PIO2_1T;
                return (-3, y0, y1);
            }
        } else {
            if ix == 0x401921fb {
                /* |x| ~= 4pi/2 */
                return medium(x, ix);
            }
            if sign == 0 {
                let z = x - 4.0 * PIO2_1;
                let y0 = z - 4.0 * PIO2_1T;
                let y1 = (z - y0) - 4.0 * PIO2_1T;
                return (4, y0, y1);
            } else {
                let z = x + 4.0 * PIO2_1;
                let y0 = z + 4.0 * PIO2_1T;
                let y1 = (z - y0) + 4.0 * PIO2_1T;
                return (-4, y0, y1);
            }
        }
    }
    if ix < 0x413921fb {
        /* |x| ~< 2^20*(pi/2), medium size */
        return medium(x, ix);
    }
    /*
     * all other (large) arguments
     */
    if ix >= 0x7ff00000 {
        /* x is inf or NaN */
        let y0 = x - x;
        let y1 = y0;
        return (0, y0, y1);
    }
    /* set z = scalbn(|x|,-ilogb(x)+23) */
    let mut ui = f64::to_bits(x);
    ui &= (!1) >> 12;
    ui |= (0x3ff + 23) << 52;
    let mut z = f64::from_bits(ui);
    let mut tx = [0.0; 3];
    let mut i = 0;
    while i < 2 {
        i!(tx,i, =, z as i32 as f64);
        z = (z - i!(tx, i)) * x1p24;
        i += 1;
    }
    i!(tx,2, =, z);
    /* skip zero terms, first term is non-zero */
    let mut i = 2;
    while i != 0 && i!(tx, i) == 0.0 {
        i -= 1;
    }
    let mut ty = [0.0; 3];
    let n;
    match i {
        2 => {
            n = rem_pio2_large(
                &[tx[0], tx[1], tx[2]],
                &mut ty,
                ((ix as i32) >> 20) - (0x3ff + 23),
                1,
            );
        }
        1 => {
            n = rem_pio2_large(
                &[tx[0], tx[1]],
                &mut ty,
                ((ix as i32) >> 20) - (0x3ff + 23),
                1,
            );
        }
        0 => {
            n = rem_pio2_large(&[tx[0]], &mut ty, ((ix as i32) >> 20) - (0x3ff + 23), 1);
        }
        _ => unreachable!(),
    }

    if sign != 0 {
        return (-n, -i!(ty, 0), -i!(ty, 1));
    }
    (n, i!(ty, 0), i!(ty, 1))
}

pub const fn sin(x: f64) -> f64 {
    /* High word of x. */
    let ix = (f64::to_bits(x) >> 32) as u32 & 0x7fffffff;

    /* |x| ~< pi/4 */
    if ix <= 0x3fe921fb {
        if ix < 0x3e500000 {
            /* |x| < 2**-26 */
            return x;
        }
        return k_sin(x, 0.0, 0);
    }

    /* sin(Inf or NaN) is NaN */
    if ix >= 0x7ff00000 {
        return x - x;
    }

    /* argument reduction needed */
    let (n, y0, y1) = rem_pio2(x);
    match n & 3 {
        0 => k_sin(y0, y1, 1),
        1 => k_cos(y0, y1),
        2 => -k_sin(y0, y1, 1),
        _ => -k_cos(y0, y1),
    }
}

// origin: FreeBSD /usr/src/lib/msun/src/s_exp2.c */
//-
// Copyright (c) 2005 David Schultz <das@FreeBSD.ORG>
// All rights reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions
// are met:
// 1. Redistributions of source code must retain the above copyright
//    notice, this list of conditions and the following disclaimer.
// 2. Redistributions in binary form must reproduce the above copyright
//    notice, this list of conditions and the following disclaimer in the
//    documentation and/or other materials provided with the distribution.
//
// THIS SOFTWARE IS PROVIDED BY THE AUTHOR AND CONTRIBUTORS ``AS IS'' AND
// ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
// IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE
// ARE DISCLAIMED.  IN NO EVENT SHALL THE AUTHOR OR CONTRIBUTORS BE LIABLE
// FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
// DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS
// OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION)
// HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT
// LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY
// OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF
// SUCH DAMAGE.

const TBLSIZE: usize = 256;

#[cfg_attr(rustfmt, rustfmt_skip)]
const TBL: [u64; TBLSIZE * 2] = [
    //  exp2(z + eps)          eps
    0x3fe6a09e667f3d5d, 0x3d39880000000000,
    0x3fe6b052fa751744, 0x3cd8000000000000,
    0x3fe6c012750bd9fe, 0xbd28780000000000,
    0x3fe6cfdcddd476bf, 0x3d1ec00000000000,
    0x3fe6dfb23c651a29, 0xbcd8000000000000,
    0x3fe6ef9298593ae3, 0xbcbc000000000000,
    0x3fe6ff7df9519386, 0xbd2fd80000000000,
    0x3fe70f7466f42da3, 0xbd2c880000000000,
    0x3fe71f75e8ec5fc3, 0x3d13c00000000000,
    0x3fe72f8286eacf05, 0xbd38300000000000,
    0x3fe73f9a48a58152, 0xbd00c00000000000,
    0x3fe74fbd35d7ccfc, 0x3d2f880000000000,
    0x3fe75feb564267f1, 0x3d03e00000000000,
    0x3fe77024b1ab6d48, 0xbd27d00000000000,
    0x3fe780694fde5d38, 0xbcdd000000000000,
    0x3fe790b938ac1d00, 0x3ce3000000000000,
    0x3fe7a11473eb0178, 0xbced000000000000,
    0x3fe7b17b0976d060, 0x3d20400000000000,
    0x3fe7c1ed0130c133, 0x3ca0000000000000,
    0x3fe7d26a62ff8636, 0xbd26900000000000,
    0x3fe7e2f336cf4e3b, 0xbd02e00000000000,
    0x3fe7f3878491c3e8, 0xbd24580000000000,
    0x3fe80427543e1b4e, 0x3d33000000000000,
    0x3fe814d2add1071a, 0x3d0f000000000000,
    0x3fe82589994ccd7e, 0xbd21c00000000000,
    0x3fe8364c1eb942d0, 0x3d29d00000000000,
    0x3fe8471a4623cab5, 0x3d47100000000000,
    0x3fe857f4179f5bbc, 0x3d22600000000000,
    0x3fe868d99b4491af, 0xbd32c40000000000,
    0x3fe879cad931a395, 0xbd23000000000000,
    0x3fe88ac7d98a65b8, 0xbd2a800000000000,
    0x3fe89bd0a4785800, 0xbced000000000000,
    0x3fe8ace5422aa223, 0x3d33280000000000,
    0x3fe8be05bad619fa, 0x3d42b40000000000,
    0x3fe8cf3216b54383, 0xbd2ed00000000000,
    0x3fe8e06a5e08664c, 0xbd20500000000000,
    0x3fe8f1ae99157807, 0x3d28280000000000,
    0x3fe902fed0282c0e, 0xbd1cb00000000000,
    0x3fe9145b0b91ff96, 0xbd05e00000000000,
    0x3fe925c353aa2ff9, 0x3cf5400000000000,
    0x3fe93737b0cdc64a, 0x3d17200000000000,
    0x3fe948b82b5f98ae, 0xbd09000000000000,
    0x3fe95a44cbc852cb, 0x3d25680000000000,
    0x3fe96bdd9a766f21, 0xbd36d00000000000,
    0x3fe97d829fde4e2a, 0xbd01000000000000,
    0x3fe98f33e47a23a3, 0x3d2d000000000000,
    0x3fe9a0f170ca0604, 0xbd38a40000000000,
    0x3fe9b2bb4d53ff89, 0x3d355c0000000000,
    0x3fe9c49182a3f15b, 0x3d26b80000000000,
    0x3fe9d674194bb8c5, 0xbcec000000000000,
    0x3fe9e86319e3238e, 0x3d17d00000000000,
    0x3fe9fa5e8d07f302, 0x3d16400000000000,
    0x3fea0c667b5de54d, 0xbcf5000000000000,
    0x3fea1e7aed8eb8f6, 0x3d09e00000000000,
    0x3fea309bec4a2e27, 0x3d2ad80000000000,
    0x3fea42c980460a5d, 0xbd1af00000000000,
    0x3fea5503b23e259b, 0x3d0b600000000000,
    0x3fea674a8af46213, 0x3d38880000000000,
    0x3fea799e1330b3a7, 0x3d11200000000000,
    0x3fea8bfe53c12e8d, 0x3d06c00000000000,
    0x3fea9e6b5579fcd2, 0xbd29b80000000000,
    0x3feab0e521356fb8, 0x3d2b700000000000,
    0x3feac36bbfd3f381, 0x3cd9000000000000,
    0x3fead5ff3a3c2780, 0x3ce4000000000000,
    0x3feae89f995ad2a3, 0xbd2c900000000000,
    0x3feafb4ce622f367, 0x3d16500000000000,
    0x3feb0e07298db790, 0x3d2fd40000000000,
    0x3feb20ce6c9a89a9, 0x3d12700000000000,
    0x3feb33a2b84f1a4b, 0x3d4d470000000000,
    0x3feb468415b747e7, 0xbd38380000000000,
    0x3feb59728de5593a, 0x3c98000000000000,
    0x3feb6c6e29f1c56a, 0x3d0ad00000000000,
    0x3feb7f76f2fb5e50, 0x3cde800000000000,
    0x3feb928cf22749b2, 0xbd04c00000000000,
    0x3feba5b030a10603, 0xbd0d700000000000,
    0x3febb8e0b79a6f66, 0x3d0d900000000000,
    0x3febcc1e904bc1ff, 0x3d02a00000000000,
    0x3febdf69c3f3a16f, 0xbd1f780000000000,
    0x3febf2c25bd71db8, 0xbd10a00000000000,
    0x3fec06286141b2e9, 0xbd11400000000000,
    0x3fec199bdd8552e0, 0x3d0be00000000000,
    0x3fec2d1cd9fa64ee, 0xbd09400000000000,
    0x3fec40ab5fffd02f, 0xbd0ed00000000000,
    0x3fec544778fafd15, 0x3d39660000000000,
    0x3fec67f12e57d0cb, 0xbd1a100000000000,
    0x3fec7ba88988c1b6, 0xbd58458000000000,
    0x3fec8f6d9406e733, 0xbd1a480000000000,
    0x3feca3405751c4df, 0x3ccb000000000000,
    0x3fecb720dcef9094, 0x3d01400000000000,
    0x3feccb0f2e6d1689, 0x3cf0200000000000,
    0x3fecdf0b555dc412, 0x3cf3600000000000,
    0x3fecf3155b5bab3b, 0xbd06900000000000,
    0x3fed072d4a0789bc, 0x3d09a00000000000,
    0x3fed1b532b08c8fa, 0xbd15e00000000000,
    0x3fed2f87080d8a85, 0x3d1d280000000000,
    0x3fed43c8eacaa203, 0x3d01a00000000000,
    0x3fed5818dcfba491, 0x3cdf000000000000,
    0x3fed6c76e862e6a1, 0xbd03a00000000000,
    0x3fed80e316c9834e, 0xbd0cd80000000000,
    0x3fed955d71ff6090, 0x3cf4c00000000000,
    0x3feda9e603db32ae, 0x3cff900000000000,
    0x3fedbe7cd63a8325, 0x3ce9800000000000,
    0x3fedd321f301b445, 0xbcf5200000000000,
    0x3fede7d5641c05bf, 0xbd1d700000000000,
    0x3fedfc97337b9aec, 0xbd16140000000000,
    0x3fee11676b197d5e, 0x3d0b480000000000,
    0x3fee264614f5a3e7, 0x3d40ce0000000000,
    0x3fee3b333b16ee5c, 0x3d0c680000000000,
    0x3fee502ee78b3fb4, 0xbd09300000000000,
    0x3fee653924676d68, 0xbce5000000000000,
    0x3fee7a51fbc74c44, 0xbd07f80000000000,
    0x3fee8f7977cdb726, 0xbcf3700000000000,
    0x3feea4afa2a490e8, 0x3ce5d00000000000,
    0x3feeb9f4867ccae4, 0x3d161a0000000000,
    0x3feecf482d8e680d, 0x3cf5500000000000,
    0x3feee4aaa2188514, 0x3cc6400000000000,
    0x3feefa1bee615a13, 0xbcee800000000000,
    0x3fef0f9c1cb64106, 0xbcfa880000000000,
    0x3fef252b376bb963, 0xbd2c900000000000,
    0x3fef3ac948dd7275, 0x3caa000000000000,
    0x3fef50765b6e4524, 0xbcf4f00000000000,
    0x3fef6632798844fd, 0x3cca800000000000,
    0x3fef7bfdad9cbe38, 0x3cfabc0000000000,
    0x3fef91d802243c82, 0xbcd4600000000000,
    0x3fefa7c1819e908e, 0xbd0b0c0000000000,
    0x3fefbdba3692d511, 0xbcc0e00000000000,
    0x3fefd3c22b8f7194, 0xbd10de8000000000,
    0x3fefe9d96b2a23ee, 0x3cee430000000000,
    0x3ff0000000000000, 0x0,
    0x3ff00b1afa5abcbe, 0xbcb3400000000000,
    0x3ff0163da9fb3303, 0xbd12170000000000,
    0x3ff02168143b0282, 0x3cba400000000000,
    0x3ff02c9a3e77806c, 0x3cef980000000000,
    0x3ff037d42e11bbca, 0xbcc7400000000000,
    0x3ff04315e86e7f89, 0x3cd8300000000000,
    0x3ff04e5f72f65467, 0xbd1a3f0000000000,
    0x3ff059b0d315855a, 0xbd02840000000000,
    0x3ff0650a0e3c1f95, 0x3cf1600000000000,
    0x3ff0706b29ddf71a, 0x3d15240000000000,
    0x3ff07bd42b72a82d, 0xbce9a00000000000,
    0x3ff0874518759bd0, 0x3ce6400000000000,
    0x3ff092bdf66607c8, 0xbd00780000000000,
    0x3ff09e3ecac6f383, 0xbc98000000000000,
    0x3ff0a9c79b1f3930, 0x3cffa00000000000,
    0x3ff0b5586cf988fc, 0xbcfac80000000000,
    0x3ff0c0f145e46c8a, 0x3cd9c00000000000,
    0x3ff0cc922b724816, 0x3d05200000000000,
    0x3ff0d83b23395dd8, 0xbcfad00000000000,
    0x3ff0e3ec32d3d1f3, 0x3d1bac0000000000,
    0x3ff0efa55fdfa9a6, 0xbd04e80000000000,
    0x3ff0fb66affed2f0, 0xbd0d300000000000,
    0x3ff1073028d7234b, 0x3cf1500000000000,
    0x3ff11301d0125b5b, 0x3cec000000000000,
    0x3ff11edbab5e2af9, 0x3d16bc0000000000,
    0x3ff12abdc06c31d5, 0x3ce8400000000000,
    0x3ff136a814f2047d, 0xbd0ed00000000000,
    0x3ff1429aaea92de9, 0x3ce8e00000000000,
    0x3ff14e95934f3138, 0x3ceb400000000000,
    0x3ff15a98c8a58e71, 0x3d05300000000000,
    0x3ff166a45471c3df, 0x3d03380000000000,
    0x3ff172b83c7d5211, 0x3d28d40000000000,
    0x3ff17ed48695bb9f, 0xbd05d00000000000,
    0x3ff18af9388c8d93, 0xbd1c880000000000,
    0x3ff1972658375d66, 0x3d11f00000000000,
    0x3ff1a35beb6fcba7, 0x3d10480000000000,
    0x3ff1af99f81387e3, 0xbd47390000000000,
    0x3ff1bbe084045d54, 0x3d24e40000000000,
    0x3ff1c82f95281c43, 0xbd0a200000000000,
    0x3ff1d4873168b9b2, 0x3ce3800000000000,
    0x3ff1e0e75eb44031, 0x3ceac00000000000,
    0x3ff1ed5022fcd938, 0x3d01900000000000,
    0x3ff1f9c18438cdf7, 0xbd1b780000000000,
    0x3ff2063b88628d8f, 0x3d2d940000000000,
    0x3ff212be3578a81e, 0x3cd8000000000000,
    0x3ff21f49917ddd41, 0x3d2b340000000000,
    0x3ff22bdda2791323, 0x3d19f80000000000,
    0x3ff2387a6e7561e7, 0xbd19c80000000000,
    0x3ff2451ffb821427, 0x3d02300000000000,
    0x3ff251ce4fb2a602, 0xbd13480000000000,
    0x3ff25e85711eceb0, 0x3d12700000000000,
    0x3ff26b4565e27d16, 0x3d11d00000000000,
    0x3ff2780e341de00f, 0x3d31ee0000000000,
    0x3ff284dfe1f5633e, 0xbd14c00000000000,
    0x3ff291ba7591bb30, 0xbd13d80000000000,
    0x3ff29e9df51fdf09, 0x3d08b00000000000,
    0x3ff2ab8a66d10e9b, 0xbd227c0000000000,
    0x3ff2b87fd0dada3a, 0x3d2a340000000000,
    0x3ff2c57e39771af9, 0xbd10800000000000,
    0x3ff2d285a6e402d9, 0xbd0ed00000000000,
    0x3ff2df961f641579, 0xbcf4200000000000,
    0x3ff2ecafa93e2ecf, 0xbd24980000000000,
    0x3ff2f9d24abd8822, 0xbd16300000000000,
    0x3ff306fe0a31b625, 0xbd32360000000000,
    0x3ff31432edeea50b, 0xbd70df8000000000,
    0x3ff32170fc4cd7b8, 0xbd22480000000000,
    0x3ff32eb83ba8e9a2, 0xbd25980000000000,
    0x3ff33c08b2641766, 0x3d1ed00000000000,
    0x3ff3496266e3fa27, 0xbcdc000000000000,
    0x3ff356c55f929f0f, 0xbd30d80000000000,
    0x3ff36431a2de88b9, 0x3d22c80000000000,
    0x3ff371a7373aaa39, 0x3d20600000000000,
    0x3ff37f26231e74fe, 0xbd16600000000000,
    0x3ff38cae6d05d838, 0xbd0ae00000000000,
    0x3ff39a401b713ec3, 0xbd44720000000000,
    0x3ff3a7db34e5a020, 0x3d08200000000000,
    0x3ff3b57fbfec6e95, 0x3d3e800000000000,
    0x3ff3c32dc313a8f2, 0x3cef800000000000,
    0x3ff3d0e544ede122, 0xbd17a00000000000,
    0x3ff3dea64c1234bb, 0x3d26300000000000,
    0x3ff3ec70df1c4ecc, 0xbd48a60000000000,
    0x3ff3fa4504ac7e8c, 0xbd3cdc0000000000,
    0x3ff40822c367a0bb, 0x3d25b80000000000,
    0x3ff4160a21f72e95, 0x3d1ec00000000000,
    0x3ff423fb27094646, 0xbd13600000000000,
    0x3ff431f5d950a920, 0x3d23980000000000,
    0x3ff43ffa3f84b9eb, 0x3cfa000000000000,
    0x3ff44e0860618919, 0xbcf6c00000000000,
    0x3ff45c2042a7d201, 0xbd0bc00000000000,
    0x3ff46a41ed1d0016, 0xbd12800000000000,
    0x3ff4786d668b3326, 0x3d30e00000000000,
    0x3ff486a2b5c13c00, 0xbd2d400000000000,
    0x3ff494e1e192af04, 0x3d0c200000000000,
    0x3ff4a32af0d7d372, 0xbd1e500000000000,
    0x3ff4b17dea6db801, 0x3d07800000000000,
    0x3ff4bfdad53629e1, 0xbd13800000000000,
    0x3ff4ce41b817c132, 0x3d00800000000000,
    0x3ff4dcb299fddddb, 0x3d2c700000000000,
    0x3ff4eb2d81d8ab96, 0xbd1ce00000000000,
    0x3ff4f9b2769d2d02, 0x3d19200000000000,
    0x3ff508417f4531c1, 0xbd08c00000000000,
    0x3ff516daa2cf662a, 0xbcfa000000000000,
    0x3ff5257de83f51ea, 0x3d4a080000000000,
    0x3ff5342b569d4eda, 0xbd26d80000000000,
    0x3ff542e2f4f6ac1a, 0xbd32440000000000,
    0x3ff551a4ca5d94db, 0x3d483c0000000000,
    0x3ff56070dde9116b, 0x3d24b00000000000,
    0x3ff56f4736b529de, 0x3d415a0000000000,
    0x3ff57e27dbe2c40e, 0xbd29e00000000000,
    0x3ff58d12d497c76f, 0xbd23080000000000,
    0x3ff59c0827ff0b4c, 0x3d4dec0000000000,
    0x3ff5ab07dd485427, 0xbcc4000000000000,
    0x3ff5ba11fba87af4, 0x3d30080000000000,
    0x3ff5c9268a59460b, 0xbd26c80000000000,
    0x3ff5d84590998e3f, 0x3d469a0000000000,
    0x3ff5e76f15ad20e1, 0xbd1b400000000000,
    0x3ff5f6a320dcebca, 0x3d17700000000000,
    0x3ff605e1b976dcb8, 0x3d26f80000000000,
    0x3ff6152ae6cdf715, 0x3d01000000000000,
    0x3ff6247eb03a5531, 0xbd15d00000000000,
    0x3ff633dd1d1929b5, 0xbd12d00000000000,
    0x3ff6434634ccc313, 0xbcea800000000000,
    0x3ff652b9febc8efa, 0xbd28600000000000,
    0x3ff6623882553397, 0x3d71fe0000000000,
    0x3ff671c1c708328e, 0xbd37200000000000,
    0x3ff68155d44ca97e, 0x3ce6800000000000,
    0x3ff690f4b19e9471, 0xbd29780000000000,
];

// exp2(x): compute the base 2 exponential of x
//
// Accuracy: Peak error < 0.503 ulp for normalized results.
//
// Method: (accurate tables)
//
//   Reduce x:
//     x = k + y, for integer k and |y| <= 1/2.
//     Thus we have exp2(x) = 2**k * exp2(y).
//
//   Reduce y:
//     y = i/TBLSIZE + z - eps[i] for integer i near y * TBLSIZE.
//     Thus we have exp2(y) = exp2(i/TBLSIZE) * exp2(z - eps[i]),
//     with |z - eps[i]| <= 2**-9 + 2**-39 for the table used.
//
//   We compute exp2(i/TBLSIZE) via table lookup and exp2(z - eps[i]) via
//   a degree-5 minimax polynomial with maximum error under 1.3 * 2**-61.
//   The values in exp2t[] and eps[] are chosen such that
//   exp2t[i] = exp2(i/TBLSIZE + eps[i]), and eps[i] is a small offset such
//   that exp2t[i] is accurate to 2**-64.
//
//   Note that the range of i is +-TBLSIZE/2, so we actually index the tables
//   by i0 = i + TBLSIZE/2.  For cache efficiency, exp2t[] and eps[] are
//   virtual tables, interleaved in the real table tbl[].
//
//   This method is due to Gal, with many details due to Gal and Bachelis:
//
//      Gal, S. and Bachelis, B.  An Accurate Elementary Mathematical Library
//      for the IEEE Floating Point Standard.  TOMS 17(1), 26-46 (1991).

/// Exponential, base 2 (f64)
///
/// Calculate `2^x`, that is, 2 raised to the power `x`.
#[cfg_attr(all(test, assert_no_panic), no_panic::no_panic)]
pub const fn exp2(mut x: f64) -> f64 {
    let redux = f64::from_bits(0x4338000000000000) / TBLSIZE as f64;
    let p1 = f64::from_bits(0x3fe62e42fefa39ef);
    let p2 = f64::from_bits(0x3fcebfbdff82c575);
    let p3 = f64::from_bits(0x3fac6b08d704a0a6);
    let p4 = f64::from_bits(0x3f83b2ab88f70400);
    let p5 = f64::from_bits(0x3f55d88003875c74);

    // double_t r, t, z;
    // uint32_t ix, i0;
    // union {double f; uint64_t i;} u = {x};
    // union {uint32_t u; int32_t i;} k;
    let x1p1023 = f64::from_bits(0x7fe0000000000000);
    let _0x1p_149 = f64::from_bits(0xb6a0000000000000);

    /* Filter out exceptional cases. */
    let ui = f64::to_bits(x);
    let ix = ui >> 32 & 0x7fffffff;
    if ix >= 0x408ff000 {
        /* |x| >= 1022 or nan */
        if ix >= 0x40900000 && ui >> 63 == 0 {
            /* x >= 1024 or nan */
            /* overflow */
            x *= x1p1023;
            return x;
        }
        if ix >= 0x7ff00000 {
            /* -inf or -nan */
            return -1.0 / x;
        }
        if ui >> 63 != 0 {
            /* x <= -1022 */
            if x <= -1075.0 {
                return 0.0;
            }
        }
    } else if ix < 0x3c900000 {
        /* |x| < 0x1p-54 */
        return 1.0 + x;
    }

    /* Reduce x, computing z, i0, and k. */
    let ui = f64::to_bits(x + redux);
    let mut i0 = ui as u32;
    i0 = i0.wrapping_add(TBLSIZE as u32 / 2);
    let ku = i0 / TBLSIZE as u32 * TBLSIZE as u32;
    let ki = div!(ku as i32, TBLSIZE as i32);
    i0 %= TBLSIZE as u32;
    let uf = f64::from_bits(ui) - redux;
    let mut z = x - uf;

    /* Compute r = exp2(y) = exp2t[i0] * p(z - eps[i]). */
    let t = f64::from_bits(i!(TBL, 2 * i0 as usize)); /* exp2t[i0] */
    z -= f64::from_bits(i!(TBL, 2 * i0 as usize + 1)); /* eps[i0]   */
    let r = t + t * z * (p1 + z * (p2 + z * (p3 + z * (p4 + z * p5))));

    scalbn(r, ki)
}
