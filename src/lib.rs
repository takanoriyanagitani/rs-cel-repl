use cel::{Program, Value as CelValueEnum};
use rustyline::error::ReadlineError;
use serde_json::{Map, Value as JsonValue};
use std::{fmt, io};

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Readline(ReadlineError),
    Cel(String),
    SerdeJson(serde_json::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(err) => write!(f, "IO error: {}", err),
            Error::Readline(err) => write!(f, "Readline error: {}", err),
            Error::Cel(err) => write!(f, "CEL error: {}", err),
            Error::SerdeJson(err) => write!(f, "JSON error: {}", err),
        }
    }
}

impl std::error::Error for Error {}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<ReadlineError> for Error {
    fn from(err: ReadlineError) -> Self {
        Error::Readline(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::SerdeJson(err)
    }
}

pub struct CelProgram(pub Program);

impl CelProgram {
    pub fn execute(&self, ctx: &cel::Context) -> Result<CelValueEnum, Error> {
        self.0.execute(ctx).map_err(|e| Error::Cel(e.to_string()))
    }

    pub fn execute_with_value(
        &self,
        ctx: &cel::Context,
        vname: &str,
        val: JsonValue,
    ) -> Result<CelValueEnum, Error> {
        let mut child: cel::Context = ctx.new_inner_scope();
        child
            .add_variable(vname, val)
            .map_err(|e| Error::Cel(e.to_string()))?;
        self.execute(&child)
    }
}

pub fn compile(expr: &str) -> Result<CelProgram, Error> {
    Program::compile(expr)
        .map(CelProgram)
        .map_err(|e| Error::Cel(e.to_string()))
}

pub struct CelValue(pub CelValueEnum);

impl From<CelValue> for JsonValue {
    fn from(wrapper_cel_value: CelValue) -> Self {
        let cel_value = wrapper_cel_value.0;
        match cel_value {
            CelValueEnum::Null => JsonValue::Null,
            CelValueEnum::Bool(b) => JsonValue::Bool(b),
            CelValueEnum::Int(i) => serde_json::json!(i),
            CelValueEnum::UInt(u) => serde_json::json!(u),
            CelValueEnum::Float(f) => serde_json::json!(f),
            CelValueEnum::String(s) => JsonValue::String(s.to_string()),
            CelValueEnum::Bytes(b) => JsonValue::String(String::from_utf8_lossy(&b).to_string()),
            CelValueEnum::List(list) => {
                let values: &[CelValueEnum] = &list;
                JsonValue::Array(values.iter().map(|v| CelValue(v.clone()).into()).collect())
            }
            CelValueEnum::Map(map_obj) => {
                let mut json_map = Map::new();
                for (key, val) in map_obj.map.iter() {
                    let cel_key_value: CelValueEnum = key.into();

                    let key_str = match cel_key_value {
                        CelValueEnum::String(s) => s.to_string(),
                        _ => format!("{:?}", cel_key_value),
                    };
                    json_map.insert(key_str, CelValue(val.clone()).into());
                }
                JsonValue::Object(json_map)
            }
            cel_value => JsonValue::String(format!("{:?}", cel_value)),
        }
    }
}
