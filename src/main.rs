use std::collections::HashMap;

#[derive(Debug)]
enum JSONParseError {
    Error,
    NotFound,
    UnexpectedChar,
    MissingClosing,
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

fn string(mut src: &str) -> Result<(&str, JSONValue), JSONParseError> {
    // TODO: implement to spec

    // first we must parse the " character

    match src.strip_prefix("\"") {
        Some(rest) => src = rest,
        None => return Err(JSONParseError::NotFound),
    };

    // now we keep going until we find the first "
    // lets just "find" the first "

    match src.find("\"") {
        Some(index) => {
            let s = src[..index].to_string();
            let rest = &src[index + 1..];
            Ok((rest, JSONValue::String(s)))
        }
        None => Err(JSONParseError::MissingClosing),
    }
}

fn number(src: &str) -> Result<(&str, JSONValue), JSONParseError> {
    // TODO: Actually support correct grammar
    // hacky version: just matches 0 to 9 for now

    let len_following_number = src.trim_start_matches(char::is_numeric).len();
    let num_chars_in_number = src.len() - len_following_number;

    if num_chars_in_number == 0 {
        return Err(JSONParseError::NotFound);
    }

    // get the first digit_chars characters
    let digits = &src[..num_chars_in_number];
    let rest = &src[num_chars_in_number..];

    // TODO: Error Handling
    let value = digits.parse::<i128>().unwrap(); // https://doc.rust-lang.org/std/string/struct.String.html#method.parse

    Ok((rest, JSONValue::Number(value)))
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

    match string(src) {
        Ok(res) => return Ok(res),
        Err(JSONParseError::NotFound) => {} // if not found, that ok
        Err(e) => return Err(e),            // if any other error, propogate it up
    }

    match number(src) {
        Ok(res) => return Ok(res),
        Err(JSONParseError::NotFound) => {} // if not found, that ok
        Err(e) => return Err(e),            // if any other error, propogate it up
    }

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

    let sample = "   \"322893784aksjdfhkja\" null    false   ";
    // let sample = "    false       ";

    println!("Source is \"{:}\"", sample);

    println!("Parser says {:?}", parse(sample));
}
