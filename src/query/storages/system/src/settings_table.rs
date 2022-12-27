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
use snailquote::escape;

use crate::SyncOneBlockSystemTable;
use crate::SyncSystemTable;

pub struct SettingsTable {
    table_info: TableInfo,
}

impl SyncSystemTable for SettingsTable {
    const NAME: &'static str = "system.settings";

    fn get_table_info(&self) -> &TableInfo {
        &self.table_info
    }

    fn get_full_data(&self, ctx: Arc<dyn TableContext>) -> Result<DataBlock> {
        let settings = ctx.get_settings().get_setting_values();

        let mut names: Vec<String> = vec![];
        let mut values: Vec<String> = vec![];
        let mut defaults: Vec<String> = vec![];
        let mut levels: Vec<String> = vec![];
        let mut descs: Vec<String> = vec![];
        let mut types: Vec<String> = vec![];
        for vals in settings {
            // Name.
            names.push(vals.0);
            // Value.
            values.push(escape(format!("{:?}", vals.1).as_str()).to_string());
            // Default Value.
            defaults.push(escape(format!("{:?}", vals.2).as_str()).to_string());
            // Scope level.
            levels.push(vals.3);
            // Desc.
            descs.push(vals.4);

            let typename = match vals.2 {
                common_meta_types::UserSettingValue::UInt64(_) => "UInt64",
                common_meta_types::UserSettingValue::String(_) => "String",
            };
            // Types.
            types.push(typename.to_string());
        }

        let names: Vec<Vec<u8>> = names.iter().map(|x| x.as_bytes().to_vec()).collect();
        let values: Vec<Vec<u8>> = values.iter().map(|x| x.as_bytes().to_vec()).collect();
        let defaults: Vec<Vec<u8>> = defaults.iter().map(|x| x.as_bytes().to_vec()).collect();
        let levels: Vec<Vec<u8>> = levels.iter().map(|x| x.as_bytes().to_vec()).collect();
        let descs: Vec<Vec<u8>> = descs.iter().map(|x| x.as_bytes().to_vec()).collect();
        let types: Vec<Vec<u8>> = types.iter().map(|x| x.as_bytes().to_vec()).collect();

        let rows_len = names.len();
        Ok(DataBlock::new(
            vec![
                BlockEntry {
                    data_type: DataType::String,
                    value: Value::Column(Column::from_data(names)),
                },
                BlockEntry {
                    data_type: DataType::String,
                    value: Value::Column(Column::from_data(values)),
                },
                BlockEntry {
                    data_type: DataType::String,
                    value: Value::Column(Column::from_data(defaults)),
                },
                BlockEntry {
                    data_type: DataType::String,
                    value: Value::Column(Column::from_data(levels)),
                },
                BlockEntry {
                    data_type: DataType::String,
                    value: Value::Column(Column::from_data(descs)),
                },
                BlockEntry {
                    data_type: DataType::String,
                    value: Value::Column(Column::from_data(types)),
                },
            ],
            rows_len,
        ))
    }
}

impl SettingsTable {
    pub fn create(table_id: u64) -> Arc<dyn Table> {
        let schema = TableSchemaRefExt::create(vec![
            TableField::new("name", TableDataType::String),
            TableField::new("value", TableDataType::String),
            TableField::new("default", TableDataType::String),
            TableField::new("level", TableDataType::String),
            TableField::new("description", TableDataType::String),
            TableField::new("type", TableDataType::String),
        ]);

        let table_info = TableInfo {
            desc: "'system'.'settings'".to_string(),
            name: "settings".to_string(),
            ident: TableIdent::new(table_id, 0),
            meta: TableMeta {
                schema,
                engine: "SystemSettings".to_string(),

                ..Default::default()
            },
            ..Default::default()
        };

        SyncOneBlockSystemTable::create(SettingsTable { table_info })
    }
}
