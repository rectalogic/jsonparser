use std::collections::HashMap;

#[derive(Debug)]
enum JSONParseError {
    Error,
    NotFound,
    UnexpectedChar,
    MissingClosing,
}

#[derive(Debug, Clone)]
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
    src.trim_start_matches(&[' ', '\n', '\t', '\r'])
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

    match object(src) {
        Ok(res) => return Ok(res),
        Err(JSONParseError::NotFound) => {} // if not found, that ok
        Err(e) => return Err(e),
    }

    match array(src) {
        Ok(res) => return Ok(res),
        Err(JSONParseError::NotFound) => {} // if not found, that ok
        Err(e) => return Err(e),            // if any other error, propogate it up
    }

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

fn element(mut src: &str) -> Result<(&str, JSONValue), JSONParseError> {
    src = ws(src);

    match value(src) {
        Ok((rest, v)) => Ok((ws(rest), v)),
        Err(e) => Err(e),
    }
}

fn elements(mut src: &str) -> Result<(&str, Vec<JSONValue>), JSONParseError> {
    let mut values = vec![];

    loop {
        match element(src) {
            Ok((rest, v)) => {
                src = rest;
                values.push(v);
            }
            Err(e) => return Err(e),
        }

        // now we wanna consume the first character of src, if it is a comma
        // or break otherwise
        if src.chars().next() == Some(',') {
            src = &src[1..];
        } else {
            break;
        }
    }

    Ok((src, values))
}

fn array(mut src: &str) -> Result<(&str, JSONValue), JSONParseError> {
    // first we must parse the [] character

    match src.strip_prefix("[") {
        Some(rest) => src = ws(rest),
        None => return Err(JSONParseError::NotFound),
    };

    // if this is true... then we have just parsed whitespace and there are no elements.
    // thus, return empty array
    if src.chars().next() == Some(']') {
        src = &src[1..];

        return Ok((src, JSONValue::Array(vec![])));
    }

    // otherwise, parse elemnts and return that

    match elements(src) {
        Ok((src, v)) => {
            if src.chars().next() == Some(']') {
                Ok((&src[1..], JSONValue::Array(v)))
            } else {
                Err(JSONParseError::MissingClosing)
            }
        }
        Err(e) => Err(e),
    }
}

fn object(mut src: &str) -> Result<(&str, JSONValue), JSONParseError> {
    // first we must parse the [] character

    match src.strip_prefix("{") {
        Some(rest) => src = ws(rest),
        None => return Err(JSONParseError::NotFound),
    };

    // if this is true... then we have just parsed whitespace and there are no elements.
    // thus, return empty array
    if src.chars().next() == Some('}') {
        src = &src[1..];

        // TODO:
        return Ok((src, JSONValue::Object(HashMap::new())));
    }

    // otherwise, parse elemnts and return that

    match members(src) {
        Ok((src, v)) => {
            if src.chars().next() == Some('}') {
                let mut map: HashMap<String, JSONValue> = HashMap::new();

                v.iter().for_each(|(key, value)| {
                    map.insert(key.to_owned(), value.to_owned());
                });

                Ok((&src[1..], JSONValue::Object(map)))
            } else {
                Err(JSONParseError::MissingClosing)
            }
        }
        Err(e) => Err(e),
    }
}

fn members(mut src: &str) -> Result<(&str, Vec<(String, JSONValue)>), JSONParseError> {
    let mut values = vec![];

    loop {
        match member(src) {
            Ok((rest, v)) => {
                src = rest;
                values.push(v);
            }
            Err(e) => return Err(e),
        }

        // now we wanna consume the first character of src, if it is a comma
        // or break otherwise
        if src.chars().next() == Some(',') {
            src = &src[1..];
        } else {
            break;
        }
    }

    Ok((src, values))
}

fn member(mut src: &str) -> Result<(&str, (String, JSONValue)), JSONParseError> {
    src = ws(src);

    match string(src) {
        Ok((rest, JSONValue::String(key))) => {
            src = rest;
            src = ws(src);

            // now expect a ":"

            if src.chars().next() == Some(':') {
                src = &src[1..];
                match element(src) {
                    Ok((rest, el)) => return Ok((rest, (key, el))),
                    Err(e) => return Err(e),
                }
            } else {
                return Err(JSONParseError::UnexpectedChar);
            }
        }
        Ok((_, _)) => Err(JSONParseError::Error),
        Err(e) => Err(e),
    }
}

fn parse(mut src: &str) -> Result<JSONValue, JSONParseError> {
    match element(src) {
        Ok((_, res)) => Ok(res),
        Err(e) => Err(e),
    }
}

fn main() {
    println!("Hello, world!");

    // let sample = "[1,2,true,null,false,\"Hello, World!\"]";
    // let sample = "    false       ";

    // let sample = "{\"hi\": 3}";

    let sample = r#"

    {
        "name": "John",
        "1": [2, 4, null, true, false],
        "e": { "key": "value" }
    }
      
    "#;

    println!("Source is \"{:}\"", sample);

    println!("Parser says {:?}", parse(sample));
}

// TODO: Add Real Tests
