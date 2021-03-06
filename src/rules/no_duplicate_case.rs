// Copyright 2020 the Deno authors. All rights reserved. MIT license.
use super::Context;
use super::LintRule;
use std::collections::HashSet;
use swc_common::Spanned;
use swc_ecmascript::visit::noop_visit_type;
use swc_ecmascript::visit::Node;
use swc_ecmascript::visit::Visit;

pub struct NoDuplicateCase;

impl LintRule for NoDuplicateCase {
  fn new() -> Box<Self> {
    Box::new(NoDuplicateCase)
  }

  fn tags(&self) -> &'static [&'static str] {
    &["recommended"]
  }

  fn code(&self) -> &'static str {
    "no-duplicate-case"
  }

  fn lint_program(
    &self,
    context: &mut Context,
    program: &swc_ecmascript::ast::Program,
  ) {
    let mut visitor = NoDuplicateCaseVisitor::new(context);
    visitor.visit_program(program, program);
  }

  fn docs(&self) -> &'static str {
    r#"Disallows using the same case clause in a switch statement more than once

When you reuse a case test expression in a `switch` statement, the duplicate case will
never be reached meaning this is almost always a bug.
    
### Invalid:
```typescript
const someText = "a";
switch (someText) {
  case "a":
    break;
  case "b":
    break;
  case "a": // duplicate test expression
    break;
  default:
    break;
}
```

### Valid:
```typescript
const someText = "a";
switch (someText) {
  case "a":
    break;
  case "b":
    break;
  case "c":
    break;
  default:
    break;
}
```
"#
  }
}

struct NoDuplicateCaseVisitor<'c> {
  context: &'c mut Context,
}

impl<'c> NoDuplicateCaseVisitor<'c> {
  fn new(context: &'c mut Context) -> Self {
    Self { context }
  }
}

impl<'c> Visit for NoDuplicateCaseVisitor<'c> {
  noop_visit_type!();

  fn visit_switch_stmt(
    &mut self,
    switch_stmt: &swc_ecmascript::ast::SwitchStmt,
    _parent: &dyn Node,
  ) {
    // Works like in ESLint - by comparing text repr of case statement
    let mut seen: HashSet<String> = HashSet::new();

    for case in &switch_stmt.cases {
      if let Some(test) = &case.test {
        let span = test.span();
        let test_txt = self.context.source_map.span_to_snippet(span).unwrap();
        if !seen.insert(test_txt) {
          self.context.add_diagnostic_with_hint(
            span,
            "no-duplicate-case",
            "Duplicate values in `case` are not allowed",
            "Remove or rename the duplicate case clause",
          );
        }
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::test_util::*;

  #[test]
  fn no_duplicate_case_test() {
    assert_lint_err_on_line::<NoDuplicateCase>(
      r#"
const someText = "some text";
switch (someText) {
    case "a":
        break;
    case "b":
        break;
    case "a":
        break;
    default:
        break;
}
      "#,
      8,
      9,
    );
  }
}
