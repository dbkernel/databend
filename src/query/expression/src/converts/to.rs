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

use common_arrow::arrow::datatypes::Field as ArrowField;
use common_datavalues::DataTypeImpl;
use common_datavalues::DataValue;
use ordered_float::OrderedFloat;

use crate::Scalar;
use crate::TableDataType;
use crate::TableField;
use crate::TableSchema;

pub fn to_type(datatype: &TableDataType) -> DataTypeImpl {
    let f = TableField::new("tmp", datatype.clone());
    let arrow_f: ArrowField = (&f).into();
    common_datavalues::from_arrow_field(&arrow_f)
}

pub fn to_schema(schema: &TableSchema) -> common_datavalues::DataSchema {
    let fields = schema
        .fields()
        .iter()
        .map(|f| {
            let ty = to_type(f.data_type());
            common_datavalues::DataField::new(f.name(), ty)
                .with_default_expr(f.default_expr().cloned())
        })
        .collect();
    common_datavalues::DataSchema::new_from(fields, schema.meta().clone())
}

pub fn scalar_to_datavalue(scalar: &Scalar) -> DataValue {
    match scalar {
        Scalar::Null => DataValue::Null,
        Scalar::EmptyArray => DataValue::Null,
        Scalar::Number(ty) => match ty {
            crate::types::number::NumberScalar::UInt8(x) => DataValue::UInt64(*x as u64),
            crate::types::number::NumberScalar::UInt16(x) => DataValue::UInt64(*x as u64),
            crate::types::number::NumberScalar::UInt32(x) => DataValue::UInt64(*x as u64),
            crate::types::number::NumberScalar::UInt64(x) => DataValue::UInt64(*x),
            crate::types::number::NumberScalar::Int8(x) => DataValue::Int64(*x as i64),
            crate::types::number::NumberScalar::Int16(x) => DataValue::Int64(*x as i64),
            crate::types::number::NumberScalar::Int32(x) => DataValue::Int64(*x as i64),
            crate::types::number::NumberScalar::Int64(x) => DataValue::Int64(*x),
            crate::types::number::NumberScalar::Float32(x) => {
                DataValue::Float64(<OrderedFloat<f32> as Into<f32>>::into(*x) as f64)
            }
            crate::types::number::NumberScalar::Float64(x) => DataValue::Float64((*x).into()),
        },
        Scalar::Timestamp(x) => DataValue::Int64(*x),
        Scalar::Date(x) => DataValue::Int64(*x as i64),
        Scalar::Boolean(x) => DataValue::Boolean(*x),
        Scalar::String(x) | Scalar::Variant(x) => DataValue::String(x.clone()),
        Scalar::Array(x) => {
            let values = (0..x.len())
                .map(|idx| scalar_to_datavalue(&x.index(idx).unwrap().to_owned()))
                .collect();
            DataValue::Array(values)
        }
        Scalar::Tuple(x) => {
            let values = x.iter().map(scalar_to_datavalue).collect();
            DataValue::Struct(values)
        }
    }
}
