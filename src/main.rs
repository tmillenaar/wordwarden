use regex::{Captures, Regex};
use std::env;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::process;

const BOLD_START: &str = "\x1b[1m";
const BOLD_END: &str = "\x1b[0m";

#[derive(Debug)]
struct Occurance {
    filename: String,
    line_number: usize,
    target_string: String,
    line_content: String,
}

fn highlight_text(line: &str, highlight_text: &str) -> String {
    let regex_pattern = format!(r"(?i){}", regex::escape(highlight_text));
    let re = Regex::new(&regex_pattern).unwrap();

    re.replace_all(line, |captures: &Captures| {
        format!("{}{}{}", BOLD_START, &captures[0], BOLD_END)
    })
    .to_string()
}

fn check_file(
    filename: &Path,
    results: &mut Vec<Occurance>,
    target: &str,
    check_case: bool,
    escape: &str,
) -> io::Result<bool> {
    let file = File::open(filename)?;
    let reader = io::BufReader::new(file);
    let mut found = false;

    for (index, line) in reader.lines().enumerate() {
        if let Ok(line) = line {
            let mut target_in_line: bool;
            if line.contains(escape) {
                continue;
            }
            if check_case {
                target_in_line = line.contains(target);
            } else {
                let regex_pattern = format!(r"(?i){}", regex::escape(target));
                let re = Regex::new(&regex_pattern).unwrap();
                target_in_line = re.is_match(&line);
            }
            if target_in_line {
                let occurance = Occurance {
                    filename: filename.to_str().unwrap().to_owned(),
                    line_number: index + 1,
                    target_string: target.to_string(),
                    line_content: line,
                };
                results.push(occurance);
                found = true;
            }
        }
    }

    Ok(found)
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!(
            "
            Usage: {} <file1> <file2> 'word1' 'word2' 'word3']
            Use --casecheck and --no-casecheck to determine whether the check should be case-sensitive.
                The default is to not be case sensitive.
            Use '--escape=skip-this-line' to ignore occurances of words found on a line with the specified escape string.
                The default escape string is 'noqa:skip-line'
        ",
            args[0]
        );
        process::exit(2);
    }

    let mut filepaths: Vec<&Path> = Vec::new();
    let mut search_strings: Vec<&String> = Vec::new();
    let mut check_case: bool = false;
    let mut escape: String = "noqa:skip-line".to_string();

    for arg in &args[1..] {
        let path = Path::new(arg);
        if path.is_file() {
            filepaths.push(path);
        } else if arg.starts_with("--casecheck") {
            check_case = true;
        } else if arg.starts_with("--no-casecheck") {
            check_case = false;
        } else if arg.starts_with("--escape=") {
            escape = arg.replace("--escape=", "");
        } else {
            search_strings.push(arg);
        }
    }

    let mut found_any = false;
    let mut results: Vec<Occurance> = Vec::new();
    for path in filepaths {
        for target in &search_strings {
            match check_file(path, &mut results, target, check_case, &escape) {
                Ok(found) => {
                    if found {
                        found_any = true;
                    }
                }
                Err(err) => {
                    eprintln!("Error reading '{}': {}", path.to_str().unwrap_or("?"), err);
                    process::exit(2);
                }
            }
        }
    }

    let extra_line_space = 1;
    let max_line_length = &results
        .iter()
        .map(|r| (extra_line_space + r.filename.len() + r.line_number.to_string().len()))
        .max()
        .unwrap_or(0);
    for result in results {
        let filename_and_line_number = format!("{}:{}", result.filename, result.line_number);
        let print_line = format!(
            "{:<width$} -> {}",
            filename_and_line_number,
            highlight_text(&result.line_content, &result.target_string),
            width = max_line_length
        );
        println!("{}", print_line);
    }

    process::exit(if found_any { 1 } else { 0 });
}
