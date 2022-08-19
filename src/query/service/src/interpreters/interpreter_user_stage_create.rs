// Copyright 2021 Datafuse Labs.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::sync::Arc;

use common_exception::ErrorCode;
use common_exception::Result;
use common_meta_types::StageType;
use common_planners::CreateUserStagePlan;
use common_streams::DataBlockStream;
use common_streams::SendableDataBlockStream;

use crate::interpreters::Interpreter;
use crate::sessions::QueryContext;
use crate::sessions::TableContext;

#[derive(Debug)]
pub struct CreateUserStageInterpreter {
    ctx: Arc<QueryContext>,
    plan: CreateUserStagePlan,
}

impl CreateUserStageInterpreter {
    pub fn try_create(ctx: Arc<QueryContext>, plan: CreateUserStagePlan) -> Result<Self> {
        Ok(CreateUserStageInterpreter { ctx, plan })
    }
}

#[async_trait::async_trait]
impl Interpreter for CreateUserStageInterpreter {
    fn name(&self) -> &str {
        "CreateUserStageInterpreter"
    }

    #[tracing::instrument(level = "info", skip(self), fields(ctx.id = self.ctx.get_id().as_str()))]
    async fn execute(&self) -> Result<SendableDataBlockStream> {
        let plan = self.plan.clone();
        let user_mgr = self.ctx.get_user_manager();
        let user_stage = plan.user_stage_info;
        let quota_api = user_mgr.get_tenant_quota_api_client(&plan.tenant)?;
        let quota = quota_api.get_quota(None).await?.data;
        let stages = user_mgr.get_stages(&plan.tenant).await?;
        if quota.max_stages != 0 && stages.len() >= quota.max_stages as usize {
            return Err(ErrorCode::TenantQuotaExceeded(format!(
                "Max stages quota exceeded {}",
                quota.max_stages
            )));
        };

        if user_stage.stage_type == StageType::Internal {
            let prefix = format!("stage/{}/", user_stage.stage_name);
            let op = self.ctx.get_storage_operator()?;
            op.object(&prefix).create().await?
        }

        let mut user_stage = user_stage;
        user_stage.creator = Some(self.ctx.get_current_user()?.identity());
        let _create_stage = user_mgr
            .add_stage(&plan.tenant, user_stage, plan.if_not_exists)
            .await?;

        Ok(Box::pin(DataBlockStream::create(
            self.plan.schema(),
            None,
            vec![],
        )))
    }
}