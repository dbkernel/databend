// Copyright 2022 Datafuse Labs.
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

use common_exception::Result;
use common_expression::types::DataType;
use common_expression::BlockEntry;
use common_expression::Column;
use common_expression::ColumnFrom;
use common_expression::DataBlock;
use common_expression::DataSchemaRef;
use common_expression::Value;
use common_meta_api::ShareApi;
use common_meta_app::share::GetShareGrantTenantsReq;
use common_meta_app::share::ShareNameIdent;
use common_users::UserApiProvider;

use crate::interpreters::Interpreter;
use crate::pipelines::PipelineBuildResult;
use crate::sessions::QueryContext;
use crate::sessions::TableContext;
use crate::sql::plans::share::ShowGrantTenantsOfSharePlan;

pub struct ShowGrantTenantsOfShareInterpreter {
    ctx: Arc<QueryContext>,
    plan: ShowGrantTenantsOfSharePlan,
}

impl ShowGrantTenantsOfShareInterpreter {
    pub fn try_create(ctx: Arc<QueryContext>, plan: ShowGrantTenantsOfSharePlan) -> Result<Self> {
        Ok(ShowGrantTenantsOfShareInterpreter { ctx, plan })
    }
}

#[async_trait::async_trait]
impl Interpreter for ShowGrantTenantsOfShareInterpreter {
    fn name(&self) -> &str {
        "ShowGrantTenantsOfShareInterpreter"
    }

    fn schema(&self) -> DataSchemaRef {
        self.plan.schema()
    }

    async fn execute2(&self) -> Result<PipelineBuildResult> {
        let meta_api = UserApiProvider::instance().get_meta_store_client();
        let tenant = self.ctx.get_tenant();
        let req = GetShareGrantTenantsReq {
            share_name: ShareNameIdent {
                tenant,
                share_name: self.plan.share_name.clone(),
            },
        };
        let resp = meta_api.get_grant_tenants_of_share(req).await?;
        if resp.accounts.is_empty() {
            return Ok(PipelineBuildResult::create());
        }

        let mut granted_ons: Vec<Vec<u8>> = vec![];
        let mut accounts: Vec<Vec<u8>> = vec![];
        let num_rows = resp.accounts.len();

        for account in resp.accounts {
            granted_ons.push(account.grant_on.to_string().as_bytes().to_vec());
            accounts.push(account.account.clone().as_bytes().to_vec());
        }

        PipelineBuildResult::from_blocks(vec![DataBlock::new(
            vec![
                BlockEntry {
                    data_type: DataType::String,
                    value: Value::Column(Column::from_data(granted_ons)),
                },
                BlockEntry {
                    data_type: DataType::String,
                    value: Value::Column(Column::from_data(accounts)),
                },
            ],
            num_rows,
        )])
    }
}
