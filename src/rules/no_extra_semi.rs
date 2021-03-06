// Copyright 2020 the Deno authors. All rights reserved. MIT license.
use super::Context;
use super::LintRule;
use swc_ecmascript::ast::{
  DoWhileStmt, EmptyStmt, ForInStmt, ForOfStmt, ForStmt, IfStmt, LabeledStmt,
  Stmt, WhileStmt, WithStmt,
};
use swc_ecmascript::visit::noop_visit_type;
use swc_ecmascript::visit::Node;
use swc_ecmascript::visit::Visit;

pub struct NoExtraSemi;

impl LintRule for NoExtraSemi {
  fn new() -> Box<Self> {
    Box::new(NoExtraSemi)
  }

  fn tags(&self) -> &'static [&'static str] {
    &["recommended"]
  }

  fn code(&self) -> &'static str {
    "no-extra-semi"
  }

  fn lint_program(
    &self,
    context: &mut Context,
    program: &swc_ecmascript::ast::Program,
  ) {
    let mut visitor = NoExtraSemiVisitor::new(context);
    visitor.visit_program(program, program);
  }

  fn docs(&self) -> &'static str {
    r#"Disallows the use of unnecessary semi-colons

Extra (and unnecessary) semi-colons can cause confusion when reading the code as
well as making the code less clean.
    
### Invalid:
```typescript
const x = 5;;

function foo() {};
```

### Valid:
```typescript
const x = 5;

function foo() {}
```
"#
  }
}

struct NoExtraSemiVisitor<'c> {
  context: &'c mut Context,
}

impl<'c> NoExtraSemiVisitor<'c> {
  fn new(context: &'c mut Context) -> Self {
    Self { context }
  }
}

impl<'c> Visit for NoExtraSemiVisitor<'c> {
  noop_visit_type!();

  fn visit_empty_stmt(&mut self, empty_stmt: &EmptyStmt, _parent: &dyn Node) {
    self.context.add_diagnostic_with_hint(
      empty_stmt.span,
      "no-extra-semi",
      "Unnecessary semicolon.",
      "Remove the extra (and unnecessary) semi-colon",
    );
  }

  fn visit_for_stmt(&mut self, for_stmt: &ForStmt, parent: &dyn Node) {
    if matches!(&*for_stmt.body, Stmt::Empty(_)) {
      if let Some(ref init) = for_stmt.init {
        swc_ecmascript::visit::visit_var_decl_or_expr(self, init, parent);
      }
      if let Some(ref test) = for_stmt.test {
        swc_ecmascript::visit::visit_expr(self, test, parent);
      }
      if let Some(ref update) = for_stmt.update {
        swc_ecmascript::visit::visit_expr(self, update, parent);
      }
    } else {
      swc_ecmascript::visit::visit_for_stmt(self, for_stmt, parent);
    }
  }

  fn visit_while_stmt(&mut self, while_stmt: &WhileStmt, parent: &dyn Node) {
    if matches!(&*while_stmt.body, Stmt::Empty(_)) {
      swc_ecmascript::visit::visit_expr(self, &*while_stmt.test, parent);
    } else {
      swc_ecmascript::visit::visit_while_stmt(self, while_stmt, parent);
    }
  }

  fn visit_do_while_stmt(
    &mut self,
    do_while_stmt: &DoWhileStmt,
    parent: &dyn Node,
  ) {
    if matches!(&*do_while_stmt.body, Stmt::Empty(_)) {
      swc_ecmascript::visit::visit_expr(self, &*do_while_stmt.test, parent);
    } else {
      swc_ecmascript::visit::visit_do_while_stmt(self, do_while_stmt, parent);
    }
  }

  fn visit_with_stmt(&mut self, with_stmt: &WithStmt, parent: &dyn Node) {
    if matches!(&*with_stmt.body, Stmt::Empty(_)) {
      swc_ecmascript::visit::visit_expr(self, &*with_stmt.obj, parent);
    } else {
      swc_ecmascript::visit::visit_with_stmt(self, with_stmt, parent);
    }
  }

  fn visit_for_of_stmt(&mut self, for_of_stmt: &ForOfStmt, parent: &dyn Node) {
    if matches!(&*for_of_stmt.body, Stmt::Empty(_)) {
      swc_ecmascript::visit::visit_var_decl_or_pat(
        self,
        &for_of_stmt.left,
        parent,
      );
      swc_ecmascript::visit::visit_expr(self, &*for_of_stmt.right, parent);
    } else {
      swc_ecmascript::visit::visit_for_of_stmt(self, for_of_stmt, parent);
    }
  }

  fn visit_for_in_stmt(&mut self, for_in_stmt: &ForInStmt, parent: &dyn Node) {
    if matches!(&*for_in_stmt.body, Stmt::Empty(_)) {
      swc_ecmascript::visit::visit_var_decl_or_pat(
        self,
        &for_in_stmt.left,
        parent,
      );
      swc_ecmascript::visit::visit_expr(self, &*for_in_stmt.right, parent);
    } else {
      swc_ecmascript::visit::visit_for_in_stmt(self, for_in_stmt, parent);
    }
  }

  fn visit_if_stmt(&mut self, if_stmt: &IfStmt, parent: &dyn Node) {
    swc_ecmascript::visit::visit_expr(self, &*if_stmt.test, parent);
    match &*if_stmt.cons {
      Stmt::Empty(_) => {}
      cons => {
        swc_ecmascript::visit::visit_stmt(self, cons, parent);
      }
    }
    match if_stmt.alt.as_deref() {
      None | Some(Stmt::Empty(_)) => {}
      Some(alt) => {
        swc_ecmascript::visit::visit_stmt(self, alt, parent);
      }
    }
  }

  fn visit_labeled_stmt(
    &mut self,
    labeled_stmt: &LabeledStmt,
    parent: &dyn Node,
  ) {
    swc_ecmascript::visit::visit_ident(self, &labeled_stmt.label, parent);
    match &*labeled_stmt.body {
      Stmt::Empty(_) => {}
      body => {
        swc_ecmascript::visit::visit_stmt(self, body, parent);
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::test_util::*;

  #[test]
  fn no_extra_semi_valid() {
    assert_lint_ok! {
      NoExtraSemi,
      "var x = 5;",
      "function foo(){}",
      "for(;;);",
      "while(0);",
      "do;while(0);",
      "for(a in b);",
      "for(a of b);",
      "if(true);",
      "if(true); else;",
      "foo: ;",
      "foo: bar: ;",
      "with(foo);",
      "class A { }",
      "var A = class { };",
      "class A { a() { this; } }",
      "var A = class { a() { this; } };",
      "class A { } a;",
    };
  }

  #[test]
  fn no_extra_semi_invalid() {
    assert_lint_err::<NoExtraSemi>("var x = 5;;", 10);
    assert_lint_err::<NoExtraSemi>("function foo(){};", 16);
    assert_lint_err::<NoExtraSemi>("for(;;);;", 8);
    assert_lint_err::<NoExtraSemi>("while(0);;", 9);
    assert_lint_err::<NoExtraSemi>("do;while(0);;", 12);
    assert_lint_err::<NoExtraSemi>("for(a in b);;", 12);
    assert_lint_err::<NoExtraSemi>("for(a of b);;", 12);
    assert_lint_err::<NoExtraSemi>("if(true);;", 9);
    assert_lint_err::<NoExtraSemi>("if(true){} else;;", 16);
    assert_lint_err_n::<NoExtraSemi>("if(true){;} else {;}", vec![9, 18]);
    assert_lint_err::<NoExtraSemi>("foo:;;", 5);
    assert_lint_err::<NoExtraSemi>("with(foo);;", 10);
    assert_lint_err::<NoExtraSemi>("with(foo){;}", 10);
    assert_lint_err::<NoExtraSemi>("class A { ; }", 10);
    assert_lint_err::<NoExtraSemi>("class A { /*a*/; }", 15);
    assert_lint_err::<NoExtraSemi>("class A { ; a() {} }", 10);
    assert_lint_err::<NoExtraSemi>("class A { a() {}; }", 16);
    assert_lint_err::<NoExtraSemi>("class A { a() {}; b() {} }", 16);
    assert_lint_err_n::<NoExtraSemi>(
      "class A {; a() {}; b() {}; }",
      vec![9, 17, 25],
    );
    assert_lint_err::<NoExtraSemi>("class A { a() {}; get b() {} }", 16);

    assert_lint_err_on_line::<NoExtraSemi>(
      r#"
for (let i = 0; i < n; i++) {
  for (;;);;
}
"#,
      3,
      11,
    );
    assert_lint_err_on_line::<NoExtraSemi>(
      r#"
while (a) {
  while (b);;
}
"#,
      3,
      12,
    );
    assert_lint_err_on_line::<NoExtraSemi>(
      r#"
do {
  do {
    ;
  } while(a);
} while(b);
"#,
      4,
      4,
    );
    assert_lint_err_on_line::<NoExtraSemi>(
      r#"
with(a) {
  with(b) {
    ;
  }
}
"#,
      4,
      4,
    );
    assert_lint_err_on_line::<NoExtraSemi>(
      r#"
for (const a of b) {
  for (const c of d) {
    ;
  }
}
"#,
      4,
      4,
    );
    assert_lint_err_on_line::<NoExtraSemi>(
      r#"
for (const a in b) {
  for (const c in d) {
    ;
  }
}
"#,
      4,
      4,
    );
    assert_lint_err_on_line::<NoExtraSemi>(
      r#"
if (a) {
  if (b) {
    ;
  } else;
}
"#,
      4,
      4,
    );
    assert_lint_err_on_line::<NoExtraSemi>(
      r#"
foo: {
  bar: {
    ;
  }
}
"#,
      4,
      4,
    );
    assert_lint_err_on_line::<NoExtraSemi>(
      r#"
class A {
  foo() {
    class B { ; }
  }
}
"#,
      4,
      14,
    );
  }
}
