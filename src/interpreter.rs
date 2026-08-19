#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Print(String),
    Cls,
    Empty,
}

pub fn parse_line(input: &str) -> Result<Command, String> {
    let mut s = input.trim();
    if s.is_empty() {
        return Ok(Command::Empty);
    }

    // Optional BASIC line number: "10 PRINT "HI""
    if let Some((first, rest)) = s.split_once(' ') {
        if first.chars().all(|c| c.is_ascii_digit()) {
            s = rest.trim_start();
        }
    }

    let upper = s.to_ascii_uppercase();

    if upper == "CLS" {
        eprintln!(">> COMMAND: CLS");
        return Ok(Command::Cls);
    }

    if upper.starts_with("PRINT") {
        eprintln!(">> COMMAND: PRINT");
        let rest = s[5..].trim_start(); // after PRINT
        if rest.is_empty() {
            return Ok(Command::Print(String::new()));
        }

        // Minimal v1:
        // PRINT "HELLO"
        // PRINT HELLO   (prints literal token as-is for now)
        let text = if rest.starts_with('"') && rest.ends_with('"') && rest.len() >= 2 {
            rest[1..rest.len() - 1].to_string()
        } else {
            rest.to_string()
        };

        return Ok(Command::Print(text));
    }

    Err(format!("Unsupported statement: {s}"))
}