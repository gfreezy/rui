macro_rules! impl_method {
    ($ty:ty$(,)?  { $($method:item)+ } ) => {
        impl $ty { $($method)+ }
    };
    ( $ty:ty, $($more:ty),+$(,)? { $($method:item)+ } ) => {
        impl_method!($ty, { $($method)+ });
        impl_method!($($more),+, { $($method)+ });
    };
}

macro_rules! impl_trait_method {
    ($trait:ty => $ty:ty$(,)?  { $($method:item)+ } ) => {
        impl $trait for $ty { $($method)+ }
    };
    ($trait:ty =>  $ty:ty, $($more:ty),+$(,)? { $($method:item)+ } ) => {
        impl_trait_method!($trait => $ty, { $($method)+ });
        impl_trait_method!($trait => $($more),+, { $($method)+ });
    };
}
