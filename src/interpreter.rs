use crate::runtime::ClsColor;

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Print(String),
    Cls(Option<ClsColor>), // None = no arg
    Goto(u32),
    Sound { tone: u8, len: u8 },
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

    if upper.starts_with("SOUND") {
        let arg_text = s[5..].trim_start();
        if arg_text.is_empty() {
            return Err("SOUND requires two arguments: SOUND tone,length".to_string());
        }

        let mut parts = arg_text.split(',').map(|p| p.trim());
        let tone_s = parts.next().unwrap_or_default();
        let len_s = parts.next().unwrap_or_default();

        // reject missing or extra args
        if tone_s.is_empty() || len_s.is_empty() || parts.next().is_some() {
            return Err(format!(
                "SOUND format is SOUND tone,length with both values 1..255, got '{arg_text}'"
            ));
        }

        let tone_u16: u16 = tone_s
            .parse()
            .map_err(|_| format!("SOUND tone must be integer 1..255, got '{tone_s}'"))?;
        let len_u16: u16 = len_s
            .parse()
            .map_err(|_| format!("SOUND length must be integer 1..255, got '{len_s}'"))?;

        if !(1..=255).contains(&tone_u16) {
            return Err(format!("SOUND tone out of range 1..255, got {}", tone_u16));
        }
        if !(1..=255).contains(&len_u16) {
            return Err(format!("SOUND length out of range 1..255, got {}", len_u16));
        }

        return Ok(Command::Sound {
            tone: tone_u16 as u8,
            len: len_u16 as u8,
        });
    }

    Err(format!("Unsupported statement: {s}"))
}

