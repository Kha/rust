// Copyright 2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![feature(repr_simd, platform_intrinsics)]

#[repr(simd)]
#[derive(Copy, Clone)]
#[allow(non_camel_case_types)]
struct i32x2(i32, i32);
#[repr(simd)]
#[derive(Copy, Clone)]
#[allow(non_camel_case_types)]
struct i32x3(i32, i32, i32);
#[repr(simd)]
#[derive(Copy, Clone)]
#[allow(non_camel_case_types)]
struct i32x4(i32, i32, i32, i32);
#[repr(simd)]
#[derive(Copy, Clone)]
#[allow(non_camel_case_types)]
struct i32x8(i32, i32, i32, i32,
             i32, i32, i32, i32);

#[repr(simd)]
#[derive(Copy, Clone)]
#[allow(non_camel_case_types)]
struct f32x2(f32, f32);
#[repr(simd)]
#[derive(Copy, Clone)]
#[allow(non_camel_case_types)]
struct f32x3(f32, f32, f32);
#[repr(simd)]
#[derive(Copy, Clone)]
#[allow(non_camel_case_types)]
struct f32x4(f32, f32, f32, f32);
#[repr(simd)]
#[derive(Copy, Clone)]
#[allow(non_camel_case_types)]
struct f32x8(f32, f32, f32, f32,
             f32, f32, f32, f32);

extern "platform-intrinsic" {
    fn simd_insert<T, E>(x: T, idx: u32, y: E) -> T;
    fn simd_extract<T, E>(x: T, idx: u32) -> E;

    fn simd_shuffle2<T, U>(x: T, y: T, idx: [u32; 2]) -> U;
    fn simd_shuffle3<T, U>(x: T, y: T, idx: [u32; 3]) -> U;
    fn simd_shuffle4<T, U>(x: T, y: T, idx: [u32; 4]) -> U;
    fn simd_shuffle8<T, U>(x: T, y: T, idx: [u32; 8]) -> U;
}

fn main() {
    let x = i32x4(0, 0, 0, 0);

    unsafe {
        simd_insert(0, 0, 0);
        //~^ ERROR expected SIMD input type, found non-SIMD `i32`
        simd_insert(x, 0, 1.0);
        //~^ ERROR expected inserted type `i32` (element of input `i32x4`), found `f64`
        simd_extract::<_, f32>(x, 0);
        //~^ ERROR expected return type `i32` (element of input `i32x4`), found `f32`

        simd_shuffle2::<i32, i32>(0, 0, [0; 2]);
        //~^ ERROR expected SIMD input type, found non-SIMD `i32`
        simd_shuffle3::<i32, i32>(0, 0, [0; 3]);
        //~^ ERROR expected SIMD input type, found non-SIMD `i32`
        simd_shuffle4::<i32, i32>(0, 0, [0; 4]);
        //~^ ERROR expected SIMD input type, found non-SIMD `i32`
        simd_shuffle8::<i32, i32>(0, 0, [0; 8]);
        //~^ ERROR expected SIMD input type, found non-SIMD `i32`

        simd_shuffle2::<_, f32x2>(x, x, [0; 2]);
//~^ ERROR element type `i32` (element of input `i32x4`), found `f32x2` with element type `f32`
        simd_shuffle3::<_, f32x3>(x, x, [0; 3]);
//~^ ERROR element type `i32` (element of input `i32x4`), found `f32x3` with element type `f32`
        simd_shuffle4::<_, f32x4>(x, x, [0; 4]);
//~^ ERROR element type `i32` (element of input `i32x4`), found `f32x4` with element type `f32`
        simd_shuffle8::<_, f32x8>(x, x, [0; 8]);
//~^ ERROR element type `i32` (element of input `i32x4`), found `f32x8` with element type `f32`

        simd_shuffle2::<_, i32x8>(x, x, [0; 2]);
        //~^ ERROR expected return type of length 2, found `i32x8` with length 8
        simd_shuffle3::<_, i32x4>(x, x, [0; 3]);
        //~^ ERROR expected return type of length 3, found `i32x4` with length 4
        simd_shuffle4::<_, i32x3>(x, x, [0; 4]);
        //~^ ERROR expected return type of length 4, found `i32x3` with length 3
        simd_shuffle8::<_, i32x2>(x, x, [0; 8]);
        //~^ ERROR expected return type of length 8, found `i32x2` with length 2
    }
}
