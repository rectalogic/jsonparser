use colored::Colorize;
use std::{cmp::max, collections::HashMap, fs};

#[derive(Debug)]
enum JSONParseError {
    Error(usize),
    NotFound,
    UnexpectedChar(usize),
    MissingClosing(usize),
}

#[derive(Debug, Clone, PartialEq)]
enum JSONValue {
    Null,
    True,
    False,
    Number(f64),
    String(String),
    Array(Vec<JSONValue>),
    Object(HashMap<String, JSONValue>),
}

// consume whitespace and return the remaining string
fn ws(src: &str) -> &str {
    src.trim_start_matches([' ', '\n', '\t', '\r'])
}

fn string(mut src: &str) -> Result<(&str, JSONValue), JSONParseError> {
    match src.strip_prefix("\"") {
        Some(rest) => src = rest,
        None => return Err(JSONParseError::NotFound),
    };

    // now we keep going until we find the first "
    // lets just "find" the first "

    let mut result: String = "".to_string();
    let mut escaping = false;

    let mut chars = src.chars();

    loop {
        let c = match chars.next() {
            Some(c) => c,
            None => return Err(JSONParseError::MissingClosing(src.len())),
        };

        // if we have the \, then we are escaping, but don't add anything to result
        if c == '\\' && !escaping {
            escaping = true;
        }
        // if we have the end quote but we are not escaping, then we are done
        else if c == '"' && !escaping {
            break;
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
                    return Err(JSONParseError::UnexpectedChar(chars.count()));
                }
            }

            escaping = false;
        } else {
            result.push(c);
        }
    }

    Ok((chars.as_str(), JSONValue::String(result)))
}

// numbers are weird

fn onenine(src: &str) -> Result<(&str, char), JSONParseError> {
    // check the first character of the string
    match src.chars().next() {
        // if the character exists
        Some(c) => {
            // check if it is numeric
            if c.is_numeric() {
                // if it is, we have to make sure it's not 0
                if c == '0' {
                    return Err(JSONParseError::NotFound);
                }
                Ok((&src[1..], c))
            } else {
                Err(JSONParseError::NotFound)
            }
        }
        None => Err(JSONParseError::NotFound),
    }
}

fn digit(src: &str) -> Result<(&str, char), JSONParseError> {
    // check the first character of the string
    match src.chars().next() {
        // if the character exists
        Some('0') => Ok((&src[1..], '0')),
        Some(_) => onenine(src),
        None => Err(JSONParseError::NotFound),
    }
}

fn digits(mut src: &str) -> Result<(&str, Vec<char>), JSONParseError> {
    let mut res = vec![];
    while let Ok((rest, c)) = digit(src) {
        src = rest;
        res.push(c);
    }

    if res.is_empty() {
        return Err(JSONParseError::NotFound);
    }

    Ok((src, res))
}

fn integer(mut src: &str) -> Result<(&str, i64), JSONParseError> {
    // first check for negative symbol.
    let negative;

    match src.strip_prefix("-") {
        Some(rest) => {
            src = rest;
            negative = true;
        }
        None => {
            negative = false;
        }
    }

    // try to parse onenine, then digits
    if let Ok((rest, c)) = onenine(src) {
        if let Ok((leftover, mut digis)) = digits(rest) {
            digis.insert(0, c);
            let int_str: String = digis.iter().collect();
            let mut resulting_int: i64 = int_str.parse::<i64>().unwrap();
            if negative {
                resulting_int *= -1;
            }
            return Ok((leftover, resulting_int));
        }
    }

    match digit(src) {
        Ok((rest, c)) => {
            let mut n: i64 = c.to_digit(10).unwrap().into();

            if negative {
                n *= -1;
            }

            Ok((rest, n))
        }

        Err(e) => Err(e),
    }
}

fn fraction(src: &str) -> Result<(&str, f64), JSONParseError> {
    match src.strip_prefix(".") {
        Some(rest) => match digits(rest) {
            Ok((leftover, mut digis)) => {
                digis.insert(0, '.');
                digis.insert(0, '0');

                let fraction_str: String = digis.iter().collect();
                let fraction_part = fraction_str.parse::<f64>().unwrap();
                Ok((leftover, fraction_part))
            }
            Err(e) => Err(e),
        },
        None => Ok((src, 0.0)),
    }
}

fn exponent(mut src: &str) -> Result<(&str, i64), JSONParseError> {
    let first_char = src.chars().next();
    if first_char == Some('e') || first_char == Some('E') {
        src = &src[1..];
    } else {
        return Ok((src, 0));
    }

    let mut negative = false;

    let sign_char = src.chars().next();
    if sign_char == Some('+') {
        // do nothing and skip
        src = &src[1..];
    } else if sign_char == Some('-') {
        negative = true;
        src = &src[1..];
    }

    // ok now digits
    match digits(src) {
        Ok((rest, digis)) => {
            let num_str: String = digis.iter().collect();
            let mut num: i64 = num_str.parse::<i64>().unwrap();
            if negative {
                num *= -1;
            }
            Ok((rest, num))
        }
        Err(e) => Err(e),
    }
}

fn number(mut src: &str) -> Result<(&str, JSONValue), JSONParseError> {
    let mut result;
    let negative;

    match integer(src) {
        Ok((rest, num)) => {
            result = num.abs() as f64;
            negative = num.is_negative();
            src = rest;
        }
        Err(e) => return Err(e),
    };

    match fraction(src) {
        Ok((rest, frac)) => {
            result += frac;
            src = rest;
        }
        Err(JSONParseError::NotFound) => {}
        Err(e) => return Err(e),
    }

    match exponent(src) {
        Ok((rest, exponent)) => {
            src = rest;

            let multipier = 10_f64.powf(exponent as f64);
            result *= multipier;
        }
        Err(JSONParseError::NotFound) => {}
        Err(e) => return Err(e),
    }

    if negative {
        result *= -1.0;
    }

    Ok((src, JSONValue::Number(result)))
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

        // now we wanna consume the first character of src
        // if it is a comma, or break otherwise
        if src.starts_with(',') {
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
    if let Some(rest) = src.strip_prefix(']') {
        return Ok((rest, JSONValue::Array(vec![])));
    }

    // otherwise, parse elemnts and return that

    match elements(src) {
        Ok((src, v)) => {
            if let Some(rest) = src.strip_prefix(']') {
                Ok((rest, JSONValue::Array(v)))
            } else {
                Err(JSONParseError::MissingClosing(src.len()))
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
    if let Some(rest) = src.strip_prefix('}') {
        // TODO:
        return Ok((rest, JSONValue::Object(HashMap::new())));
    }

    // otherwise, parse elemnts and return that

    match members(src) {
        Ok((src, v)) => {
            if let Some(rest) = src.strip_prefix('}') {
                let mut map: HashMap<String, JSONValue> = HashMap::new();

                v.iter().for_each(|(key, value)| {
                    map.insert(key.to_owned(), value.to_owned());
                });

                Ok((rest, JSONValue::Object(map)))
            } else {
                Err(JSONParseError::MissingClosing(src.len()))
            }
        }
        Err(e) => Err(e),
    }
}

#[allow(clippy::type_complexity)]
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
        if src.starts_with(',') {
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

            if src.starts_with(':') {
                src = &src[1..];
                match element(src) {
                    Ok((rest, el)) => Ok((rest, (key, el))),
                    Err(e) => Err(e),
                }
            } else {
                Err(JSONParseError::UnexpectedChar(src.len()))
            }
        }
        Ok((_, _)) => Err(JSONParseError::Error(src.len())),
        Err(e) => Err(e),
    }
}

fn parse(src: &str) -> Result<JSONValue, JSONParseError> {
    match element(src) {
        Ok((_, res)) => Ok(res),
        Err(e) => Err(e),
    }
}

fn main() {
    // open and read the broken.json file
    let text_file_contents = fs::read_to_string("broken.json").unwrap();
    let src = text_file_contents.as_str();

    match parse(src) {
        Ok(v) => {
            println!("{:?}", v);
        }
        Err(e) => {
            println!("{}", format!("Error: {:?}", e).normal().on_red());
            let pos = match e {
                JSONParseError::Error(p) => p,
                JSONParseError::UnexpectedChar(p) => p,
                JSONParseError::MissingClosing(p) => p,
                JSONParseError::NotFound => 0,
            };

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
            match e {
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
    }

    let big_file = std::fs::read_to_string("twitter.json").expect("Could not read file");

    // print!("{}", big_file);
    // let big_file = std::fs::read_to_string("canada.json").expect("Could not read file");

    // how many bytes of data?
    let num_bytes = big_file.len();

    let mul = 1000;
    let bytes_to_parse = num_bytes * mul;

    let start_time = std::time::Instant::now();
    for _ in 0..mul {
        let _ = parse(big_file.as_str());
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
            Ok((_, v)) => assert_eq!(v, super::JSONValue::True),
            Err(_) => panic!("Expected true"),
        }
    }

    #[test]
    fn bool_false() {
        match super::bool("false") {
            Ok((_, v)) => assert_eq!(v, super::JSONValue::False),
            Err(_) => panic!("Expected false"),
        }
    }

    #[test]
    fn json_bool_true() {
        match super::parse("true") {
            Ok(v) => assert_eq!(v, super::JSONValue::True),
            Err(_) => panic!("Expected true"),
        }
    }

    #[test]
    fn json_bool_false() {
        match super::parse("false") {
            Ok(v) => assert_eq!(v, super::JSONValue::False),
            Err(_) => panic!("Expected false"),
        }
    }

    #[test]
    fn json_null() {
        match super::parse("null") {
            Ok(v) => assert_eq!(v, super::JSONValue::Null),
            Err(_) => panic!("Expected null"),
        }
    }

    #[test]
    fn json_integer_positive() {
        match super::parse("123") {
            Ok(v) => assert_eq!(v, super::JSONValue::Number(123.0)),
            Err(_) => panic!("Expected 123"),
        }
    }

    #[test]
    fn json_integer_negative() {
        match super::parse("-123") {
            Ok(v) => assert_eq!(v, super::JSONValue::Number(-123.0)),
            Err(_) => panic!("Expected -123"),
        }
    }

    #[test]
    fn json_float_positive() {
        match super::parse("123.456") {
            Ok(v) => assert_eq!(v, super::JSONValue::Number(123.456)),
            Err(_) => panic!("Expected 123.456"),
        }
    }

    #[test]
    fn json_float_negative() {
        match super::parse("-123.456") {
            Ok(v) => assert_eq!(v, super::JSONValue::Number(-123.456)),
            Err(_) => panic!("Expected -123.456"),
        }
    }

    #[test]
    fn json_float_negative_exp() {
        match super::parse("-123.456e-2") {
            Ok(v) => assert_eq!(v, super::JSONValue::Number(-1.23456)),
            Err(_) => panic!("Expected -1.23456"),
        }
    }

    #[test]
    fn json_float_positive_exp() {
        match super::parse("123.456e2") {
            Ok(v) => assert_eq!(v, super::JSONValue::Number(12345.6)),
            Err(_) => panic!("Expected 12345.6"),
        }
    }

    #[test]
    fn json_basic_string() {
        match super::parse(r#""Hello, World!""#) {
            Ok(v) => assert_eq!(v, super::JSONValue::String("Hello, World!".to_string())),
            Err(_) => panic!("Expected \"Hello, World!\""),
        }
    }

    #[test]
    fn read_canada_json() {
        let contents =
            fs::read_to_string("canada.json").expect("Should have been able to read the file");
        match super::parse(contents.as_str()) {
            Ok(_) => {}
            Err(_) => panic!("Errored"),
        }
    }

    #[test]
    fn read_twitter_json() {
        let contents =
            fs::read_to_string("twitter.json").expect("Should have been able to read the file");
        match super::parse(contents.as_str()) {
            Ok(_) => {}
            Err(e) => {
                let err_str = format!("Error: {:?}", e);
                panic!("{}", err_str);
            }
        }
    }

    #[test]
    fn json_escaped_newline() {
        let src = r#" 
        
        "hi there\nthis is a test"
        
        "#;

        let expected = "hi there\nthis is a test";

        match super::parse(src) {
            Ok(v) => {
                assert_eq!(v, super::JSONValue::String(expected.to_string()));
            }
            Err(_) => panic!("Expected \"hi there\nthis is a test\""),
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
            Ok(v) => {
                assert_eq!(v, expected);
            }
            Err(_) => panic!("Expected [1, 2, 3, 4, 5]"),
        }
    }

    #[test]
    fn json_empty_list() {
        let src = r#"[]"#;

        let expected = super::JSONValue::Array(vec![]);

        match super::parse(src) {
            Ok(v) => {
                assert_eq!(v, expected);
            }
            Err(_) => panic!("Expected []"),
        }
    }
}
