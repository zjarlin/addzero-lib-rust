// Generated from OpenAPI spec. Do not edit by hand.
//! Evals REST endpoint contract.

use async_trait::async_trait;

use crate::models::{
    CreateEvalRequest,
    CreateEvalRunRequest,
    DeleteEvalResponse,
    DeleteEvalRunResponse,
    Eval,
    EvalList,
    EvalRun,
    EvalRunList,
    EvalRunOutputItem,
    EvalRunOutputItemList,
    UpdateEvalRequest,
};

/// Evals REST endpoints.
#[async_trait]
pub trait OpenAiEvalsApi: Send + Sync {
    /// Error type returned by the application-layer implementation.
    type Error: std::error::Error + Send + Sync + 'static;

    /// List evaluations for a project.
    ///
    /// REST: `GET /evals`.
    /// Path constant: [`OpenAiApiPath::EVALS`](crate::paths::OpenAiApiPath::EVALS).
    async fn list_evals(
        &self,
        after: Option<String>,
        limit: Option<i32>,
        order: Option<String>,
        order_by: Option<String>,
    ) -> Result<EvalList, Self::Error>;

    /// Create the structure of an evaluation that can be used to test a model's performance. An evaluation
    /// is a set of testing criteria and the config for a data source, which dictates the schema of the data
    /// used in the evaluation. After creating an evaluation, you can run it on different models and model
    /// parameters. We support several types of graders and datasources. For more information, see the
    /// [Evals guide](/docs/guides/evals).
    ///
    /// REST: `POST /evals`.
    /// Path constant: [`OpenAiApiPath::EVALS`](crate::paths::OpenAiApiPath::EVALS).
    async fn create_eval(&self, body: CreateEvalRequest) -> Result<Eval, Self::Error>;

    /// Get an evaluation by ID.
    ///
    /// REST: `GET /evals/{eval_id}`.
    /// Path constant: [`OpenAiApiPath::EVALS_BY_EVAL_ID`](crate::paths::OpenAiApiPath::EVALS_BY_EVAL_ID).
    async fn get_eval(&self, eval_id: String) -> Result<Eval, Self::Error>;

    /// Update certain properties of an evaluation.
    ///
    /// REST: `POST /evals/{eval_id}`.
    /// Path constant: [`OpenAiApiPath::EVALS_BY_EVAL_ID`](crate::paths::OpenAiApiPath::EVALS_BY_EVAL_ID).
    async fn update_eval(
        &self,
        eval_id: String,
        body: UpdateEvalRequest,
    ) -> Result<Eval, Self::Error>;

    /// Delete an evaluation.
    ///
    /// REST: `DELETE /evals/{eval_id}`.
    /// Path constant: [`OpenAiApiPath::EVALS_BY_EVAL_ID`](crate::paths::OpenAiApiPath::EVALS_BY_EVAL_ID).
    async fn delete_eval(&self, eval_id: String) -> Result<DeleteEvalResponse, Self::Error>;

    /// Get a list of runs for an evaluation.
    ///
    /// REST: `GET /evals/{eval_id}/runs`.
    /// Path constant: [`OpenAiApiPath::EVALS_BY_EVAL_ID_BY_RUNS`](crate::paths::OpenAiApiPath::EVALS_BY_EVAL_ID_BY_RUNS).
    async fn get_eval_runs(
        &self,
        eval_id: String,
        after: Option<String>,
        limit: Option<i32>,
        order: Option<String>,
        status: Option<String>,
    ) -> Result<EvalRunList, Self::Error>;

    /// Kicks off a new run for a given evaluation, specifying the data source, and what model configuration
    /// to use to test. The datasource will be validated against the schema specified in the config of the
    /// evaluation.
    ///
    /// REST: `POST /evals/{eval_id}/runs`.
    /// Path constant: [`OpenAiApiPath::EVALS_BY_EVAL_ID_BY_RUNS`](crate::paths::OpenAiApiPath::EVALS_BY_EVAL_ID_BY_RUNS).
    async fn create_eval_run(
        &self,
        eval_id: String,
        body: CreateEvalRunRequest,
    ) -> Result<EvalRun, Self::Error>;

    /// Get an evaluation run by ID.
    ///
    /// REST: `GET /evals/{eval_id}/runs/{run_id}`.
    /// Path constant: [`OpenAiApiPath::EVALS_BY_EVAL_ID_BY_RUNS_BY_RUN_ID`](crate::paths::OpenAiApiPath::EVALS_BY_EVAL_ID_BY_RUNS_BY_RUN_ID).
    async fn get_eval_run(&self, eval_id: String, run_id: String) -> Result<EvalRun, Self::Error>;

    /// Cancel an ongoing evaluation run.
    ///
    /// REST: `POST /evals/{eval_id}/runs/{run_id}`.
    /// Path constant: [`OpenAiApiPath::EVALS_BY_EVAL_ID_BY_RUNS_BY_RUN_ID`](crate::paths::OpenAiApiPath::EVALS_BY_EVAL_ID_BY_RUNS_BY_RUN_ID).
    async fn cancel_eval_run(
        &self,
        eval_id: String,
        run_id: String,
    ) -> Result<EvalRun, Self::Error>;

    /// Delete an eval run.
    ///
    /// REST: `DELETE /evals/{eval_id}/runs/{run_id}`.
    /// Path constant: [`OpenAiApiPath::EVALS_BY_EVAL_ID_BY_RUNS_BY_RUN_ID`](crate::paths::OpenAiApiPath::EVALS_BY_EVAL_ID_BY_RUNS_BY_RUN_ID).
    async fn delete_eval_run(
        &self,
        eval_id: String,
        run_id: String,
    ) -> Result<DeleteEvalRunResponse, Self::Error>;

    /// Get a list of output items for an evaluation run.
    ///
    /// REST: `GET /evals/{eval_id}/runs/{run_id}/output_items`.
    /// Path constant: [`OpenAiApiPath::EVALS_BY_EVAL_ID_BY_RUNS_BY_RUN_ID_BY_OUTPUT_ITEMS`](crate::paths::OpenAiApiPath::EVALS_BY_EVAL_ID_BY_RUNS_BY_RUN_ID_BY_OUTPUT_ITEMS).
    async fn get_eval_run_output_items(
        &self,
        eval_id: String,
        run_id: String,
        after: Option<String>,
        limit: Option<i32>,
        status: Option<String>,
        order: Option<String>,
    ) -> Result<EvalRunOutputItemList, Self::Error>;

    /// Get an evaluation run output item by ID.
    ///
    /// REST: `GET /evals/{eval_id}/runs/{run_id}/output_items/{output_item_id}`.
    /// Path constant: [`OpenAiApiPath::EVALS_BY_EVAL_ID_BY_RUNS_BY_RUN_ID_BY_OUTPUT_ITEMS_BY_OUTPUT_ITEM_ID`](crate::paths::OpenAiApiPath::EVALS_BY_EVAL_ID_BY_RUNS_BY_RUN_ID_BY_OUTPUT_ITEMS_BY_OUTPUT_ITEM_ID).
    async fn get_eval_run_output_item(
        &self,
        eval_id: String,
        run_id: String,
        output_item_id: String,
    ) -> Result<EvalRunOutputItem, Self::Error>;
}
