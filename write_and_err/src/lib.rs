#![feature(rustc_private)]
#![warn(unused_extern_crates)]
#![allow(unused_attributes)]
#![recursion_limit="256"]

dylint_linting::dylint_library!();

extern crate rustc_ast;
//extern crate rustc_ast_pretty;
//extern crate rustc_attr;
//extern crate rustc_data_structures;
extern crate rustc_errors;
extern crate rustc_hir;
//extern crate rustc_hir_pretty;
//extern crate rustc_index;
//extern crate rustc_infer;
//extern crate rustc_lexer;
extern crate rustc_lint;
extern crate rustc_middle;
//extern crate rustc_mir;
//extern crate rustc_parse;
//extern crate rustc_parse_format;
extern crate rustc_session;
extern crate rustc_span;
//extern crate rustc_target;
//extern crate rustc_trait_selection;
//extern crate rustc_typeck;

mod write_and_error;

#[no_mangle]
pub fn register_lints(_sess: &rustc_session::Session, lint_store: &mut rustc_lint::LintStore) {
    lint_store.register_lints(&[write_and_error::WRITE_AND_ERROR]);
    lint_store.register_early_pass(|| Box::new(write_and_error::WriteAndError));
    lint_store.register_late_pass(|| Box::new(write_and_error::WriteAndError));
}

#[test]
fn ui() {
    dylint_testing::ui_test(
        env!("CARGO_PKG_NAME"),
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("ui"),
    );
}
