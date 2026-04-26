use std::sync::{Arc, Mutex};

use thiserror::Error;

use crate::domain::plan::Plan;
use crate::domain::step::{Step, StepExecutionResult};
use crate::domain::task::{Task, TaskRunRequest};
use crate::domain::task_context::TaskContext;

pub trait Planner: Send + Sync {
    fn create_initial_plan(
        &self,
        request: &TaskRunRequest,
        context: &TaskContext,
    ) -> Result<Plan, PlanningError>;

    fn replan(
        &self,
        task: &Task,
        failed_step: &Step,
        failure: &StepExecutionResult,
    ) -> Result<Vec<Step>, PlanningError>;
}

#[derive(Debug, Clone)]
pub struct StaticPlanner {
    initial_plan: Plan,
    replans: Arc<Mutex<Vec<Vec<Step>>>>,
}

impl StaticPlanner {
    pub fn new(initial_plan: Plan) -> Self {
        Self { initial_plan, replans: Arc::new(Mutex::new(Vec::new())) }
    }

    pub fn with_replans(initial_plan: Plan, replans: Vec<Vec<Step>>) -> Self {
        Self { initial_plan, replans: Arc::new(Mutex::new(replans)) }
    }
}

type InitialPlanCallback =
    dyn Fn(&TaskRunRequest, &TaskContext) -> Result<Plan, PlanningError> + Send + Sync;
type ReplanCallback =
    dyn Fn(&Task, &Step, &StepExecutionResult) -> Result<Vec<Step>, PlanningError> + Send + Sync;

#[derive(Clone)]
pub struct CallbackPlanner {
    create_initial_plan_callback: Arc<InitialPlanCallback>,
    replan_callback: Arc<ReplanCallback>,
}

impl CallbackPlanner {
    pub fn new<CreateInitialPlan, Replan>(
        create_initial_plan_callback: CreateInitialPlan,
        replan_callback: Replan,
    ) -> Self
    where
        CreateInitialPlan: Fn(&TaskRunRequest, &TaskContext) -> Result<Plan, PlanningError>
            + Send
            + Sync
            + 'static,
        Replan: Fn(&Task, &Step, &StepExecutionResult) -> Result<Vec<Step>, PlanningError>
            + Send
            + Sync
            + 'static,
    {
        Self {
            create_initial_plan_callback: Arc::new(create_initial_plan_callback),
            replan_callback: Arc::new(replan_callback),
        }
    }
}

impl Planner for StaticPlanner {
    fn create_initial_plan(
        &self,
        _request: &TaskRunRequest,
        _context: &TaskContext,
    ) -> Result<Plan, PlanningError> {
        Ok(self.initial_plan.clone())
    }

    fn replan(
        &self,
        _task: &Task,
        _failed_step: &Step,
        _failure: &StepExecutionResult,
    ) -> Result<Vec<Step>, PlanningError> {
        let mut replans = self.replans.lock().map_err(|_| {
            PlanningError::Internal("failed to acquire replan queue lock".to_string())
        })?;

        if replans.is_empty() {
            return Err(PlanningError::ReplanUnavailable(
                "no replacement plan is available".to_string(),
            ));
        }

        Ok(replans.remove(0))
    }
}

impl Planner for CallbackPlanner {
    fn create_initial_plan(
        &self,
        request: &TaskRunRequest,
        context: &TaskContext,
    ) -> Result<Plan, PlanningError> {
        (self.create_initial_plan_callback)(request, context)
    }

    fn replan(
        &self,
        task: &Task,
        failed_step: &Step,
        failure: &StepExecutionResult,
    ) -> Result<Vec<Step>, PlanningError> {
        (self.replan_callback)(task, failed_step, failure)
    }
}

impl<P> Planner for Arc<P>
where
    P: Planner + ?Sized,
{
    fn create_initial_plan(
        &self,
        request: &TaskRunRequest,
        context: &TaskContext,
    ) -> Result<Plan, PlanningError> {
        (**self).create_initial_plan(request, context)
    }

    fn replan(
        &self,
        task: &Task,
        failed_step: &Step,
        failure: &StepExecutionResult,
    ) -> Result<Vec<Step>, PlanningError> {
        (**self).replan(task, failed_step, failure)
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum PlanningError {
    #[error("planner returned an invalid plan: {0}")]
    InvalidPlan(String),
    #[error("planner cannot provide a replan: {0}")]
    ReplanUnavailable(String),
    #[error("planner internal error: {0}")]
    Internal(String),
}

#[cfg(test)]
mod tests {
    use std::panic::{AssertUnwindSafe, catch_unwind};
    use std::sync::{Arc, Mutex};

    use serde_json::json;

    use super::{CallbackPlanner, Planner, PlanningError, StaticPlanner};
    use crate::domain::limits::RunLimits;
    use crate::domain::plan::Plan;
    use crate::domain::step::{Recoverability, Step, StepExecutionResult};
    use crate::domain::task::{Task, TaskRunRequest};

    fn build_task() -> Task {
        let request = TaskRunRequest {
            goal: "Recover a bounded task".to_string(),
            input: json!({"ticket": "PLAN-1"}),
            session_id: "session-planner".to_string(),
            workspace_ref: "/tmp/synod-planner".to_string(),
            limits: RunLimits::default(),
            initial_context: None,
        };
        let plan = Plan::new(vec![Step::decision("verify", json!({})).unwrap()]).unwrap();
        Task::new("task-planner", &request, plan).unwrap()
    }

    #[test]
    fn static_planner_replan_reports_internal_error_when_the_queue_lock_is_poisoned() {
        let planner = StaticPlanner::with_replans(
            Plan::new(vec![Step::decision("initial", json!({})).unwrap()]).unwrap(),
            vec![vec![Step::decision("replacement", json!({})).unwrap()]],
        );
        let task = build_task();
        let failed_step = task.plan.current_step().unwrap().clone();
        let failure = StepExecutionResult::failure(
            crate::domain::step::ErrorInfo::new("bad-output", "need a replan"),
            Recoverability::ReplanRequired,
        );

        let replans = planner.replans.clone();
        let _ = catch_unwind(AssertUnwindSafe(|| {
            let _guard = replans.lock().unwrap();
            panic!("poison the replan queue");
        }));

        assert!(matches!(
            planner.replan(&task, &failed_step, &failure).unwrap_err(),
            PlanningError::Internal(message) if message.contains("failed to acquire replan queue lock")
        ));
    }

    #[test]
    fn callback_planner_delegates_initial_plan_and_replan() {
        let events = Arc::new(Mutex::new(Vec::new()));
        let initial_steps =
            Plan::new(vec![Step::decision("initial", json!({"adaptive": true})).unwrap()]).unwrap();
        let replacement = vec![Step::decision("replacement", json!({"attempt": 2})).unwrap()];

        let planner = CallbackPlanner::new(
            {
                let events = events.clone();
                let initial_steps = initial_steps.clone();
                move |request, _context| {
                    events.lock().unwrap().push(format!("initial:{}", request.goal));
                    Ok(initial_steps.clone())
                }
            },
            {
                let events = events.clone();
                let replacement = replacement.clone();
                move |task, failed_step, _failure| {
                    events.lock().unwrap().push(format!("replan:{}:{}", task.goal, failed_step.id));
                    Ok(replacement.clone())
                }
            },
        );

        let task = build_task();
        let failed_step = task.plan.current_step().unwrap().clone();
        let failure = StepExecutionResult::failure(
            crate::domain::step::ErrorInfo::new("bad-output", "need a replan"),
            Recoverability::ReplanRequired,
        );

        let created = planner
            .create_initial_plan(
                &TaskRunRequest {
                    goal: "adaptive goal".to_string(),
                    input: json!({}),
                    session_id: "session".to_string(),
                    workspace_ref: "/tmp/workspace".to_string(),
                    limits: RunLimits::default(),
                    initial_context: None,
                },
                &task.context,
            )
            .unwrap();
        let replanned = planner.replan(&task, &failed_step, &failure).unwrap();

        assert_eq!(created, initial_steps);
        assert_eq!(replanned, replacement);
        assert_eq!(
            events.lock().unwrap().clone(),
            vec![
                "initial:adaptive goal".to_string(),
                "replan:Recover a bounded task:verify".to_string(),
            ]
        );
    }
}
