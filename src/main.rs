use colored::Colorize;
use std::{borrow::Cow, cmp::max, collections::HashMap, fs};

#[derive(Debug)]
enum JSONParseError {
    Error(usize),
    NotFound,
    UnexpectedChar(usize),
    MissingClosing(usize),
}

#[derive(Debug, Clone, PartialEq)]
enum JSONValue<'a> {
    Null,
    True,
    False,
    Number(f64),
    String(Cow<'a, str>),
    Array(Vec<JSONValue<'a>>),
    Object(HashMap<Cow<'a, str>, JSONValue<'a>>),
}

const WHITESPACE: &[char] = &[' ', '\n', '\t', '\r'];

// consume whitespace and return the remaining string
fn ws(src: &str) -> &str {
    src.trim_start_matches(WHITESPACE)
}

fn string(mut src: &str) -> Result<Option<(&str, JSONValue)>, JSONParseError> {
    match src.strip_prefix("\"") {
        Some(rest) => src = rest,
        None => return Ok(None),
    };

    // now we keep going until we find the first "
    // lets just "find" the first "

    let mut result = String::new();
    let mut escaping = false;

    let mut chars = src.char_indices();

    loop {
        let (i, c) = match chars.next() {
            Some(r) => r,
            None => return Err(JSONParseError::MissingClosing(src.len())),
        };

        // if we have the \, then we are escaping, but don't add anything to result
        if c == '\\' && !escaping {
            escaping = true;
            if result.is_empty() {
                result = src[..i].to_string();
            }
        }
        // if we have the end quote but we are not escaping, then we are done
        else if c == '"' && !escaping {
            let cow = if result.is_empty() {
                Cow::Borrowed(&src[..i])
            } else {
                Cow::Owned(result)
            };
            return Ok(Some((&src[i + 1..], JSONValue::String(cow))));
        } else if escaping {
            // if we are escaping, then we need to check for special characters

            match c {
                '"' => result.push('"'),        // quotation mark
                '\\' => result.push('\\'),      // reverse solidus
                '/' => result.push('/'),        // solidus
                'b' => result.push('\u{0008}'), // backspace
                'f' => result.push('\u{000c}'), // form feed
                'n' => result.push('\n'),       // line feed
                'r' => result.push('\r'),       // carriage return
                't' => result.push('\t'),       // tab
                _ => {
                    // can't escape whatever this is
                    return Err(JSONParseError::UnexpectedChar(src.len() - i));
                }
            }

            escaping = false;
        } else if !result.is_empty() {
            result.push(c);
        }
    }
}

fn number(src: &str) -> Result<Option<(&str, JSONValue)>, JSONParseError> {
    if !src.starts_with([
        '+', '-', '.', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
    ]) {
        return Ok(None);
    }
    let mut delimiters = Vec::from(WHITESPACE);
    delimiters.extend([']', '}', ',', ':']);
    let index = match src.find(&delimiters[..]) {
        None => src.len(),
        Some(index) => index,
    };

    let number = &src[..index]
        .parse::<f64>()
        .map_err(|_| JSONParseError::Error(src.len()))?;
    Ok(Some((&src[index..], JSONValue::Number(*number))))
}

fn bool(src: &str) -> Result<Option<(&str, JSONValue)>, JSONParseError> {
    Ok(src
        .strip_prefix("true")
        .map(|rest| (rest, JSONValue::True))
        .or_else(|| {
            src.strip_prefix("false")
                .map(|rest| (rest, JSONValue::False))
        }))
}

fn null(src: &str) -> Result<Option<(&str, JSONValue)>, JSONParseError> {
    Ok(src.strip_prefix("null").map(|rest| (rest, JSONValue::Null)))
}

fn value(src: &str) -> Result<Option<(&str, JSONValue)>, JSONParseError> {
    if let Some(res) = object(src)? {
        Ok(Some(res))
    } else if let Some(res) = array(src)? {
        Ok(Some(res)) // if any other error, propogate it up
    } else if let Some(res) = string(src)? {
        Ok(Some(res)) // if any other error, propogate it up
    } else if let Some(res) = number(src)? {
        Ok(Some(res)) // if any other error, propogate it up
    } else if let Some(res) = bool(src)? {
        Ok(Some(res)) // if any other error, propogate it up
    } else if let Some(res) = null(src)? {
        Ok(Some(res)) // if any other error, propogate it up
    } else {
        Ok(None)
    }
}

fn element(mut src: &str) -> Result<Option<(&str, JSONValue)>, JSONParseError> {
    src = ws(src);
    if let Some((rest, v)) = value(src)? {
        Ok(Some((ws(rest), v)))
    } else {
        Ok(None)
    }
}

fn elements(mut src: &str) -> Result<Option<(&str, Vec<JSONValue>)>, JSONParseError> {
    let mut values = vec![];

    loop {
        if let Some((rest, v)) = element(src)? {
            src = rest;
            values.push(v);
        } else {
            return Ok(None);
        }

        // now we wanna consume the first character of src
        // if it is a comma, or break otherwise
        if src.starts_with(',') {
            src = &src[1..];
        } else {
            break;
        }
    }

    Ok(Some((src, values)))
}

fn array(mut src: &str) -> Result<Option<(&str, JSONValue)>, JSONParseError> {
    // first we must parse the [] character

    match src.strip_prefix('[') {
        Some(rest) => src = ws(rest),
        None => return Ok(None),
    };

    // if this is true... then we have just parsed whitespace and there are no elements.
    // thus, return empty array
    if let Some(rest) = src.strip_prefix(']') {
        return Ok(Some((rest, JSONValue::Array(vec![]))));
    }

    // otherwise, parse elemnts and return that

    if let Some((src, v)) = elements(src)? {
        if let Some(rest) = src.strip_prefix(']') {
            Ok(Some((rest, JSONValue::Array(v))))
        } else {
            Err(JSONParseError::MissingClosing(src.len()))
        }
    } else {
        Ok(None)
    }
}

fn object(mut src: &str) -> Result<Option<(&str, JSONValue)>, JSONParseError> {
    // first we must parse the [] character

    match src.strip_prefix("{") {
        Some(rest) => src = ws(rest),
        None => return Ok(None),
    };

    // if this is true... then we have just parsed whitespace and there are no elements.
    // thus, return empty array
    if let Some(rest) = src.strip_prefix('}') {
        // TODO:
        return Ok(Some((rest, JSONValue::Object(HashMap::new()))));
    }

    // otherwise, parse elemnts and return that

    if let Some((src, v)) = members(src)? {
        if let Some(rest) = src.strip_prefix('}') {
            let mut map: HashMap<Cow<str>, JSONValue> = HashMap::new();

            v.into_iter().for_each(|(key, value)| {
                map.insert(key, value);
            });

            Ok(Some((rest, JSONValue::Object(map))))
        } else {
            Err(JSONParseError::MissingClosing(src.len()))
        }
    } else {
        Ok(None)
    }
}

#[allow(clippy::type_complexity)]
fn members(mut src: &str) -> Result<Option<(&str, Vec<(Cow<str>, JSONValue)>)>, JSONParseError> {
    let mut values = vec![];

    loop {
        if let Some((rest, v)) = member(src)? {
            src = rest;
            values.push(v);
        } else {
            return Ok(None);
        }

        // now we wanna consume the first character of src, if it is a comma
        // or break otherwise
        if src.starts_with(',') {
            src = &src[1..];
        } else {
            break;
        }
    }

    Ok(Some((src, values)))
}

#[allow(clippy::type_complexity)]
fn member(mut src: &str) -> Result<Option<(&str, (Cow<str>, JSONValue))>, JSONParseError> {
    src = ws(src);
    if let Some((rest, JSONValue::String(key))) = string(src)? {
        src = rest;
        src = ws(src);

        // now expect a ":"

        if src.starts_with(':') {
            if let Some((rest, el)) = element(&src[1..])? {
                Ok(Some((rest, (key, el))))
            } else {
                Ok(None)
            }
        } else {
            Err(JSONParseError::UnexpectedChar(src.len()))
        }
    } else {
        Ok(None)
    }
}

fn parse(src: &str) -> Result<Option<JSONValue>, JSONParseError> {
    Ok(element(src)?.map(|(_, value)| value))
}

fn format_error(src: &str, pos: usize, error: JSONParseError) {
    let total = src.len();
    let error_pos = total - pos;

    // lets get 2 lines from the src, one before and one of the error

    let lines = src.split("\n").collect::<Vec<&str>>();

    let mut leftover = error_pos;
    let mut line_index = 0;
    let mut last_line = "";
    let err_line;
    loop {
        let line = lines[line_index];
        let line_len = line.len();

        if leftover < line_len {
            err_line = line;
            break;
        } else {
            last_line = line;
            leftover -= line_len + 1;
            line_index += 1;
        }
    }

    // // print seperator -'s

    println!("{}", "-".repeat(max(last_line.len(), err_line.len())));
    println!("{}", last_line);
    println!("{}", err_line);

    // print an ascii arrow to point to the error
    for i in 0..3 {
        for _ in 0..(leftover) {
            print!(" ");
        }
        println!("{}", if i == 0 { "^" } else { "|" });
    }

    // print the error message
    match error {
        JSONParseError::Error(_) => println!(
            "{}",
            format!(
                "Error: {} on Line {} Char {}",
                "Error",
                line_index + 1,
                leftover
            )
            .red()
        ),
        JSONParseError::UnexpectedChar(_) => println!(
            "{}",
            format!(
                "Error: {} on Line {} Char {}",
                "Unexpected Character",
                line_index + 1,
                leftover
            )
            .red()
        ),
        JSONParseError::MissingClosing(_) => println!(
            "{}",
            format!(
                "Error: {} on Line {} Char {}",
                "Missing Closing",
                line_index + 1,
                leftover
            )
            .red()
        ),
        JSONParseError::NotFound => {
            println!("Error: Not Found")
        }
    }
}

fn handle_parse(src: &str, silent: bool) {
    match parse(src) {
        Ok(Some(v)) => {
            if !silent {
                dbg!(v);
            }
        }
        Ok(None) => format_error(src, 0, JSONParseError::NotFound),
        Err(e) => {
            println!("{}", format!("Error: {:?}", e).normal().on_red());
            match e {
                JSONParseError::Error(p) => format_error(src, p, e),
                JSONParseError::UnexpectedChar(p) => format_error(src, p, e),
                JSONParseError::MissingClosing(p) => format_error(src, p, e),
                JSONParseError::NotFound => format_error(src, 0, e),
            };
        }
    }
}

fn main() {
    // open and read the broken.json file
    let text_file_contents = fs::read_to_string("broken.json").unwrap();
    let src = text_file_contents.as_str();

    handle_parse(src, false);

    let big_file = std::fs::read_to_string("twitter.json").expect("Could not read file");

    // print!("{}", big_file);
    // let big_file = std::fs::read_to_string("canada.json").expect("Could not read file");

    // how many bytes of data?
    let num_bytes = big_file.len();

    let mul = 1000;
    let bytes_to_parse = num_bytes * mul;

    let start_time = std::time::Instant::now();
    for i in 0..mul {
        handle_parse(big_file.as_str(), i != 0);
    }
    let end_time = std::time::Instant::now();

    let bps = bytes_to_parse as f64 / (end_time - start_time).as_secs_f64();

    let mbs = (bytes_to_parse as f64) / (1_000_000.0);
    let mbps = mbs / (end_time - start_time).as_secs_f64();

    let gbs = (bytes_to_parse as f64) / (1_000_000_000.0);
    let gbps = gbs / (end_time - start_time).as_secs_f64();

    println!("Parsing speed: {:.2} Bytes/s", bps);
    println!("Parsing speed: {:.2} MB/s", mbps);
    println!("Parsing speed: {:.2} GB/s", gbps);
}

#[cfg(test)]
mod tests {
    use std::fs;

    #[test]
    fn ws_empty() {
        let result = super::ws("");
        assert_eq!(result, "");
    }

    #[test]
    fn ws_space() {
        let result = super::ws("\u{0020}");
        assert_eq!(result, "");
    }

    #[test]
    fn ws_linefeed() {
        let result = super::ws("\u{000A}");
        assert_eq!(result, "");
    }

    #[test]
    fn ws_tab() {
        let result = super::ws("\u{0009}");
        assert_eq!(result, "");
    }

    #[test]
    fn ws_carriage_return() {
        let result = super::ws("\u{000D}");
        assert_eq!(result, "");
    }

    #[test]
    fn bool_true() {
        match super::bool("true") {
            Ok(Some((_, v))) => assert_eq!(v, super::JSONValue::True),
            Ok(None) | Err(_) => panic!("Expected true"),
        }
    }

    #[test]
    fn bool_false() {
        match super::bool("false") {
            Ok(Some((_, v))) => assert_eq!(v, super::JSONValue::False),
            Ok(None) | Err(_) => panic!("Expected false"),
        }
    }

    #[test]
    fn json_bool_true() {
        match super::parse("true") {
            Ok(Some(v)) => assert_eq!(v, super::JSONValue::True),
            Ok(None) | Err(_) => panic!("Expected true"),
        }
    }

    #[test]
    fn json_bool_false() {
        match super::parse("false") {
            Ok(Some(v)) => assert_eq!(v, super::JSONValue::False),
            Ok(None) | Err(_) => panic!("Expected false"),
        }
    }

    #[test]
    fn json_null() {
        match super::parse("null") {
            Ok(Some(v)) => assert_eq!(v, super::JSONValue::Null),
            Ok(None) | Err(_) => panic!("Expected null"),
        }
    }

    #[test]
    fn json_integer_positive() {
        match super::parse("123") {
            Ok(Some(v)) => assert_eq!(v, super::JSONValue::Number(123.0)),
            Ok(None) | Err(_) => panic!("Expected 123"),
        }
    }

    #[test]
    fn json_integer_negative() {
        match super::parse("-123") {
            Ok(Some(v)) => assert_eq!(v, super::JSONValue::Number(-123.0)),
            Ok(None) | Err(_) => panic!("Expected -123"),
        }
    }

    #[test]
    fn json_float_positive() {
        match super::parse("123.456") {
            Ok(Some(v)) => assert_eq!(v, super::JSONValue::Number(123.456)),
            Ok(None) | Err(_) => panic!("Expected 123.456"),
        }
    }

    #[test]
    fn json_float_negative() {
        match super::parse("-123.456") {
            Ok(Some(v)) => assert_eq!(v, super::JSONValue::Number(-123.456)),
            Ok(None) | Err(_) => panic!("Expected -123.456"),
        }
    }

    #[test]
    fn json_float_negative_exp() {
        match super::parse("-123.456e-2") {
            Ok(Some(v)) => assert_eq!(v, super::JSONValue::Number(-1.23456)),
            Ok(None) | Err(_) => panic!("Expected -1.23456"),
        }
    }

    #[test]
    fn json_float_positive_exp() {
        match super::parse("123.456e2") {
            Ok(Some(v)) => assert_eq!(v, super::JSONValue::Number(12345.6)),
            Ok(None) | Err(_) => panic!("Expected 12345.6"),
        }
    }

    #[test]
    fn json_basic_string() {
        match super::parse(r#""Hello, World!""#) {
            Ok(Some(v)) => assert_eq!(
                v,
                super::JSONValue::String(super::Cow::from("Hello, World!"))
            ),
            Ok(None) | Err(_) => panic!("Expected \"Hello, World!\""),
        }
    }

    #[test]
    fn read_canada_json() {
        let contents =
            fs::read_to_string("canada.json").expect("Should have been able to read the file");
        match super::parse(contents.as_str()) {
            Ok(Some(_)) => {}
            Ok(None) | Err(_) => panic!("Errored"),
        }
    }

    #[test]
    fn read_twitter_json() {
        let contents =
            fs::read_to_string("twitter.json").expect("Should have been able to read the file");
        match super::parse(contents.as_str()) {
            Ok(Some(_)) => {}
            Ok(None) | Err(_) => panic!("Errored"),
        }
    }

    #[test]
    fn json_escaped_newline() {
        let src = r#"

        "hi there\nthis is a test"

        "#;

        let expected = "hi there\nthis is a test";

        match super::parse(src) {
            Ok(Some(v)) => {
                assert_eq!(v, super::JSONValue::String(super::Cow::from(expected)));
            }
            Ok(None) | Err(_) => panic!("Expected \"hi there\nthis is a test\""),
        }
    }

    #[test]
    fn json_list_of_numbers() {
        let src = r#"[1, 2, 3, 4, 5]"#;

        let expected = super::JSONValue::Array(vec![
            super::JSONValue::Number(1.0),
            super::JSONValue::Number(2.0),
            super::JSONValue::Number(3.0),
            super::JSONValue::Number(4.0),
            super::JSONValue::Number(5.0),
        ]);

        match super::parse(src) {
            Ok(Some(v)) => {
                assert_eq!(v, expected);
            }
            Ok(None) | Err(_) => panic!("Expected [1, 2, 3, 4, 5]"),
        }
    }

    #[test]
    fn json_empty_list() {
        let src = r#"[]"#;

        let expected = super::JSONValue::Array(vec![]);

        match super::parse(src) {
            Ok(Some(v)) => {
                assert_eq!(v, expected);
            }
            Ok(None) | Err(_) => panic!("Expected []"),
        }
    }
}
