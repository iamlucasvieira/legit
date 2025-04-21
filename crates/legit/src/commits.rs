// use itertools::Itertools;
// use std::collections::HashMap;
// use anyhow::Result;
// pub struct Commit {
//     pub tree: String,
//     pub parents: Vec<String>,
//     pub author: String,
//     pub committer: String,
//     pub gpgsig: Option<String>,
//     pub message: String,
// }
//
// /// Parse a list of key value pairs from a list of strings
// ///
// /// Lines is a list of string, each line contain a key space value
// /// if the next line is empty, it means the value is multi-line
// /// and we need to append the next line to the value until we find a new line
// pub fn parse_key_values(lines: &[&str]) -> Result<HashMap<String, String>> {
//     let mut map = HashMap::new();
//     let mut remaining = lines;
//     while !remaining.is_empty() {
//         remaining = parse_key_value(remaining, &mut map)?;
//     }
//     Ok(map)
// }
//
// /// Parse a single key value pair and return the remaining lines
// ///
// /// Lines is a list of string, each line contain a key space value
// /// if the next line is empty, it means the value is multi-line
// /// and we need to append the next line to the value until we find a new line
// fn parse_key_value<'a>(
//     lines: &'a [&str],
//     map: &mut HashMap<String, String>,
// ) -> Result<&'a [&'a str]> {
//     let mut lines_consumed = 1;
//     let (key, value) = lines
//         .iter()
//         .take(1)
//         .map(|line| line.split_whitespace().collect_tuple::<(&str, &str)>())
//         .collect_tuple::<(&str, &str)>()
//         .ok_or(anyhow::anyhow!("No key value pair found"))?;
//
//     let mut value_str = value.to_string();
//
//     // TODO: Fix this logic - it's not correctly handling multiline values
//     let mut remaining = &lines[1..];
//     while !remaining.is_empty() && remaining[0].is_empty() {
//         if remaining.len() > 1 {
//             value_str.push_str(&remaining[1]);
//             lines_consumed += 2;
//             remaining = &remaining[2..];
//         } else {
//             break;
//         }
//     }
//
//     map.insert(key.to_string(), value_str);
//     Ok(&lines[lines_consumed..])
// }
//
// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn test_parse_key_values() {
//         let text = "
// key value
//
// line2
// ";
//         let lines = text.lines().map(|s| s.to_string()).collect::<Vec<_>>();
//         let map = parse_key_values(&lines).unwrap();
//         assert_eq!(map.get("key"), Some(&"value".to_string()));
//         assert_eq!(map.get("line2"), Some(&"line2".to_string()));
//     }
// }
