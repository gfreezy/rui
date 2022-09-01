macro_rules! impl_method {
    ($ty:ty$(,)?  { $($method:item)+ } ) => {
        impl $ty { $($method)+ }
    };
    ( $ty:ty, $($more:ty),+$(,)? { $($method:item)+ } ) => {
        impl_method!($ty, { $($method)+ });
        impl_method!($($more),+, { $($method)+ });
    };
}
