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

use common_catalog::table::Table;
use common_catalog::table_context::TableContext;
use common_exception::Result;
use common_expression::types::DataType;
use common_expression::utils::ColumnFrom;
use common_expression::BlockEntry;
use common_expression::Column;
use common_expression::DataBlock;
use common_expression::TableDataType;
use common_expression::TableField;
use common_expression::TableSchemaRefExt;
use common_expression::Value;
use common_meta_app::schema::TableIdent;
use common_meta_app::schema::TableInfo;
use common_meta_app::schema::TableMeta;

use crate::SyncOneBlockSystemTable;
use crate::SyncSystemTable;

pub struct ContributorsTable {
    table_info: TableInfo,
}

impl SyncSystemTable for ContributorsTable {
    const NAME: &'static str = "system.contributors";

    fn get_table_info(&self) -> &TableInfo {
        &self.table_info
    }

    fn get_full_data(&self, _: Arc<dyn TableContext>) -> Result<DataBlock> {
        let contributors: Vec<Vec<u8>> = env!("DATABEND_COMMIT_AUTHORS")
            .split_terminator(',')
            .map(|x| x.trim().as_bytes().to_vec())
            .collect();

        let rows_len = contributors.len();
        Ok(DataBlock::new(
            vec![BlockEntry {
                data_type: DataType::String,
                value: Value::Column(Column::from_data(contributors)),
            }],
            rows_len,
        ))
    }
}

impl ContributorsTable {
    pub fn create(table_id: u64) -> Arc<dyn Table> {
        let schema =
            TableSchemaRefExt::create(vec![TableField::new("name", TableDataType::String)]);

        let table_info = TableInfo {
            desc: "'system'.'contributors'".to_string(),
            name: "contributors".to_string(),
            ident: TableIdent::new(table_id, 0),
            meta: TableMeta {
                schema,
                engine: "SystemContributors".to_string(),
                ..Default::default()
            },
            ..Default::default()
        };

        SyncOneBlockSystemTable::create(ContributorsTable { table_info })
    }
}
