use regex::{Captures, Regex};
use std::env;
use std::fs::{read_dir, File};
use std::io::{self, BufRead};
use std::path::Path;
use std::path::PathBuf;
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

fn files_in_dir(dir: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut files: Vec<PathBuf> = Vec::new();

    for entry in read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            files.push(path);
        } else if path.is_dir() {
            match files_in_dir(path.as_path()) {
                Ok(entries) => files.extend(entries),
                _ => {} // Don't check any files that cause errors when checking if they are a file
            };
        }
    }

    Ok(files)
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

    if args.len() < 2 || args.contains(&"-h".to_string()) || args.contains(&"--help".to_string()) {
        eprintln!(
            "
            Usage: {} <file1> <file2> 'word1' 'word2' 'word3'
            Use --casecheck and --no-casecheck to determine whether the check should be case-sensitive.
                The default is to not be case sensitive.
            Use '--escape=skip-this-line' to ignore occurances of words found on a line with the specified escape string.
                The default escape string is 'wordwarden:skip-line'
            Use -w to force the interpretation of the argument as a word if it also happens to be the name of
                a file or directory. This might look like: {} examples -w 'examples'
                In that example we check for the word 'examples' in the files in the folder called examples
        ",
            args[0], args[0]
        );
        process::exit(2);
    }

    let mut filepaths: Vec<PathBuf> = Vec::new();
    let mut search_strings: Vec<&String> = Vec::new();
    let mut check_case: bool = false;
    let mut escape: String = "wordwarden:skip-line".to_string();

    let mut i = 1;
    while i <= args[1..].len() {
        let arg = &args[i];
        let path = Path::new(&arg);
        if path.is_file() {
            // For better integration with pre-commit, don't check the .pre-commit-config.yaml
            // for occurences because by the way the hook is set up, you specify the arguments
            // to this package in that file. If we did not hardcode it here every user would
            // need to use an escape entry in that config file.
            if (path.file_name().unwrap() != ".pre-commit-config.yaml")
                && (path.file_name().unwrap() != ".pre-commit-config.yml")
            {
                filepaths.push(path.to_path_buf())
            }
        } else if path.is_dir() {
            match files_in_dir(path) {
                Ok(entries) => filepaths.extend(entries),
                _ => {} // Don't check any files that cause errors when checking if they are a file
            };
        } else if arg.starts_with("--casecheck") {
            check_case = true;
        } else if arg.starts_with("--no-casecheck") {
            check_case = false;
        } else if arg.starts_with("--escape=") {
            escape = arg.replace("--escape=", "");
        } else if arg.starts_with("-w") {
            // Treat -w as the precursor for a word to check, append the next word to the search_strings vec
            i += 1;
            search_strings.push(&args[i])
        } else {
            search_strings.push(&arg);
        }
        i += 1;
    }

    let mut found_any = false;
    let mut results: Vec<Occurance> = Vec::new();
    for path in filepaths {
        for target in &search_strings {
            match check_file(
                &path.to_path_buf(),
                &mut results,
                target,
                check_case,
                &escape,
            ) {
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
