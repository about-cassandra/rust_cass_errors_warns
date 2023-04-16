use regex::Regex;
use std::fs;
use std::io::BufRead;
use std::io::Write;
use std::path::Path;

fn main() -> Result<(), std::io::Error> {
    let search_dir = ".";
    let search_path = Path::new(search_dir);

    if !search_path.is_dir() {
        eprintln!("Error: {} is not a directory", search_dir);
        return Ok(());
    }

    let output_file = "/tmp/errors-warns.log";
    let output_path = Path::new(output_file);
    if output_path.exists() {
        eprintln!("Error: {} already exists", output_file);
        return Ok(());
    }

    let mut entries: Vec<String> = Vec::new();
    search_log_files(search_path, &mut entries)?;
    // entries.sort();

    entries.sort_by(|a, b| {
        let a_columns: Vec<&str> = a.split_whitespace().collect();
        let b_columns: Vec<&str> = b.split_whitespace().collect();
        let date_col = a_columns[3].cmp(b_columns[3]);
        let time_col = a_columns[4].cmp(b_columns[4]);
        date_col.then(time_col)
    });

    let mut output = fs::File::create(output_path)?;
    for entry in entries {
        output.write_all(format!("{}\n", entry).as_bytes())?;
    }

    Ok(())
}

fn search_log_files(path: &Path, entries: &mut Vec<String>) -> Result<(), std::io::Error> {
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                search_log_files(&path, entries)?;
            } else {
                if path.file_name().unwrap() == "system.log" {
                    process_log_file(&path, entries)?;
                }
            }
        }
    }
    Ok(())
}

fn process_log_file(path: &Path, entries: &mut Vec<String>) -> Result<(), std::io::Error> {
    let re = Regex::new(r"(^ERROR|^WARN)").unwrap();
    let file = fs::File::open(path)?;
    let node_ip = match extract_ip_address(&path.display().to_string()) {
        Some(ip) => ip,
        None => path.display().to_string(),
    };
    let reader = std::io::BufReader::new(file);
    for line in reader.lines() {
        let line = line?;
        if re.is_match(&line) {
            let mut cols: Vec<&str> = line.split_whitespace().collect();
            if !cols[1].contains('[') {
                cols.insert(1, "[No_Thread]");
            }
            let line_with_thread = cols.join(" ");
            let cleaned_line = replace_spaces_in_brackets(&line_with_thread);
            let final_line = format!("{} {}", node_ip, cleaned_line);
            entries.push(final_line);
        }
    }
    Ok(())
}


fn replace_spaces_in_brackets(s: &str) -> String {
    let re = Regex::new(r"\[([^\]]*)\]").unwrap();
    re.replace_all(s, |caps: &regex::Captures| {
        let mut bracket_content = caps[1].to_string();
        bracket_content = bracket_content.replace(" ", "_");
        format!("[{}]", bracket_content)
    })
    .to_string()
}

fn extract_ip_address(text: &str) -> Option<String> {
    let re = Regex::new(r"(\d{1,3}.\d{1,3}.\d{1,3}.\d{1,3})").unwrap();
    let captures = re.captures(text)?;
    let ip = captures.get(1)?.as_str();
    Some(ip.to_string())
}
