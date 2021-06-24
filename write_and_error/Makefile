.PHONY: all

all:
	export origpwd=$$PWD; (cd inputs/pseudo_write_and_err_00/ && cargo clean && cd $$origpwd ) && cargo build && cargo dylint target/debug/libwrite_and_error@* --  --manifest-path=inputs/pseudo_write_and_err_00/Cargo.toml
