use std::collections::HashMap;

#[derive(Debug)]
enum JSONParseError {
    Error,
    NotFound,
    UnexpectedChar,
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

fn null(src: &str) -> Result<(&str, JSONValue), JSONParseError> {
    match src.strip_prefix("null") {
        Some(rest) => Ok((rest, JSONValue::Null)),
        None => Err(JSONParseError::NotFound),
    }
}

fn value(src: &str) -> Result<(&str, JSONValue), JSONParseError> {
    // TODO: Better Error Handling
    match bool(src) {
        Ok(res) => return Ok(res),
        Err(JSONParseError::NotFound) => {} // if not found, that ok
        Err(e) => return Err(e),            // if any other error, propogate it up
    };

    match null(src) {
        Ok(res) => return Ok(res),
        Err(JSONParseError::NotFound) => {} // if not found, that ok
        Err(e) => return Err(e),            // if any other error, propogate it up
    };

    Err(JSONParseError::NotFound)
}

fn parse(mut src: &str) -> Result<JSONValue, JSONParseError> {
    src = ws(src);
    match value(src) {
        Ok((_, res)) => Ok(res),
        Err(e) => Err(e),
    }
}

fn main() {
    println!("Hello, world!");
    // let sample =
    //     String::from("{\"1\":[2,4,null,true,false],\"name\":\"John\",\"e\":{\"key\":\"value\"}}");

    let sample = "   null    false   ";
    // let sample = "    false       ";

    println!("Source is \"{:}\"", sample);

    println!("Parser says {:?}", parse(sample));
}
