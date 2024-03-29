use std::fmt::Display;

use anyhow::Result;

use crate::traits::{act::Act, configurable::Configurable, durative::Durative, interpreter::Interpreter};

use super::{
    condition::{DurativeCondition, SpanCondition},
    effects::{DurativeEffect, SpanEffect},
    env::Env,
    parameter::Parameter,
    state::State,
    time::{TemporalInterval, TemporalIntervalExpression, Timepoint},
    value::Value,
};

/* ========================================================================== */
/*                             Action Enumeration                             */
/* ========================================================================== */

/// Represents a span or a durative action.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action<E> {
    Span(SpanAction<E>),
    Durative(DurativeAction<E>),
}

impl<E: Clone + Interpreter> Action<E> {
    pub fn into_durative(actions: &[Action<E>]) -> Vec<DurativeAction<E>> {
        let mut c = 0;
        actions
            .iter()
            .map(|a| match a {
                Action::Span(s) => {
                    c += 2;
                    DurativeAction::new(
                        s.name().to_string(),
                        s.id().to_string(),
                        s.params().to_vec(),
                        s.conditions()
                            .iter()
                            .map(|c| DurativeCondition::from_span(c.clone(), TemporalInterval::overall()))
                            .collect::<Vec<_>>(),
                        s.effects()
                            .iter()
                            .map(|e| DurativeEffect::from_span(e.clone(), Timepoint::at_end()))
                            .collect::<Vec<_>>(),
                        Timepoint::fixed((c - 2).into()),
                        Timepoint::fixed((c - 1).into()),
                        None,
                    )
                }
                Action::Durative(d) => d.clone(),
            })
            .collect::<Vec<_>>()
    }
}

impl<E> From<SpanAction<E>> for Action<E> {
    fn from(a: SpanAction<E>) -> Self {
        Action::Span(a)
    }
}

impl<E> From<DurativeAction<E>> for Action<E> {
    fn from(a: DurativeAction<E>) -> Self {
        Action::Durative(a)
    }
}

/* ========================================================================== */
/*                                 Base Action                                */
/* ========================================================================== */

/// Common parts of a SpanAction and a DurativeAction.
#[derive(Clone, Debug, PartialEq, Eq)]
struct BaseAction {
    /// The name of the action.
    name: String,
    /// The identifier of the action that might be used to refer to it (e.g. in HTN plans).
    id: String,
    /// The parameters of the action.
    params: Vec<Parameter>,
}

impl<E: Clone> Configurable<E> for BaseAction {
    fn params(&self) -> &[Parameter] {
        self.params.as_ref()
    }
}

/* ========================================================================== */
/*                                 Span Action                                */
/* ========================================================================== */

#[derive(Clone, Debug, PartialEq, Eq)]
/// Representation of a span action for the validation.
pub struct SpanAction<E> {
    /// The common parts of a span and a durative action.
    base: BaseAction,
    /// The list of conditions for the application of the action.
    conditions: Vec<SpanCondition<E>>,
    /// The list of effects.
    effects: Vec<SpanEffect<E>>,
}

impl<E> SpanAction<E> {
    pub fn new(
        name: String,
        id: String,
        params: Vec<Parameter>,
        conditions: Vec<SpanCondition<E>>,
        effects: Vec<SpanEffect<E>>,
    ) -> Self {
        Self {
            base: BaseAction { name, id, params },
            conditions,
            effects,
        }
    }

    /// Returns the name of the action.
    pub fn name(&self) -> &String {
        &self.base.name
    }

    /// Returns the id of the action.
    pub fn id(&self) -> &String {
        &self.base.id
    }

    /// Returns the list of effects of the action.
    pub fn effects(&self) -> &Vec<SpanEffect<E>> {
        &self.effects
    }

    /// Add a new condition to the action.
    pub fn add_condition(&mut self, value: SpanCondition<E>) {
        self.conditions.push(value)
    }

    /// Add a new effect to the action.
    pub fn add_effect(&mut self, value: SpanEffect<E>) {
        self.effects.push(value)
    }
}

impl<E: Clone> Configurable<E> for SpanAction<E> {
    fn params(&self) -> &[Parameter] {
        self.base.params.as_ref()
    }
}

impl<E: Clone + Interpreter> Act<E> for SpanAction<E> {
    fn conditions(&self) -> &Vec<SpanCondition<E>> {
        &self.conditions
    }

    fn applicable(&self, env: &Env<E>) -> Result<bool> {
        let new_env = self.new_env_with_params(env);
        // Check the conditions.
        for c in self.conditions() {
            if !c.is_valid(&new_env)? {
                return Ok(false);
            }
        }
        // Check that two effects don't affect the same fluent.
        let mut changes: Vec<Vec<Value>> = vec![];
        for e in self.effects.iter() {
            if let Some((f, _)) = e.changes(&new_env)? {
                if changes.contains(&f) {
                    return Ok(false);
                }
                changes.push(f);
            }
        }
        Ok(true)
    }

    fn apply(&self, env: &Env<E>, s: &State) -> Result<Option<State>> {
        let new_env = self.new_env_with_params(env);
        if !self.applicable(&new_env)? {
            return Ok(None);
        }
        let mut new_s = s.clone();
        for e in self.effects.iter() {
            if let Some(s) = e.apply(&new_env, &new_s)? {
                new_s = s;
            }
        }
        Ok(Some(new_s))
    }
}

/* ========================================================================== */
/*                               Durative Action                              */
/* ========================================================================== */

#[derive(Clone, Debug, PartialEq, Eq)]
/// Representation of  a durative action for the validation.
pub struct DurativeAction<E> {
    /// The common parts of a span and a durative action.
    base: BaseAction,
    /// The list of conditions for the application of the action.
    conditions: Vec<DurativeCondition<E>>,
    /// The list of effects.
    effects: Vec<DurativeEffect<E>>,
    /// The start timepoint of the action.
    start: Timepoint,
    /// The end timepoint of the action.
    end: Timepoint,
    /// The expected duration of the action.
    duration: Option<TemporalIntervalExpression<E>>,
}

impl<E> DurativeAction<E> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: String,
        id: String,
        params: Vec<Parameter>,
        conditions: Vec<DurativeCondition<E>>,
        effects: Vec<DurativeEffect<E>>,
        start: Timepoint,
        end: Timepoint,
        duration: Option<TemporalIntervalExpression<E>>,
    ) -> Self {
        Self {
            base: BaseAction { name, id, params },
            conditions,
            effects,
            start,
            end,
            duration,
        }
    }

    /// Returns the name of the action.
    pub fn name(&self) -> &String {
        &self.base.name
    }

    /// Returns the id of the action.
    pub fn id(&self) -> &String {
        &self.base.id
    }

    /// Returns the conditions of the action.
    pub fn conditions(&self) -> &[DurativeCondition<E>] {
        self.conditions.as_ref()
    }

    /// Returns the effects of the action.
    pub fn effects(&self) -> &[DurativeEffect<E>] {
        self.effects.as_ref()
    }

    /// Returns the expected duration of the action.
    pub fn duration(&self) -> &Option<TemporalIntervalExpression<E>> {
        &self.duration
    }
}

impl<E: Clone> Configurable<E> for DurativeAction<E> {
    fn params(&self) -> &[Parameter] {
        self.base.params.as_ref()
    }
}

impl<E> Durative<E> for DurativeAction<E> {
    fn start(&self, _: &Env<E>) -> &Timepoint {
        &self.start
    }

    fn end(&self, _: &Env<E>) -> &Timepoint {
        &self.end
    }

    fn is_start_open(&self) -> bool {
        false
    }

    fn is_end_open(&self) -> bool {
        false
    }
}

impl<E> Display for DurativeAction<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{} ({})",
            self.name(),
            self.base
                .params
                .iter()
                .map(|p| format!("{}", p.value()))
                .collect::<Vec<_>>()
                .join(", ")
        ))
    }
}

/* ========================================================================== */
/*                                    Tests                                   */
/* ========================================================================== */

#[cfg(test)]
mod tests {
    use crate::models::{effects::EffectKind, value::Value};

    use super::*;

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct MockExpr(Value);
    impl Default for MockExpr {
        fn default() -> Self {
            Self(true.into())
        }
    }
    impl Interpreter for MockExpr {
        fn eval(&self, _: &Env<Self>) -> Result<Value> {
            Ok(self.0.clone())
        }

        fn convert_to_csp_constraint(&self, _: &Env<Self>) -> Result<crate::models::csp::CspConstraint> {
            todo!()
        }
    }

    fn f(s: &str) -> Vec<MockExpr> {
        vec![MockExpr(s.into())]
    }
    fn v(i: i64) -> MockExpr {
        MockExpr(i.into())
    }
    fn c(b: bool) -> SpanCondition<MockExpr> {
        SpanCondition::new(MockExpr(b.into()))
    }
    fn e(cond: &[bool], fs: &str, val: i64) -> SpanEffect<MockExpr> {
        let conditions = cond.iter().map(|b| c(*b)).collect::<Vec<_>>();
        SpanEffect::new(f(fs), v(val), EffectKind::Assign, conditions)
    }
    fn sa(cond: &[bool], effects: Vec<SpanEffect<MockExpr>>) -> SpanAction<MockExpr> {
        let conditions = cond.iter().map(|b| c(*b)).collect::<Vec<_>>();
        SpanAction::new("a".into(), "".into(), vec![], conditions, effects)
    }
    fn da() -> DurativeAction<MockExpr> {
        let s = Timepoint::fixed(5.into());
        let e = Timepoint::fixed(10.into());
        DurativeAction::new("d".into(), "".into(), vec![], vec![], vec![], s, e, None)
    }

    #[test]
    fn into_durative() {
        let dur_actions: Vec<Action<MockExpr>> = vec![da().into(), da().into(), da().into()];
        assert_eq!(Action::into_durative(&dur_actions), vec![da(), da(), da()]);

        let spn_actions: Vec<Action<MockExpr>> = vec![
            sa(&[], vec![]).into(),
            sa(&[], vec![]).into(),
            sa(&[], vec![]).into(),
            sa(&[], vec![]).into(),
        ];
        let expected = &[(0, 1), (2, 3), (4, 5), (6, 7)]
            .iter()
            .map(|(s, e)| {
                DurativeAction::new(
                    "a".into(),
                    "".into(),
                    vec![],
                    vec![],
                    vec![],
                    Timepoint::fixed((*s).into()),
                    Timepoint::fixed((*e).into()),
                    None,
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(Action::into_durative(&spn_actions), expected.to_vec());
    }

    #[test]
    fn from_span() {
        assert_eq!(Action::Span(sa(&[], vec![])), sa(&[], vec![]).into());
    }

    #[test]
    fn from_durative() {
        assert_eq!(Action::Durative(da()), da().into());
    }

    #[test]
    fn conditions() {
        let a = sa(&[true, false], vec![]);
        assert_eq!(a.conditions(), &[c(true), c(false)]);
    }

    #[test]
    fn applicable() -> Result<()> {
        let mut env = Env::<MockExpr>::default();
        env.bound_fluent(vec!["a".into()], 10.into());
        env.bound_fluent(vec!["b".into()], 10.into());

        let eta = e(&[true], "a", 5);
        let efa = e(&[false], "a", 5);
        let etb = e(&[true], "b", 2);
        let efb = e(&[false], "b", 2);
        let effects = vec![eta.clone(), etb.clone(), efa.clone(), efb.clone()];

        for condition in vec![true, false] {
            for e1 in effects.iter() {
                for e2 in effects.iter() {
                    let conditions = [condition];
                    let action = sa(&conditions, vec![e1.clone(), e2.clone()]);

                    if !condition || (e1 == e2 && e1.applicable(&env)?) {
                        assert!(!action.applicable(&env)?, "{:?}\n{:?}", e1, e2);
                    } else {
                        assert!(action.applicable(&env)?, "{:?}\n{:?}", e1, e2);
                    }
                }
            }
        }
        Ok(())
    }

    #[test]
    fn apply() -> Result<()> {
        let mut env = Env::<MockExpr>::default();
        env.bound_fluent(vec!["a".into()], 10.into());
        env.bound_fluent(vec!["b".into()], 10.into());

        let eta = e(&[true], "a", 5);
        let efa = e(&[false], "a", 5);
        let etb = e(&[true], "b", 2);
        let efb = e(&[false], "b", 2);
        let effects = vec![eta.clone(), etb.clone(), efa.clone(), efb.clone()];

        for condition in vec![true, false] {
            for e1 in effects.iter() {
                for e2 in effects.iter() {
                    let conditions = [condition];
                    let action = sa(&conditions, vec![e1.clone(), e2.clone()]);
                    let state = action.apply(&env, env.state())?;

                    if !condition || (e1 == e2 && e1.applicable(&env)?) {
                        assert!(state.is_none(), "{:?}\n{:?}", e1, e2);
                    } else {
                        assert!(state.is_some(), "{:?}\n{:?}", e1, e2);
                        let state = state.unwrap();

                        if *e1 == eta || *e2 == eta {
                            assert_eq!(*state.get(&vec!["a".into()]).unwrap(), 5.into());
                        } else if *e1 == efa || *e2 == efa {
                            assert_eq!(*state.get(&vec!["a".into()]).unwrap(), 10.into());
                        } else if *e1 == etb || *e2 == etb {
                            assert_eq!(*state.get(&vec!["b".into()]).unwrap(), 2.into());
                        } else {
                            // efb
                            assert_eq!(*state.get(&vec!["b".into()]).unwrap(), 10.into())
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
