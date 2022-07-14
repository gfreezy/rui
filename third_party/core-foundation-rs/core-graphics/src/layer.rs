// Copyright 2015 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use base::CGFloat;
use color::CGColor;
use color_space::CGColorSpace;
use core_foundation::base::{CFTypeID, TCFType};
use core_foundation::dictionary::{CFDictionary, CFDictionaryRef};
use font::{CGFont, CGGlyph};
use geometry::{CGPoint, CGSize};
use gradient::{CGGradient, CGGradientDrawingOptions};
use libc::{c_int, size_t};
use path::CGPathRef;
use std::os::raw::c_void;

use foreign_types::{ForeignType, ForeignTypeRef};
use geometry::{CGAffineTransform, CGRect};
use image::CGImage;
use std::cmp;
use std::ptr;
use std::slice;

use crate::context::{CGContext, CGContextRef};

foreign_type! {
    #[doc(hidden)]
    type CType = ::sys::CGLayer;
    fn drop = |cs| CGLayerRelease(cs);
    fn clone = |p| CGLayerRetain(p);
    pub struct CGLayer;
    pub struct CGLayerRef;
}

impl CGLayer {
    pub fn type_id() -> CFTypeID {
        unsafe { CGLayerGetTypeID() }
    }

    pub fn create_layer_with_context(context: &CGContextRef, size: CGSize) -> Self {
        unsafe {
            let result = CGLayerCreateWithContext(dbg!(context.as_ptr()), dbg!(size), ptr::null());
            assert!(!result.is_null());

            Self::from_ptr(result)
        }
    }
}

impl CGLayerRef {
    pub fn size(&self) -> CGSize {
        unsafe { CGLayerGetSize(self.as_ptr()) }
    }

    pub fn context(&self) -> CGContext {
        unsafe {
            let resut = CGLayerGetContext(self.as_ptr());
            assert!(!resut.is_null());
            CGContext::from_existing_context_ptr(resut)
        }
    }
}

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGLayerCreateWithContext(
        context: ::sys::CGContextRef,
        size: CGSize,
        auxiliaryInfo: CFDictionaryRef,
    ) -> ::sys::CGLayerRef;
    fn CGLayerRetain(c: ::sys::CGLayerRef) -> ::sys::CGLayerRef;
    fn CGLayerRelease(c: ::sys::CGLayerRef);
    fn CGLayerGetTypeID() -> CFTypeID;

    fn CGLayerGetSize(layer: ::sys::CGLayerRef) -> CGSize;
    fn CGLayerGetContext(layer: ::sys::CGLayerRef) -> ::sys::CGContextRef;

}
