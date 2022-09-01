# 0.7.0 (6. 6. 2022)
- Add new `#[into]` attribute for delegated function parameters. If specified, the parameter will be
converted using the `From` trait before being passed as an argument to the called function.
- Add new `#[try_from]` attribute to delegated functions. You can use it to convert the delegated
expression using the `TryFrom` trait. You can also use `#[try_from(unwrap)]` to unwrap the result of
the conversion.

# 0.6.2 (31. 1. 2022)
- Add new `#[await(true/false)]` attribute to delegated functions. You can use it to control whether
`.await` will be appended to the delegated expression. It will be generated by default if the delegation
method signature is `async`.

# 0.6.1 (25. 7. 2021)
- add support for `async` functions. The delegated call will now use `.await`.

# 0.6.0 (7. 7. 2021)
- add the option to specify inline expressions that will be used as arguments for the delegated call (https://github.com/kobzol/rust-delegate/pull/34)
- removed `append_args` attribute, which is superseded by the mentioned expression in signature support

# 0.5.2 (16. 6. 2021)
- add the `append_args` attribute to append attributes to delegated calls (https://github.com/kobzol/rust-delegate/pull/31)

# 0.5.1 (6. 1. 2021)
- fix breaking change caused by using `syn` private API (https://github.com/kobzol/rust-delegate/issues/28) 

# 0.5.0 (16. 11. 2020)
- `self` can now be used as the delegation target
- Rust 1.46 introduced a change that makes it a bit difficult to use `rust-delegate` implementations
generated by macros. If you have this use case, please use [this workaround](https://github.com/kobzol/rust-delegate/issues/25#issuecomment-716774685).