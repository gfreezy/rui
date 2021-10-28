#[macro_export]
macro_rules! clone {
    (@as_expr $e:expr) => { $e };

    ([$($var:ident),*] $cl:expr) => {{
        $(let $var = $var.clone();)*
            $cl
    }};
}
