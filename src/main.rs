use std::collections::HashMap;

#[derive(Debug)]
enum JSONParseError {
    Error,
    NotFound,
}

#[derive(Debug)]
enum JSONValue {
    Null,
    True,
    False,
    Number(i128),
    String(String),
    Array(Vec<JSONValue>),
    Object(HashMap<String, JSONValue>),
}

struct ParserContext {}

// consume whitespace and return the remaining string
fn ws(src: &str) -> &str {
    src.trim_start_matches(|x| x == ' ')
}

fn bool(src: &str) -> Result<(&str, JSONValue), JSONParseError> {
    match src.strip_prefix("true") {
        Some(rest) => Ok((rest, JSONValue::True)),
        None => match src.strip_prefix("false") {
            Some(rest) => Ok((rest, JSONValue::False)),
            None => Err(JSONParseError::NotFound),
        },
    }
}

fn parse(src: &str) -> Result<JSONValue, JSONParseError> {
    let mut ctx = src;
    ctx = ws(ctx);
    match bool(ctx) {
        Ok((c, v)) => {
            ctx = c;
            println!("Bool found {:?}", v);
        }
        Err(_) => todo!(),
    }

    Ok(JSONValue::String(ctx.to_string()))
}

fn main() {
    println!("Hello, world!");
    // let sample =
    //     String::from("{\"1\":[2,4,null,true,false],\"name\":\"John\",\"e\":{\"key\":\"value\"}}");

    // let sample = "   true       ";
    let sample = "    false       ";

    println!("Source is \"{:}\"", sample);

    println!("Parser says {:?}", parse(sample));
}
