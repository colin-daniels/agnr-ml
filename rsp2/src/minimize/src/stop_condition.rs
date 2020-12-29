/* ************************************************************************ **
** This file is part of rsp2, and is licensed under EITHER the MIT license  **
** or the Apache 2.0 license, at your option.                               **
**                                                                          **
**     http://www.apache.org/licenses/LICENSE-2.0                           **
**     http://opensource.org/licenses/MIT                                   **
**                                                                          **
** Be aware that not all of rsp2 is provided under this permissive license, **
** and that the project as a whole is licensed under the GPL 3.0.           **
** ************************************************************************ */

//! Exposes a serializable expression language for algorithmic stop conditions.

pub mod prelude {
    pub use super::ShouldStop;
}

/// Generic trait for stop conditions.
///
/// It is expected that little code should depend on this for generic types;
/// its raison d'etre is to provide a bound on `Rpn`.
pub trait ShouldStop<T> {
    fn should_stop(&self, x: &T) -> bool;
}

/// Represents logical expressions of T in JSON.
///
/// Where `(T)` stands in for a valid JSON representation of `T`,
/// the accepted forms of `LogicalExpressions<T>` are as follows:
///
///  - `{"any": [(T), ...]}` - a logical-or of 0 or more expressions
///  - `{"all": [(T), ...]}` - a logical-and of 0 or more expressions
///
/// This is often used through the `Cereal` struct, where these
/// variants will appear untagged alongside valid representations of `T`.
/// Hence, one should be cautious about adding further variants.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[derive(Serialize, Deserialize)]
pub enum LogicalExpression<T> {
    /// Succeed as soon as any of the subconditions are met.
    #[serde(rename = "any")]
    Any(Vec<T>),

    /// Succeed when all subconditions are simultaneously met.
    #[serde(rename = "all")]
    All(Vec<T>),
}

/// Type that stop condition config can deserialize directly into.
///
/// This extends a simple predicate type (represented as an object
/// with a single kv pair) with `"all"`, `"any"`, `true`, and `false` variants.
#[derive(Debug, Clone, PartialEq, Eq)]
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum Cereal<P> {
    Simple(P),
    Const(bool),
    Logical(LogicalExpression<Cereal<P>>),
}

/// A composite stop condition expressed in reverse-Polish notation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rpn<P>(Vec<Action<P>>);

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Action<P> {
    /// Push a constant onto the stack
    Constant(bool),
    /// Test a single objective and push the result onto the stack
    Predicate(P),
    /// Pop two items and compute their logical 'or'
    Or,
    /// Pop two items and compute their logical 'and'
    And,
}

impl<P: Clone> Rpn<P> {
    pub fn from_cereal(cereal: &Cereal<P>) -> Self {
        fn append_actions<Q: Clone>(out: &mut Vec<Action<Q>>, cereal: &Cereal<Q>) {
            match cereal {
                Cereal::Const(x) => {
                    out.push(Action::Constant(*x));
                },
                Cereal::Simple(x) => {
                    out.push(Action::Predicate(x.clone()));
                },
                Cereal::Logical(LogicalExpression::Any(xs)) => {
                    append_fold(out, &xs, Action::Constant(false), Action::Or);
                },
                Cereal::Logical(LogicalExpression::All(xs)) =>  {
                    append_fold(out, &xs, Action::Constant(true), Action::And);
                },
            }
        }

        fn append_fold<Q: Clone>(
            out: &mut Vec<Action<Q>>,
            seq: &[Cereal<Q>],
            identity: Action<Q>,
            operator: Action<Q>,
        ) {
            out.push(identity);
            for x in seq {
                append_actions(out, x);
                out.push(operator.clone());
            }
        }

        let mut out = vec![];
        append_actions(&mut out, cereal);
        Rpn(out)
    }
}

impl<T, P> ShouldStop<T> for Rpn<P>
where P: ShouldStop<T>,
{
    fn should_stop(&self, x: &T) -> bool {
        let mut stack = vec![];
        for act in &self.0 {
            let b = match *act {
                Action::Constant(b) => b,
                Action::Predicate(ref cond) => cond.should_stop(x),
                Action::Or  => stack.pop().unwrap() | stack.pop().unwrap(),
                Action::And => stack.pop().unwrap() & stack.pop().unwrap(),
            };
            stack.push(b);
        }
        assert_eq!(stack.len(), 1);
        stack[0]
    }
}

mod tests {
    // High level "is it broken?" test of stop conditions that checks:
    ///   * deserialization;
    ///   * conversion into runtime representation;
    ///   * and execution
    /// without much thought for edge-cases.
    #[test]
    fn test_stop_condition() {
        use crate::stop_condition::prelude::*;
        use crate::cg::stop_condition::Objectives;

        // (value that has at least one of each logical expression type)
        let cereal: crate::cg::stop_condition::StopCondition = from_json!(
            {"any": [
                {"all": [
                    {"grad-max": 1.0},
                    {"grad-rms": 1.0},
                ]},
                {"iterations": 100},
            ]}
        );

        // conversion...
        let pred = super::Rpn::from_cereal(&cereal);

        // execution...
        // (base input which fails all conditions)
        let base = Objectives {
            grad_max: 2.0,
            grad_norm: 2.0,
            grad_rms: 2.0,
            values: &[],
            iterations: 0,
        };

        // (F && T) || F
        let objs = Objectives { grad_rms: 0.5, ..base };
        assert!(!pred.should_stop(&objs));

        // (T && F) || F
        let objs = Objectives { grad_max: 0.5, ..base };
        assert!(!pred.should_stop(&objs));

        // (T && T) || F
        let objs = Objectives { grad_rms: 0.5, grad_max: 0.5, ..base };
        assert!( pred.should_stop(&objs));

        // (F && F) || T
        let objs = Objectives { iterations: 200, ..base };
        assert!( pred.should_stop(&objs));
    }
}
