use crate::runtime::ClsColor;

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Print(String),
    Cls(Option<ClsColor>), // None = no arg
    Goto(u32),
    Empty,
}

pub fn parse_line(input: &str) -> Result<Command, String> {
    let mut s = input.trim();
    if s.is_empty() {
        return Ok(Command::Empty);
    }

    let upper = s.to_ascii_uppercase();

    if upper == "CLS" {
        eprintln!(">> COMMAND: CLS");
        return Ok(Command::Cls(None));
    }

    if upper.len() == 4 && upper.starts_with("CLS") {
        let digit = upper.as_bytes()[3];
        if digit.is_ascii_digit() {
            let n = (digit - b'0') as u8;
            let color = ClsColor::from_u8(n)
                .ok_or_else(|| format!("CLS argument out of range 0..8, got '{n}'"))?;
            eprintln!(">> COMMAND: CLS{n}");
            return Ok(Command::Cls(Some(color)));
        }
    }

    if upper.starts_with("CLS ") {
        let arg = s[3..].trim(); // text after CLS
        let n: u8 = arg
            .parse()
            .map_err(|_| format!("CLS argument must be integer 0..8, got '{arg}'"))?;
        let color = ClsColor::from_u8(n)
            .ok_or_else(|| format!("CLS argument out of range 0..8, got '{n}'"))?;
        eprintln!(">> COMMAND: CLS {arg} [{n}]");
        return Ok(Command::Cls(Some(color)));
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

    if upper.starts_with("GOTO") {
        let arg = s[4..].trim_start();
        if arg.is_empty() {
            return Err("GOTO requires a target line number".to_string());
        }
        let target: u32 = arg
            .parse()
            .map_err(|_| format!("GOTO target must be a line number, got '{arg}'"))?;
        return Ok(Command::Goto(target));
    }

    Err(format!("Unsupported statement: {s}"))
}

