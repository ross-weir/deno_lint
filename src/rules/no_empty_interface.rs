// Copyright 2020 the Deno authors. All rights reserved. MIT license.
use super::Context;
use super::LintRule;
use swc_ecmascript::ast::TsInterfaceDecl;
use swc_ecmascript::visit::Node;
use swc_ecmascript::visit::Visit;

pub struct NoEmptyInterface;

impl LintRule for NoEmptyInterface {
  fn new() -> Box<Self> {
    Box::new(NoEmptyInterface)
  }

  fn tags(&self) -> &'static [&'static str] {
    &["recommended"]
  }

  fn code(&self) -> &'static str {
    "no-empty-interface"
  }

  fn lint_program(
    &self,
    context: &mut Context,
    program: &swc_ecmascript::ast::Program,
  ) {
    let mut visitor = NoEmptyInterfaceVisitor::new(context);
    visitor.visit_program(program, program);
  }

  fn docs(&self) -> &'static str {
    r#"Disallows the declaration of an empty interface

An interface with no members serves no purpose.  Either the interface extends
another interface, in which case the supertype can be used, or it does not
extend a supertype in which case it is the equivalent to an empty object.  This
rule will capture these situations as either unnecessary code or a mistaken
empty implementation.
    
### Invalid:
```typescript
interface Foo {}
interface Foo extends Bar {}
```

### Valid:
```typescript
interface Foo {
  name: string;
}

interface Bar {
  age: number;
}

// Using an empty interface as a union type is allowed
interface Baz extends Foo, Bar {}
```
"#
  }
}

struct NoEmptyInterfaceVisitor<'c> {
  context: &'c mut Context,
}

impl<'c> NoEmptyInterfaceVisitor<'c> {
  fn new(context: &'c mut Context) -> Self {
    Self { context }
  }
}

impl<'c> Visit for NoEmptyInterfaceVisitor<'c> {
  fn visit_ts_interface_decl(
    &mut self,
    interface_decl: &TsInterfaceDecl,
    _parent: &dyn Node,
  ) {
    if interface_decl.extends.len() <= 1 && interface_decl.body.body.is_empty()
    {
      self.context.add_diagnostic_with_hint(
        interface_decl.span,
        "no-empty-interface",
        if interface_decl.extends.is_empty() {
          "An empty interface is equivalent to `{}`."
        } else {
          "An interface declaring no members is equivalent to its supertype."
        },
        if interface_decl.extends.is_empty() {
          "Remove this interface or add members to this interface."
        } else {
          "Use the supertype instead, or add members to this interface."
        },
      );
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::test_util::*;

  #[test]
  fn no_empty_interface_valid() {
    assert_lint_ok! {
      NoEmptyInterface,
      "interface Foo { a: string }",
      "interface Foo { a: number }",

      // This is valid because an interface with more than one supertype
      // can be used as a replacement of a union type.
      "interface Foo extends Bar, Baz {}",
    };
  }

  #[test]
  fn no_empty_interface_invalid() {
    assert_lint_err::<NoEmptyInterface>("interface Foo {}", 0);
    assert_lint_err::<NoEmptyInterface>("interface Foo extends {}", 0);
    assert_lint_err_on_line::<NoEmptyInterface>(
      r#"
interface Foo {
  a: string;
}

interface Bar extends Foo {}
"#,
      6,
      0,
    );
    assert_lint_err::<NoEmptyInterface>(
      "interface Foo extends Array<number> {}",
      0,
    );
    assert_lint_err::<NoEmptyInterface>(
      "interface Foo extends Array<number | {}> {}",
      0,
    );
    assert_lint_err_on_line::<NoEmptyInterface>(
      r#"
interface Foo {
  a: string;
}

interface Bar extends Array<Foo> {}
"#,
      6,
      0,
    );
    assert_lint_err_on_line::<NoEmptyInterface>(
      r#"
type R = Record<string, unknown>;
interface Foo extends R {}
"#,
      3,
      0,
    );
    assert_lint_err::<NoEmptyInterface>(
      "interface Foo<T> extends Bar<T> {}",
      0,
    );
    assert_lint_err_on_line::<NoEmptyInterface>(
      r#"
declare module FooBar {
  type Baz = typeof baz;
  export interface Bar extends Baz {}
}
"#,
      4,
      9,
    );
  }
}
