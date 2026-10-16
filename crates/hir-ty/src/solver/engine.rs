use std::{
    collections::VecDeque,
    convert::Infallible,
    hash::{Hash, Hasher},
};

use super::evidence::{BranchOrigin, BranchRoot};

use tablesolve::{
    AnswerlessMode, Canonical as TabledCanonical, CanonicalizeResult, ClausesResult,
    Config as TabledConfig, ContextTransition, Limits, ReportOptions, ResolutionContext,
    Scheduling, Transition,
};

use super::*;

// Tabled resolution can otherwise spend the entire work-item fuel budget on a
// strictly type-growing chain (`C(a) => C(Box(a))`), retaining and repeatedly
// canonicalizing increasingly large goals. Real programs share subgoals and
// stay far below this bound; the separate cap keeps pathological growth cheap
// without charging rigid-head-prefiltered clauses against useful solver fuel.
const MAX_TABLE_ENTRIES: usize = 1_024;

/// Solcore's language adapter for the shared tabled-resolution engine.
pub(super) struct TabledEngine<'db> {
    db: &'db dyn Db,
    env: TraitEnvId<'db>,
    /// Variables fixed by the surrounding checked body; never solved by the
    /// engine and tracked by stable origin across canonicalization.
    local_context_vars: Vec<RigidVar>,
    /// Work-item budget retained for compatibility with `SolverReport`.
    fuel: usize,
    discovery: Option<ProofDiscovery<'db>>,
    preferred: ProofPreference<'db>,
}

impl<'db> TabledEngine<'db> {
    pub(super) fn new(db: &'db dyn Db, env: TraitEnvId<'db>, fuel: usize) -> Self {
        let mut local_context_vars = FxHashSet::default();
        for pred in env.local_givens(db) {
            collect_pred_vars(db, *pred, &mut local_context_vars);
        }
        let mut local_context_vars = local_context_vars.into_iter().collect::<Vec<_>>();
        local_context_vars.sort_unstable();
        let local_context_vars = local_context_vars
            .into_iter()
            .map(RigidVar::identity)
            .collect();
        Self {
            db,
            env,
            local_context_vars,
            fuel,
            discovery: Some(ProofDiscovery::default()),
            preferred: ProofPreference::default(),
        }
    }

    /// Resolve `goal` through `tablesolve`, mapping the generic engine report
    /// back to the solver's existing result and counter types.
    pub(super) fn run(
        &mut self,
        goal: Pred<'db>,
        allowed_goal_vars: &FxHashSet<u32>,
    ) -> EngineResult<'db> {
        // Discover logical answers without multiplying their dictionary
        // derivations. If no different dictionaries share a logical answer,
        // this first pass already contains every complete proof.
        let mut logical = self.run_once(goal, allowed_goal_vars);
        let discovery = self.discovery.take().expect("initial discovery pass");
        if !discovery.collapsed || logical.exhausted {
            return logical;
        }

        // Saturated finite witnesses justify preference before expansion.
        // Their recursion footprints determine which contexts can safely use
        // them; a recursive preferred proof must not erase its own seed.
        let Ok(preferred) = discovery.finish(&mut logical.fuel_remaining) else {
            logical.exhausted = true;
            return logical;
        };
        self.preferred = preferred;
        self.fuel = logical.fuel_remaining;
        let mut complete = self.run_once(goal, allowed_goal_vars);
        complete.stats.add(logical.stats);
        complete
    }

    fn run_once(
        &mut self,
        goal: Pred<'db>,
        allowed_goal_vars: &FxHashSet<u32>,
    ) -> EngineResult<'db> {
        let root = SolverGoal {
            pred: goal,
            allowed_vars: allowed_goal_vars.clone(),
            rigid_vars: self.local_context_vars.clone(),
            context: ProofContext::default(),
        };
        let config = TabledConfig {
            scheduling: Scheduling::Fair,
            limits: Limits {
                max_steps: Some(self.fuel),
                max_tables: Some(MAX_TABLE_ENTRIES),
                max_root_answers: None,
                max_pending_work: None,
            },
        };
        let report_options = ReportOptions::new().with_answerless(AnswerlessMode::Omit);
        let report = tablesolve::solve_with_options(self, root, config, report_options)
            .unwrap_or_else(|error: Infallible| match error {});
        let stats = SolverStats {
            table_size: report.stats.tables_created,
            generator_steps: report.stats.clauses_tried,
            answers_found: report.stats.answers_added,
        };
        let exhausted = report.resource_exhausted();
        let fuel_remaining = self.fuel.saturating_sub(report.stats.steps);
        EngineResult {
            answers: report.answers,
            exhausted,
            fuel_remaining,
            stats,
        }
    }

    /// Program clauses eligible for `key`, in resolution order: local givens,
    /// then non-default instances, then superclass projections, and — only
    /// when no non-default clause head can unify with the goal — defaults.
    fn applicable_clauses(&self, key: &TableKey<'db>) -> Vec<ProgramClause<'db>> {
        // This is a one-way prefilter. Variables and their correlations are
        // deliberately ignored, so impossible clauses may remain but an
        // applicable clause is never discarded.
        let head_can_apply = |clause: &ProgramClause<'db>| {
            pred_head_shapes_may_match(self.db, clause.head, key.pred)
        };
        let mut clauses = Vec::new();
        clauses.extend(
            self.env
                .local_givens(self.db)
                .iter()
                .copied()
                .map(|given| ProgramClause {
                    binder_count: 0,
                    head: canonicalize_local_given(self.db, given, key),
                    conditions: Vec::new(),
                    origin: ClauseOrigin::Given,
                })
                .filter(&head_can_apply),
        );
        clauses.extend(env::builtin_function_invokables(self.db, key.pred));
        let base_clauses = self.env.clauses_for_pred(self.db, key.pred);
        clauses.extend(base_clauses.iter().filter_map(|clause| {
            (!clause.origin.is_default()
                && !matches!(clause.origin, ClauseOrigin::Superclass(_))
                && head_can_apply(clause))
            .then_some(clause.clone())
        }));
        clauses.extend(base_clauses.iter().filter_map(|clause| {
            (!clause.origin.is_default()
                && matches!(clause.origin, ClauseOrigin::Superclass(_))
                && head_can_apply(clause))
            .then_some(clause.clone())
        }));

        // Default selection is local to each tabled subgoal. A non-default
        // instance may itself rely on a condition discharged by a default.
        let default_clauses = base_clauses
            .iter()
            .filter(|clause| clause.origin.is_default() && head_can_apply(clause))
            .cloned()
            .collect::<Vec<_>>();
        if !default_clauses.is_empty() && !self.has_non_default_unifying_head(key, base_clauses) {
            clauses.extend(default_clauses);
        }
        clauses
    }

    fn has_non_default_unifying_head(
        &self,
        key: &TableKey<'db>,
        base_clauses: &[ProgramClause<'db>],
    ) -> bool {
        let mut goal_vars = key.allowed_vars();
        collect_pred_vars(self.db, key.pred, &mut goal_vars);
        base_clauses.iter().any(|clause| {
            !clause.origin.is_default()
                && !matches!(clause.origin, ClauseOrigin::Superclass(_))
                && head_can_unify(self.db, clause, key.pred, &goal_vars)
        })
    }

    fn suspend(&mut self, state: ConsumerState<'db>) -> ContextTransition<Self> {
        let condition = state
            .subst
            .apply_pred(self.db, state.clause.conditions[state.next_condition]);
        let goal = SolverGoal {
            pred: condition,
            allowed_vars: state.condition_vars.clone(),
            rigid_vars: state.rigid_vars.clone(),
            context: state.context.clone(),
        };
        Transition::Suspend { goal, state }
    }

    fn answer(
        &mut self,
        contextual: &ContextKey<'db>,
        clause: &InstantiatedClause<'db>,
        subst: MatchSubst<'db>,
        sub_evidence: Vec<Evidence<'db>>,
        prerequisites: Vec<ProofNode<'db>>,
    ) -> ContextTransition<Self> {
        let key = &contextual.logical;
        let evidence = clause_evidence(self.db, key.pred, clause, &subst, sub_evidence);
        let evidence = apply_evidence(self.db, evidence, &subst);
        // Recursive clauses can wrap a completed proof indefinitely. Only
        // simple dictionary derivations are useful: a repeated dictionary on a
        // branch can be replaced by the descendant proof of that dictionary.
        let repeats_root = evidence.repeats_root();
        if repeats_root && self.discovery.is_none() {
            return Transition::Reject;
        }
        let answer = Answer {
            candidate: Candidate {
                subst: subst.snapshot_for_vars(self.db, key.flex_count),
                evidence,
            },
            origin: clause.origin.clone(),
        };
        let logical_key = self.logical_answer_key(key, &answer);
        if let Some(discovery) = &mut self.discovery {
            // Discovery depends only on substitutions and provenance. A
            // repeated witness still proves logical reachability, but needs
            // the complete pass to enumerate its finite dictionary choices.
            discovery.collapsed |= repeats_root;
            discovery.record_recipe(key, &logical_key, prerequisites);
            let answers = discovery.answers.entry(key.clone()).or_default();
            if let Some(previous) = answers.get(&logical_key) {
                discovery.collapsed |= previous != &answer;
            } else {
                answers.insert(logical_key, answer.clone());
            }
        } else if self.preferred.dominates(
            key,
            &answer,
            logical_key.priority(),
            &contextual.context,
        ) {
            return Transition::Reject;
        }
        Transition::Answer(answer)
    }

    fn logical_answer_key(
        &self,
        key: &TableKey<'db>,
        answer: &Answer<'db>,
    ) -> LogicalAnswerKey<'db> {
        let local_given =
            evidence::evidence_root_given(&answer.candidate.evidence).is_some_and(|pred| {
                self.env
                    .local_givens(self.db)
                    .iter()
                    .any(|given| canonicalize_local_given(self.db, *given, key) == pred)
            });
        LogicalAnswerKey {
            subst: answer.candidate.subst.clone(),
            origin: answer.origin.clone(),
            local_given,
        }
    }
}

impl<'db> ResolutionContext for TabledEngine<'db> {
    type Goal = SolverGoal<'db>;
    type Key = ContextKey<'db>;
    type Clause = ProgramClause<'db>;
    type Answer = Answer<'db>;
    type AnswerKey = AnswerKey<'db>;
    type Output = Answer<'db>;
    type State = ConsumerState<'db>;
    type Rebase = GoalRenaming;
    type Error = Infallible;
    type StopReason = Infallible;

    fn canonicalize(
        &mut self,
        goal: Self::Goal,
    ) -> CanonicalizeResult<Self::Key, Self::Rebase, Self::StopReason, Self::Error> {
        let (key, rebase) =
            canonicalize_goal(self.db, goal.pred, &goal.allowed_vars, &goal.rigid_vars);
        Ok(TabledCanonical::new(
            ContextKey {
                logical: key,
                context: goal.context,
            },
            rebase,
        )
        .into())
    }

    fn clauses(
        &mut self,
        key: &Self::Key,
    ) -> ClausesResult<Self::Clause, Self::StopReason, Self::Error> {
        Ok(self.applicable_clauses(&key.logical).into())
    }

    fn apply_clause(
        &mut self,
        key: &Self::Key,
        clause: Self::Clause,
    ) -> Result<ContextTransition<Self>, Self::Error> {
        let logical = &key.logical;
        let allowed_goal_vars = logical.allowed_vars();
        let avoid_vars = logical.canonical_context_vars();
        let instantiated = instantiate_clause(self.db, &clause, logical.pred, &avoid_vars);
        let Some(subst) = match_head(
            self.db,
            instantiated.head,
            logical.pred,
            &instantiated.binder_vars,
            &allowed_goal_vars,
        ) else {
            return Ok(Transition::Reject);
        };

        let mut condition_vars = allowed_goal_vars;
        condition_vars.extend(instantiated.binder_vars.iter().copied());
        if instantiated.conditions.is_empty() {
            return Ok(self.answer(key, &instantiated, subst, Vec::new(), Vec::new()));
        }

        let mut context = key.context.clone();
        if self.discovery.is_none() {
            let evidence =
                clause_evidence(self.db, logical.pred, &instantiated, &subst, Vec::new());
            let root = apply_evidence(self.db, evidence, &subst).branch_root();
            if let Some(root) =
                root.and_then(|root| canonical::stable_branch_root(self.db, root, logical))
                && !context.roots.insert(root)
            {
                // Every completion would fail the final simple-proof check.
                return Ok(Transition::Reject);
            }
            context.origins.insert(
                BranchOrigin::for_clause(&instantiated.origin)
                    .expect("clauses with prerequisites have dictionary branches"),
            );
        }
        Ok(self.suspend(ConsumerState {
            clause: instantiated,
            subst,
            sub_evidence: Vec::new(),
            prerequisites: Vec::new(),
            next_condition: 0,
            condition_vars,
            rigid_vars: logical.rigid_vars().to_vec(),
            context,
        }))
    }

    fn resume(
        &mut self,
        parent: &Self::Key,
        mut state: Self::State,
        answer: Self::Answer,
        rebase: Self::Rebase,
    ) -> Result<ContextTransition<Self>, Self::Error> {
        let condition = state
            .subst
            .apply_pred(self.db, state.clause.conditions[state.next_condition]);
        let (child, _) =
            canonicalize_goal(self.db, condition, &state.condition_vars, &state.rigid_vars);
        let answer_key = self.logical_answer_key(&child, &answer);
        if self.discovery.is_some() {
            state.prerequisites.push(ProofNode {
                table: child,
                answer: answer_key,
            });
        }
        let alternative = actualize_answer(self.db, &answer, &rebase);
        let mut combined_subst = state.subst.clone();
        if !combined_subst.merge(self.db, &alternative.candidate.subst) {
            return Ok(Transition::Reject);
        }
        for (_, ty) in &alternative.candidate.subst.values {
            collect_ty_vars(self.db, *ty, &mut state.condition_vars);
        }
        state.sub_evidence.push(apply_evidence(
            self.db,
            alternative.candidate.evidence,
            &combined_subst,
        ));
        state.subst = combined_subst;
        state.next_condition += 1;
        if state.next_condition < state.clause.conditions.len() {
            Ok(self.suspend(state))
        } else {
            Ok(self.answer(
                parent,
                &state.clause,
                state.subst,
                state.sub_evidence,
                state.prerequisites,
            ))
        }
    }

    fn rebase_answer(
        &mut self,
        answer: &Self::Answer,
        rebase: &Self::Rebase,
    ) -> Result<Self::Output, Self::Error> {
        Ok(actualize_answer(self.db, answer, rebase))
    }

    fn answer_key(&self, key: &Self::Key, answer: &Self::Answer) -> Self::AnswerKey {
        if self.discovery.is_some() {
            AnswerKey::Logical(self.logical_answer_key(&key.logical, answer))
        } else {
            // Equal-priority dictionaries include all prerequisite evidence.
            AnswerKey::Complete(
                answer.candidate.subst.clone(),
                answer.candidate.evidence.dictionary_identity(&|pred| {
                    self.env.local_givens(self.db).iter().any(|given| {
                        canonicalize_local_given(self.db, *given, &key.logical) == pred
                    })
                }),
            )
        }
    }
}

/// Complete dictionaries are shared only in contexts with the same ancestors.
/// Discovery leaves this context empty, preserving context-free reachability.
#[derive(Clone, PartialEq, Eq, Hash)]
pub(super) struct ContextKey<'db> {
    logical: TableKey<'db>,
    context: ProofContext<'db>,
}

#[derive(Clone, Default, PartialEq, Eq)]
struct ProofContext<'db> {
    origins: Footprint<'db>,
    roots: FxHashSet<BranchRoot<'db>>,
}

impl Hash for ProofContext<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Set equality is independent of traversal order, and so must hashing
        // be. Hash collisions here are harmless: table equality compares sets.
        fn hash_set<T: Hash, H: Hasher>(set: &FxHashSet<T>, state: &mut H) {
            let mut hashes = set
                .iter()
                .map(|item| {
                    let mut hasher = std::collections::hash_map::DefaultHasher::new();
                    item.hash(&mut hasher);
                    hasher.finish()
                })
                .collect::<Vec<_>>();
            hashes.sort_unstable();
            hashes.hash(state);
        }
        hash_set(&self.origins, state);
        hash_set(&self.roots, state);
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub(super) enum AnswerKey<'db> {
    Logical(LogicalAnswerKey<'db>),
    Complete(Substitution<'db>, evidence::DictionaryIdentity<'db>),
}

/// Identity sufficient for logical reachability and root priority, but not
/// for the final choice of a dictionary. Local-given provenance matters for
/// superclass projections, which otherwise have the same clause origin.
#[derive(Clone, PartialEq, Eq, Hash)]
pub(super) struct LogicalAnswerKey<'db> {
    subst: Substitution<'db>,
    origin: ClauseOrigin<'db>,
    local_given: bool,
}

impl LogicalAnswerKey<'_> {
    fn priority(&self) -> u8 {
        evidence::clause_priority(&self.origin, self.local_given)
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct ProofNode<'db> {
    table: TableKey<'db>,
    answer: LogicalAnswerKey<'db>,
}

/// Child identities retain the correlations established by clause matching,
/// without committing their parents to the first arriving dictionary trees.
#[derive(PartialEq, Eq, Hash)]
struct ProofRecipe<'db> {
    origin: Option<BranchOrigin<'db>>,
    prerequisites: Vec<ProofNode<'db>>,
}

#[derive(Default)]
struct ProofDiscovery<'db> {
    answers: FxHashMap<TableKey<'db>, FxHashMap<LogicalAnswerKey<'db>, Answer<'db>>>,
    recipes: FxHashMap<ProofNode<'db>, FxHashSet<ProofRecipe<'db>>>,
    collapsed: bool,
}

type Footprint<'db> = FxHashSet<BranchOrigin<'db>>;
type WitnessIndex<'db> =
    FxHashMap<TableKey<'db>, FxHashMap<Substitution<'db>, Vec<ProofWitness<'db>>>>;

/// A finite proof's priority and conservative recursion footprint. Only
/// non-leaf origins matter: a leaf cannot be an ancestor of a substituted proof.
struct ProofWitness<'db> {
    priority: u8,
    origins: Footprint<'db>,
}

#[derive(Default)]
struct ProofPreference<'db> {
    witnesses: WitnessIndex<'db>,
}

impl<'db> ProofDiscovery<'db> {
    fn record_recipe(
        &mut self,
        table: &TableKey<'db>,
        answer: &LogicalAnswerKey<'db>,
        prerequisites: Vec<ProofNode<'db>>,
    ) {
        let origin = (!prerequisites.is_empty()).then(|| {
            BranchOrigin::for_clause(&answer.origin)
                .expect("clauses with prerequisites have dictionary branches")
        });
        self.recipes
            .entry(ProofNode {
                table: table.clone(),
                answer: answer.clone(),
            })
            .or_default()
            .insert(ProofRecipe {
                origin,
                prerequisites,
            });
    }

    fn finish(self, fuel: &mut usize) -> Result<ProofPreference<'db>, ()> {
        Ok(ProofPreference {
            witnesses: self.finite_witnesses(fuel)?,
        })
    }

    fn finite_witnesses(&self, fuel: &mut usize) -> Result<WitnessIndex<'db>, ()> {
        let mut consumers: FxHashMap<_, FxHashSet<_>> = FxHashMap::default();
        for (parent, recipes) in &self.recipes {
            for recipe in recipes {
                for child in &recipe.prerequisites {
                    consumers
                        .entry(child.clone())
                        .or_default()
                        .insert(parent.clone());
                }
            }
        }
        let mut footprints: FxHashMap<ProofNode<'db>, Vec<Footprint<'db>>> = FxHashMap::default();
        let mut pending = self.recipes.keys().cloned().collect::<VecDeque<_>>();
        let mut queued = self.recipes.keys().cloned().collect::<FxHashSet<_>>();
        while let Some(parent) = pending.pop_front() {
            queued.remove(&parent);
            let mut changed = false;
            for recipe in &self.recipes[&parent] {
                spend_analysis_step(fuel)?;
                let mut alternatives =
                    vec![recipe.origin.iter().cloned().collect::<Footprint<'db>>()];
                for child in &recipe.prerequisites {
                    let Some(children) = footprints.get(child) else {
                        alternatives.clear();
                        break;
                    };
                    let mut combined = Vec::new();
                    for prefix in &alternatives {
                        for child in children {
                            spend_analysis_step(fuel)?;
                            let mut origins = prefix.clone();
                            origins.extend(child.iter().cloned());
                            insert_minimal_footprint(&mut combined, origins);
                        }
                    }
                    alternatives = combined;
                }
                let witnesses = footprints.entry(parent.clone()).or_default();
                for alternative in alternatives {
                    changed |= insert_minimal_footprint(witnesses, alternative);
                }
            }
            if changed {
                for consumer in consumers.get(&parent).into_iter().flatten() {
                    if queued.insert(consumer.clone()) {
                        pending.push_back(consumer.clone());
                    }
                }
            }
        }
        // Leaf recipes seed a least fixed point of finite derivations. A
        // repeated dictionary root in a finite tree can be replaced by its
        // descendant proof, producing a simple witness with a subset of the
        // origins. These footprints therefore safely over-approximate it.
        let mut witnesses: WitnessIndex<'db> = FxHashMap::default();
        for (node, alternatives) in footprints {
            let destination = witnesses
                .entry(node.table)
                .or_default()
                .entry(node.answer.subst)
                .or_default();
            destination.extend(alternatives.into_iter().map(|origins| ProofWitness {
                priority: evidence::clause_priority(&node.answer.origin, node.answer.local_given),
                origins,
            }));
        }
        Ok(witnesses)
    }
}

fn insert_minimal_footprint<'db>(
    alternatives: &mut Vec<Footprint<'db>>,
    footprint: Footprint<'db>,
) -> bool {
    if alternatives.iter().any(|old| old.is_subset(&footprint)) {
        return false;
    }
    alternatives.retain(|old| !footprint.is_subset(old));
    alternatives.push(footprint);
    true
}

fn spend_analysis_step(fuel: &mut usize) -> Result<(), ()> {
    *fuel = fuel.checked_sub(1).ok_or(())?;
    Ok(())
}

impl<'db> ProofPreference<'db> {
    fn dominates(
        &self,
        goal: &TableKey<'db>,
        answer: &Answer<'db>,
        priority: u8,
        context: &ProofContext<'db>,
    ) -> bool {
        let Some(witnesses) = self
            .witnesses
            .get(goal)
            .and_then(|answers| answers.get(&answer.candidate.subst))
        else {
            return false;
        };
        witnesses.iter().any(|witness| {
            witness.priority < priority && witness.origins.is_disjoint(&context.origins)
        })
    }
}

#[derive(Clone)]
pub(super) struct SolverGoal<'db> {
    pred: Pred<'db>,
    allowed_vars: FxHashSet<u32>,
    rigid_vars: Vec<RigidVar>,
    context: ProofContext<'db>,
}

/// A partially solved clause retained by `tablesolve` while it waits for the
/// current condition's table to produce answers.
#[derive(Clone)]
pub(super) struct ConsumerState<'db> {
    clause: InstantiatedClause<'db>,
    subst: MatchSubst<'db>,
    sub_evidence: Vec<Evidence<'db>>,
    prerequisites: Vec<ProofNode<'db>>,
    next_condition: usize,
    condition_vars: FxHashSet<u32>,
    /// Stable rigid origins mapped into the parent subgoal's coordinates.
    rigid_vars: Vec<RigidVar>,
    context: ProofContext<'db>,
}

pub(super) struct EngineResult<'db> {
    pub(super) answers: Vec<Answer<'db>>,
    pub(super) exhausted: bool,
    pub(super) fuel_remaining: usize,
    pub(super) stats: SolverStats,
}

/// One answer for a subgoal: a substitution over its flex variables plus the
/// evidence that discharges the goal, tagged with the clause it came from.
#[derive(Clone, PartialEq, Eq, Hash)]
pub(super) struct Answer<'db> {
    pub(super) candidate: Candidate<'db>,
    pub(super) origin: ClauseOrigin<'db>,
}

fn pred_head_shapes_may_match<'db>(db: &'db dyn Db, lhs: Pred<'db>, rhs: Pred<'db>) -> bool {
    match (lhs.kind(db), rhs.kind(db)) {
        (
            PredKind::InClass {
                class: lhs_class,
                main: lhs_main,
                args: lhs_args,
            },
            PredKind::InClass {
                class: rhs_class,
                main: rhs_main,
                args: rhs_args,
            },
        ) if lhs_class == rhs_class && lhs_args.len() == rhs_args.len() => {
            ty_shapes_may_match(db, *lhs_main, *rhs_main)
                && lhs_args
                    .iter()
                    .zip(rhs_args)
                    .all(|(lhs_arg, rhs_arg)| ty_shapes_may_match(db, *lhs_arg, *rhs_arg))
        }
        (
            PredKind::Eq {
                lhs: lhs_l,
                rhs: lhs_r,
            },
            PredKind::Eq {
                lhs: rhs_l,
                rhs: rhs_r,
            },
        ) => ty_shapes_may_match(db, *lhs_l, *rhs_l) && ty_shapes_may_match(db, *lhs_r, *rhs_r),
        (PredKind::Error, PredKind::Error) => true,
        _ => false,
    }
}

fn ty_shapes_may_match<'db>(db: &'db dyn Db, lhs: Ty<'db>, rhs: Ty<'db>) -> bool {
    if let TyKind::Comptime(inner) = lhs.kind(db) {
        return ty_shapes_may_match(db, *inner, rhs);
    }
    if let TyKind::Comptime(inner) = rhs.kind(db) {
        return ty_shapes_may_match(db, lhs, *inner);
    }

    match (lhs.kind(db), rhs.kind(db)) {
        // Correlations between variables are intentionally ignored. This is a
        // one-way prefilter: false positives cost fuel, while false negatives
        // would make resolution incomplete.
        (TyKind::BoundVar(_), _) | (_, TyKind::BoundVar(_)) => true,
        (TyKind::Error | TyKind::Unknown, _) | (_, TyKind::Error | TyKind::Unknown) => true,
        (
            TyKind::Named {
                ctor: lhs_ctor,
                args: lhs_args,
            },
            TyKind::Named {
                ctor: rhs_ctor,
                args: rhs_args,
            },
        ) if lhs_ctor == rhs_ctor && lhs_args.len() == rhs_args.len() => lhs_args
            .iter()
            .zip(rhs_args)
            .all(|(lhs_arg, rhs_arg)| ty_shapes_may_match(db, *lhs_arg, *rhs_arg)),
        (
            TyKind::Named {
                ctor: TyCtor::Builtin(crate::BuiltinTyCtor::Unit),
                args,
            },
            TyKind::Tuple(elems),
        )
        | (
            TyKind::Tuple(elems),
            TyKind::Named {
                ctor: TyCtor::Builtin(crate::BuiltinTyCtor::Unit),
                args,
            },
        ) if args.is_empty() && elems.is_empty() => true,
        (
            TyKind::Function {
                params: lhs_params,
                ret: lhs_ret,
            },
            TyKind::Function {
                params: rhs_params,
                ret: rhs_ret,
            },
        ) if lhs_params.len() == rhs_params.len() => {
            lhs_params
                .iter()
                .zip(rhs_params)
                .all(|(lhs_param, rhs_param)| ty_shapes_may_match(db, *lhs_param, *rhs_param))
                && ty_shapes_may_match(db, *lhs_ret, *rhs_ret)
        }
        (TyKind::Tuple(lhs_elems), TyKind::Tuple(rhs_elems))
            if lhs_elems.len() == rhs_elems.len() =>
        {
            lhs_elems
                .iter()
                .zip(rhs_elems)
                .all(|(lhs_elem, rhs_elem)| ty_shapes_may_match(db, *lhs_elem, *rhs_elem))
        }
        _ => false,
    }
}
