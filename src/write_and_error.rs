#![allow(unused_attributes)]
#![allow(unused_imports)]
use clippy_utils::diagnostics::{span_lint, span_lint_and_sugg, span_lint_and_then};
use clippy_utils::source::snippet;
use clippy_utils::source::snippet_with_applicability;
use if_chain::if_chain;
use rustc_ast::ast;
use rustc_ast::node_id::NodeId;
use rustc_ast::visit as ast_visit;
use rustc_ast::visit::FnKind;
use rustc_ast::visit::Visitor as AstVisitor;
use rustc_errors::Applicability;
use rustc_hir as hir;
use rustc_hir::intravisit as hir_visit;
use rustc_hir::intravisit::Visitor as HirVisitor;
use rustc_lint::{EarlyContext, EarlyLintPass, LateContext, LateLintPass, LintContext};
use rustc_middle::hir::map::Map;
use rustc_middle::lint::in_external_macro;
use rustc_session::{declare_lint, declare_lint_pass};
use rustc_span::source_map::Span;

declare_lint! {
    /// **What it does:** Lint for Substrate projects detecting if there is attempt to throw error after writing to storage.
    /// Looks for situations, when error
    /// is returned after write to storage
    /// (it's early version WIP with false positives, and false negatives).
    ///
    ///
    /// **Why is this bad?** Runtimes should be idempotent, returning error
    /// after storage makes it questionable if `#[transactional]` should be used for extrinsics instead.
    /// (More considerations about transactional https://github.com/paritytech/substrate/issues/8975 )
    /// 
    ///
    /// **Known problems:** Throwing error without using `#[transactional]` may break idempotence,
    /// while using it may increase cost too high. Please note this lint is early stage WIP,
    /// therefore has false positives, false negatives, you still need to look after problem yourself.
    ///
    /// **Example:**
    /// ```rust,ignore
    /// // Bad
    /// pub fn xyz_bad(origin: OriginFor<T>) -> DispatchResult {
    ///   xyz::<T>::put(true);
    ///   ensure_root(origin)?;
    ///   // this pattern is wrong because we could both change storage and return an error
    /// }
    ///
    /// // Good
    /// #[transactional]
    /// pub fn xyz_transactional(origin: OriginFor<T>) -> DispatchResult {
    ///   xyz::<T>::put(true);
    ///   ensure_root(origin)?;
    ///   // this pattern is wrong because we could both change storage and return an error
    /// }
    ///
    /// ```
    pub WRITE_AND_ERROR,
    Warn,
    "throwaway closures called in the expression they are defined"
}

declare_lint_pass!(WriteAndError => [WRITE_AND_ERROR]);

// Used to find `return` statements or equivalents e.g., `?`
#[allow(dead_code)]
struct ReturnVisitor {
    found_return: bool,
}

#[allow(dead_code)]
impl ReturnVisitor {
    #[must_use]
    fn new() -> Self {
        Self {
            found_return: false,
        }
    }
}

impl<'ast> ast_visit::Visitor<'ast> for ReturnVisitor {
    fn visit_expr(&mut self, ex: &'ast ast::Expr) {
        if let ast::ExprKind::Ret(_) = ex.kind {
            self.found_return = true;
        } else if let ast::ExprKind::Try(_) = ex.kind {
            self.found_return = true;
        }

        ast_visit::walk_expr(self, ex)
    }
}

// Used to find some "write to storage" (i.e. "put") call followed by "?" error
// propagation statement. (TODO: error return i.e. e.g. `?` explicit `return`
// and what function returns at the end... but it may require late pass type checking)

//#[derive(Clone,Copy,Debug,Eq,Ord,PartialEq, PartialOrd)]
#[derive(Clone, Copy, Debug)]
pub struct Match<'ast> {
    span: Span,
    expr_kind: &'ast ast::ExprKind,
}

impl<'ast> Match<'ast> {
    fn new(span: Span, expr_kind: &'ast ast::ExprKind) -> Self {
        Match { span, expr_kind }
    }
}

pub struct CollectCallsAndRetsVisitor<'ast> {
    found_return: bool,
    returns: Vec<Match<'ast>>,
}

impl<'ast> CollectCallsAndRetsVisitor<'ast> {
    #[must_use]
    fn new() -> Self {
        Self {
            found_return: false,
            returns: Vec::new(),
        }
    }
}

impl<'ast> ast_visit::Visitor<'ast> for CollectCallsAndRetsVisitor<'ast> {
    fn visit_expr(&mut self, ex: &'ast ast::Expr) {
        let does_it_match = match ex.kind {
            ast::ExprKind::Ret(_) => true,
            ast::ExprKind::Try(_) => true,
            ast::ExprKind::Call(ref _p, _) => {
                // TODO: consider filtering for "put" here
                true
            }
            ast::ExprKind::MethodCall(ref _path_segment, ref _p, ref _span) => {
                // TODO: consider filtering for "put" here
                true
            }
            _ => false,
        };
        if does_it_match == true {
            self.found_return = true;
            self.returns.push(Match::new(ex.span, &ex.kind));
        }

        ast_visit::walk_expr(self, ex)
    }
}

impl EarlyLintPass for WriteAndError {
    fn check_fn(
        &mut self,
        cx: &EarlyContext<'_>,
        fn_kind: FnKind<'_>,
        span: Span,
        node_id: NodeId,
    ) {
        if let FnKind::Fn(_, _, _, _, Some(block)) = fn_kind {
            //let mut count_stmts = 0;
            //let mut stmts_str = String::from("");

            /*
            for _stmt in &block.stmts {
                stmts_str += "A";
                count_stmts += 1;
            }
            */

            let mut visitor = CollectCallsAndRetsVisitor::new();
            visitor.visit_block(block);
            //let mut returns_str = String::from("");

            let mut earliest_potential_write_to_storage_encountered = Option::<Span>::None;
            let mut latest_potential_error_return_encountared = Option::<Span>::None;

            for m in visitor.returns {
                use ast::ExprKind::*;
                match m.expr_kind {
                    Call(_, _) => {
                        if let None = earliest_potential_write_to_storage_encountered {
                            let snip = snippet(cx, m.span, "<>");
                            if snip.contains("put") {
                                //TODO: improve this matching
                                earliest_potential_write_to_storage_encountered = Some(m.span);
                            }
                        }
                    }
                    Try(_) => {
                        // TODO: check if they return error
                        latest_potential_error_return_encountared = Some(m.span);
                    }
                    Ret(_) => {
                        // TODO: check if they return error
                        latest_potential_error_return_encountared = Some(m.span);
                    }
                    // TODO: MethodCall(_)
                    _ => {}
                };
                //returns_str.push_str(&(snippet(cx, m.span, "<>").to_string() + ";"));
            }

            if let Some(potwrite) = earliest_potential_write_to_storage_encountered {
                if let Some(poterr) = latest_potential_error_return_encountared {
                    if potwrite.lo() < poterr.lo() {
                        //let fn_snippet = snippet(cx, span, "<body function>");
                        // TODO: check if multiple span snippet would work to point potential write and error spans
                        span_lint(
                            cx,
                            WRITE_AND_ERROR,
                            span,
                            &format!(
                                "check_fn match! (fn_kind: {:?}, node_id: {:?} visitor.found_return={:?} write=`{:?}` err=`{:?}`",
                                "Fn", 
                                node_id,
                                visitor.found_return,
                                earliest_potential_write_to_storage_encountered,
                                latest_potential_error_return_encountared
                            ),
                        );
                    }
                }
            }
        }
        /*
        fn check_expr(&mut self, cx: &EarlyContext<'_>, expr: &ast::Expr) {
            if in_external_macro(cx.sess(), expr.span) {
                return;
            }
            if_chain! {
                if let ast::ExprKind::Call(ref paren, _) = expr.kind;
                if let ast::ExprKind::Paren(ref closure) = paren.kind;
                if let ast::ExprKind::Closure(_, _, _, ref decl, ref block, _) = closure.kind;
                then {
                    let mut visitor = ReturnVisitor::new();
                    visitor.visit_expr(block);
                    if !visitor.found_return {
                        span_lint_and_then(
                            cx,
                            WRITE_AND_ERROR,
                            expr.span,
                            "try not to call a closure in the expression where it is declared",
                            |diag| {
                                if decl.inputs.is_empty() {
                                    let mut app = Applicability::MachineApplicable;
                                    let hint =
                                        snippet_with_applicability(cx, block.span, "..", &mut app).into_owned();
                                    diag.span_suggestion(expr.span, "try doing something like", hint, app);
                                }
                            },
                        );
                    }
                }
            }
        }
        */
        /*
        fn check_expr(&mut self, cx: &EarlyContext<'_>, expr: &ast::Expr) {
            span_lint(
                    cx,
                    WRITE_AND_ERROR,
                    expr.span,
                    &format!("check_expr match! (expr_kind: {:?}", expr.kind)
                    );
        }
        */
    }
}

impl<'tcx> LateLintPass<'tcx> for WriteAndError {
    /*
    fn check_block(&mut self, cx: &LateContext<'tcx>, block: &'tcx hir::Block<'_>) {
        fn count_closure_usage<'a, 'tcx>(
            cx: &'a LateContext<'tcx>,
            block: &'tcx hir::Block<'_>,
            path: &'tcx hir::Path<'tcx>,
        ) -> usize {
            struct ClosureUsageCount<'a, 'tcx> {
                cx: &'a LateContext<'tcx>,
                path: &'tcx hir::Path<'tcx>,
                count: usize,
            }
            impl<'a, 'tcx> hir_visit::Visitor<'tcx> for ClosureUsageCount<'a, 'tcx> {
                type Map = Map<'tcx>;

                fn visit_expr(&mut self, expr: &'tcx hir::Expr<'tcx>) {
                    if_chain! {
                        if let hir::ExprKind::Call(ref closure, _) = expr.kind;
                        if let hir::ExprKind::Path(hir::QPath::Resolved(_, ref path)) = closure.kind;
                        if self.path.segments[0].ident == path.segments[0].ident
                            && self.path.res == path.res;
                        then {
                            self.count += 1;
                        }
                    }
                    hir_visit::walk_expr(self, expr);
                }

                fn nested_visit_map(&mut self) -> hir_visit::NestedVisitorMap<Self::Map> {
                    hir_visit::NestedVisitorMap::OnlyBodies(self.cx.tcx.hir())
                }
            }
            let mut closure_usage_count = ClosureUsageCount { cx, path, count: 0 };
            closure_usage_count.visit_block(block);
            closure_usage_count.count
        }

        for w in block.stmts.windows(2) {
            if_chain! {
                if let hir::StmtKind::Local(ref local) = w[0].kind;
                if let Option::Some(ref t) = local.init;
                if let hir::ExprKind::Closure(..) = t.kind;
                if let hir::PatKind::Binding(_, _, ident, _) = local.pat.kind;
                if let hir::StmtKind::Semi(ref second) = w[1].kind;
                if let hir::ExprKind::Assign(_, ref call, _) = second.kind;
                if let hir::ExprKind::Call(ref closure, _) = call.kind;
                if let hir::ExprKind::Path(hir::QPath::Resolved(_, ref path)) = closure.kind;
                if ident == path.segments[0].ident;
                if count_closure_usage(cx, block, path) == 1;
                then {
                    span_lint(
                        cx,
                        WRITE_AND_ERROR,
                        second.span,
                        "closure called just once immediately after it was declared",
                    );
                }
            }
        }
    }
    */
}
