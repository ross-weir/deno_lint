// Copyright 2020 the Deno authors. All rights reserved. MIT license.
use super::Context;
use super::LintRule;
use crate::{scopes::BindingKind, swc_util::find_lhs_ids};

use swc_ecmascript::ast::AssignExpr;
use swc_ecmascript::visit::noop_visit_type;
use swc_ecmascript::visit::Node;
use swc_ecmascript::visit::Visit;

pub struct NoExAssign;

const CODE: &str = "no-ex-assign";
const MESSAGE: &str = "Reassigning exception parameter is not allowed";
const HINT: &str = "Use a different variable for the assignment";

impl LintRule for NoExAssign {
  fn new() -> Box<Self> {
    Box::new(NoExAssign)
  }

  fn tags(&self) -> &'static [&'static str] {
    &["recommended"]
  }

  fn code(&self) -> &'static str {
    CODE
  }

  fn lint_program(
    &self,
    context: &mut Context,
    program: &swc_ecmascript::ast::Program,
  ) {
    let mut visitor = NoExAssignVisitor::new(context);
    visitor.visit_program(program, program);
  }

  fn docs(&self) -> &'static str {
    r#"Disallows the reassignment of exception parameters 

There is generally no good reason to reassign an exception parameter.  Once
reassigned the code from that point on has no reference to the error anymore.
    
### Invalid:
```typescript
try {
  someFunc();
} catch (e) {
  e = true;
  // can no longer access the thrown error
}
```

### Valid:
```typescript
try {
  someFunc();
} catch (e) {
  const anotherVar = true;
}
```
"#
  }
}

struct NoExAssignVisitor<'c> {
  context: &'c mut Context,
}

impl<'c> NoExAssignVisitor<'c> {
  fn new(context: &'c mut Context) -> Self {
    Self { context }
  }
}

impl<'c> Visit for NoExAssignVisitor<'c> {
  noop_visit_type!();

  fn visit_assign_expr(&mut self, assign_expr: &AssignExpr, _node: &dyn Node) {
    let ids = find_lhs_ids(&assign_expr.left);

    for id in ids {
      let var = self.context.scope.var(&id);

      if let Some(var) = var {
        if let BindingKind::CatchClause = var.kind() {
          self.context.add_diagnostic_with_hint(
            assign_expr.span,
            CODE,
            MESSAGE,
            HINT,
          );
        }
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn no_ex_assign_valid() {
    assert_lint_ok! {
      NoExAssign,
      r#"
try {} catch { e = 1; }
try {} catch (ex) { something = 1; }
try {} catch (ex) { return 1; }
function foo() { try { } catch (e) { return false; } }
      "#,
    };
  }

  #[test]
  fn no_ex_assign_invalid() {
    assert_lint_err! {
      NoExAssign,
      r#"
try {} catch (e) { e = 1; }
try {} catch (ex) { ex = 1; }
try {} catch (ex) { [ex] = []; }
try {} catch (ex) { ({x: ex = 0} = {}); }
try {} catch ({message}) { message = 1; }
      "#: [
        {
          line: 2,
          col: 19,
          message: MESSAGE,
          hint: HINT,
        },
        {
          line: 3,
          col: 20,
          message: MESSAGE,
          hint: HINT,
        },
        {
          line: 4,
          col: 20,
          message: MESSAGE,
          hint: HINT,
        },
        {
          line: 5,
          col: 21,
          message: MESSAGE,
          hint: HINT,
        },
        {
          line: 6,
          col: 27,
          message: MESSAGE,
          hint: HINT,
        },
      ]
    }
  }
}
