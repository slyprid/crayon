use crate::interpreter::{parse_line, Command};
use crate::runtime::Runtime;

pub fn run_program(source: &str, rt: &mut Runtime) -> Result<(), String> {
    for (idx, raw) in source.lines().enumerate() {
        let physical_line = idx + 1;
        let (basic_line, stmt) = split_basic_line(raw);

        let label = if let Some(n) = basic_line {
            format!("BASIC line {}", n)
        } else {
            format!("file line {}", physical_line)
        };

        let cmd = parse_line(stmt)
            .map_err(|e| format!("{label}: {e} | source: {}", raw.trim()))?;

        match cmd {
            Command::Empty => {}
            Command::Print(s) => rt.print(s),
            Command::Cls(color) => rt.cls(color),
        }
    }
    Ok(())
}

fn split_basic_line(raw: &str) -> (Option<u32>, &str) {
    let s = raw.trim_start();
    if s.is_empty() {
        return (None, s);
    }

    if let Some((first, rest)) = s.split_once(char::is_whitespace) {
        if first.chars().all(|c| c.is_ascii_digit()) {
            if let Ok(n) = first.parse::<u32>() {
                return (Some(n), rest.trim_start());
            }
        }
    }

    (None, s)
}