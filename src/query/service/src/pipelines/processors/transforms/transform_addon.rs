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

use common_catalog::table_context::TableContext;
use common_exception::Result;
use common_expression::BlockEntry;
use common_expression::DataBlock;
use common_expression::DataField;
use common_expression::DataSchema;
use common_expression::DataSchemaRef;
use common_expression::DataSchemaRefExt;
use common_expression::Value;
use common_sql::evaluator::BlockOperator;
use common_sql::evaluator::CompoundBlockOperator;
use common_sql::parse_exprs;
use common_storages_factory::Table;

use crate::pipelines::processors::port::InputPort;
use crate::pipelines::processors::port::OutputPort;
use crate::pipelines::processors::processor::ProcessorPtr;
use crate::pipelines::processors::transforms::transform::Transform;
use crate::pipelines::processors::transforms::transform::Transformer;
use crate::sessions::QueryContext;

pub struct TransformAddOn {
    default_nonexpr_fields: Vec<DataField>,

    expression_transform: CompoundBlockOperator,

    /// The final schema of the output chunk.
    output_schema: DataSchemaRef,
    /// The schema of the output chunk before resorting.
    /// input fields | default expr fields | default nonexpr fields
    unresort_schema: DataSchemaRef,
}

impl TransformAddOn
where Self: Transform
{
    pub fn try_create(
        input: Arc<InputPort>,
        output: Arc<OutputPort>,
        input_schema: DataSchemaRef,
        table: Arc<dyn Table>,
        ctx: Arc<QueryContext>,
    ) -> Result<ProcessorPtr> {
        let mut default_exprs = Vec::new();
        let mut default_nonexpr_fields = Vec::new();

        let fields = table
            .schema()
            .fields()
            .iter()
            .map(DataField::from)
            .collect::<Vec<_>>();

        let mut unresort_fields = input_schema.fields().to_vec();

        for f in fields.iter() {
            if !input_schema.has_field(f.name()) {
                if let Some(default_expr) = f.default_expr() {
                    let expr = parse_exprs(ctx.clone(), table.clone(), default_expr)?;
                    let expr = expr[0].clone();
                    default_exprs.push(BlockOperator::Map { expr });
                    unresort_fields.push(f.clone());
                } else {
                    default_nonexpr_fields.push(f.clone());
                }
            }
        }

        unresort_fields.extend_from_slice(&default_nonexpr_fields);

        let func_ctx = ctx.try_get_function_context()?;
        let expression_transform = CompoundBlockOperator {
            ctx: func_ctx,
            operators: default_exprs,
        };

        Ok(Transformer::create(input, output, Self {
            default_nonexpr_fields,
            expression_transform,
            output_schema: Arc::new(DataSchema::from(table.schema())),
            unresort_schema: DataSchemaRefExt::create(unresort_fields),
        }))
    }
}

impl Transform for TransformAddOn {
    const NAME: &'static str = "AddOnTransform";

    fn transform(&mut self, mut block: DataBlock) -> Result<DataBlock> {
        block = self.expression_transform.transform(block.clone())?;

        for f in &self.default_nonexpr_fields {
            let default_value = f.data_type().default_value();
            let column = BlockEntry {
                data_type: f.data_type().clone(),
                value: Value::Scalar(default_value),
            };
            block.add_column(column);
        }

        block = block.resort(&self.unresort_schema, &self.output_schema)?;

        Ok(block)
    }
}
