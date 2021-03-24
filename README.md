# write_and_err

Lint for Substrate projects detecting if there is attempt to throw error after writing to storage.

As explained in more verbose way in `decl_lint!` in `src/write_and_error.rs`:

* issue we are addressing is that it's not allowed to throw error
after writing to storage.

Which issue is discussed in
https://github.com/paritytech/substrate/issues/8962
and follow up tickets/issues,
like
https://github.com/paritytech/substrate/issues/8975
.

This is WIP, therefore this lint has false positives, false negatives.
There are obviously ways to improve it, but we release this early version,
to know based on feedback, which "false" cases to cover first
or if it's useful lint at all.

In order to build this lint,
I constructed AST tree in file `inputs/pseudo_write_and_err_00/src/lib.rs` (TODO: make it an convenient URL after pushing)
to mimmic AST structures we try to catch.
It is build based on snipppet provided on https://github.com/paritytech/substrate/issues/8962#issuecomment-851923189 .

# how to run it

Lint is using dylint framework.

TODO: how to run it commands

# notes for developers

When "re-running" lint, it is not sufficient to `cargo clean` lint project directory.
You need also to `cargo clean` directory of project you are testing lint on.
(Now it make be understandable example in `Makefile` )
