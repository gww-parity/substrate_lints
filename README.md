Currently one lint:

* [write_and_err](write_and_err) , check it's [README.md](write_and_err/README.md)

[License](write_and_err/LICENSE)

Lints are made to be used with [Dylint] framework.
As this framework require lints to be compiled for each version of toolchain,
recommened workflow is:

* keep project directories of all lints you want to update regularily in one directory
* use [`dylints_updater`](https://github.com/gww-parity/dylints_updater) to rebuild and update them in batch

Using [`dylints_updater`](https://github.com/gww-parity/dylints_updater) will make life easier with updating lints (or adding new ones) to new compiler toolchains, so you can enjoy linting like:

```
cargo dylint write_and_error
```

or 

```
cargo dylint --all
```


[Dylint]: https://github.com/trailofbits/dylint
[`dylints_updater`]: https://github.com/gww-parity/dylints_updater
