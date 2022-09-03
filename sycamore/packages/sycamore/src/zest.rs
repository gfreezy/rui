//! HTML tag definitions.
//!
//! _Documentation sources: <https://developer.mozilla.org/en-US/>_

use crate::builder::ElementBuilder;
use crate::generic_node::SycamoreElement;
use crate::prelude::*;

/// MBE for generating elements.
macro_rules! define_elements {
    (
        $ns:expr,
        $(
            $(#[$attr:meta])*
            $el:ident {
                $(
                    $(#[$attr_method:meta])*
                    $at:ident: $ty:path,
                )*
            },
        )*
    ) => {
        $(
            #[allow(non_camel_case_types)]
            #[doc = concat!("Build a [`<", stringify!($el), ">`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/", stringify!($el), ") element.")]
            $(#[$attr])*
            #[derive(Debug)]
            pub struct $el {}

            impl SycamoreElement for $el {
                const TAG_NAME: &'static str = stringify!($el);
                const NAME_SPACE: Option<&'static str> = $ns;
            }

            #[allow(non_snake_case)]
            #[doc = concat!("Create a [`<", stringify!($el), ">`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/", stringify!($el), ") element builder.")]
            $(#[$attr])*
            pub fn $el<'a, G: GenericNode>() -> ElementBuilder<'a, G, impl FnOnce(Scope<'a>) -> G> {
                ElementBuilder::new(move |_| G::element::<$el>())
            }
        )*
    };
}

// A list of valid HTML5 elements (does not include removed or obsolete elements).
define_elements! {
    None,
    text {},
    listener {},
    flex {},
}
