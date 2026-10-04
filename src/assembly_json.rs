//! Read typed assembly evidence without a managed-runtime dependency.
use crate::{
    asm::{AddressMode, Modifier, Operand},
    expr::{Arg, Expr, Position},
    json::Value,
};
use std::collections::BTreeMap;
fn object(v: &Value) -> Result<&BTreeMap<String, Value>, String> {
    if let Value::Object(o) = v {
        Ok(o)
    } else {
        Err("expected JSON object".into())
    }
}
fn array(v: &Value) -> Result<&[Value], String> {
    if let Value::Array(a) = v {
        Ok(a)
    } else {
        Err("expected JSON array".into())
    }
}
fn string(v: &Value) -> Result<&str, String> {
    if let Value::String(s) = v {
        Ok(s)
    } else {
        Err("expected JSON string".into())
    }
}
fn integer(v: &Value) -> Result<i32, String> {
    if let Value::Number(n) = v {
        i32::try_from(*n).map_err(|_| "assembly integer exceeds 32 bits".into())
    } else {
        Err("expected JSON integer".into())
    }
}
fn field<'a>(o: &'a BTreeMap<String, Value>, name: &str) -> Result<&'a Value, String> {
    o.get(name)
        .ok_or_else(|| format!("missing JSON field {name}"))
}
fn mode(text: &str) -> Result<AddressMode, String> {
    use AddressMode::*;
    Ok(match text {
        "Implicit" => Implicit,
        "Immediate" => Immediate,
        "Immediate16" => Immediate16,
        "HighMem" => HighMem,
        "HighMemX" => HighMemX,
        "HighMemY" => HighMemY,
        "Absolute" => Absolute,
        "AbsoluteX" => AbsoluteX,
        "AbsoluteY" => AbsoluteY,
        "Indirect" => Indirect,
        "IndirectX" => IndirectX,
        "IndirectY" => IndirectY,
        "Relative" => Relative,
        _ => return Err(format!("unknown assembly addressing mode {text}")),
    })
}
fn arg(v: &Value) -> Result<Arg, String> {
    match v {
        Value::String(s) => Ok(Arg::Text(s.clone())),
        Value::Number(_) => Ok(Arg::Int(integer(v)?)),
        Value::Array(a) => Ok(Arg::Exprs(a.iter().map(expr).collect::<Result<_, _>>()?)),
        Value::Object(o) if o.contains_key("args") => Ok(Arg::from(expr(v)?)),
        Value::Object(o) => match string(field(o, "type")?)? {
            "bytes" => Ok(Arg::Bytes(
                array(field(o, "value")?)?
                    .iter()
                    .map(|n| u8::try_from(integer(n)?).map_err(|_| "invalid assembly byte".into()))
                    .collect::<Result<_, String>>()?,
            )),
            "ints" => Ok(Arg::Ints(
                array(field(o, "value")?)?
                    .iter()
                    .map(integer)
                    .collect::<Result<_, _>>()?,
            )),
            "operand" => {
                let base = match field(o, "base")? {
                    Value::Null => None,
                    v => Some(string(v)?.to_owned()),
                };
                let modifier = match string(field(o, "modifier")?)? {
                    "None" => Modifier::None,
                    "LowByte" => Modifier::LowByte,
                    "HighByte" => Modifier::HighByte,
                    "Bank" => Modifier::Bank,
                    s => return Err(format!("unknown assembly modifier {s}")),
                };
                let comment = match o.get("comment") {
                    Some(Value::String(s)) => Some(s.clone()),
                    _ => None,
                };
                Ok(Arg::Operand(Operand {
                    base,
                    offset: integer(field(o, "offset")?)?,
                    mode: mode(string(field(o, "mode")?)?)?,
                    modifier,
                    comment,
                }))
            }
            kind => Err(format!("unsupported assembly JSON value: {kind}")),
        },
        _ => Err("unsupported assembly JSON value".into()),
    }
}
fn expr(v: &Value) -> Result<Expr, String> {
    let o = object(v)?;
    let args = array(field(o, "args")?)?
        .iter()
        .map(arg)
        .collect::<Result<Vec<_>, _>>()?;
    if !matches!(args.first(), Some(Arg::Text(_))) {
        return Err("assembly node has no tag".into());
    }
    let source = if let Some(source) = o.get("source") {
        let o = object(source)?;
        Position {
            filename: string(field(o, "filename")?)?.to_owned(),
            line: integer(field(o, "line")?)?.max(0) as usize,
            column: integer(field(o, "column")?)?.max(0) as usize,
        }
    } else {
        Position::default()
    };
    Ok(Expr { args, source })
}
pub fn parse(text: &str) -> Result<Vec<Expr>, String> {
    array(&Value::parse(text)?)?.iter().map(expr).collect()
}
