use std::collections::HashMap;

use crate:: {
    audio::Audio,
    interpreter:: {
        parse_line,
        Command,
        PrintPart,
        RuntimeError,
        RuntimeErrorKind,
    },
    runtime:: {
        Runtime,
        Value
    }
};

//////////////////////////////////
/// STUCTS and ENUMS
//////////////////////////////////
#[derive(Debug, Clone)]
pub enum VmEffect {
    PlayTone { hz: u32, ms: u64 },
    BeginInput { prompt: Option<String>, var: String },
}

#[derive(Debug, Clone)]
pub struct ProgramLine {
    pub basic_line: Option<u32>,
    pub stmt: String,
    pub raw: String,
    pub physical_line: usize,
}

#[derive(Debug)]
pub struct Program {
    pub lines: Vec<ProgramLine>,
    pub line_index: HashMap<u32, usize>,
    pub pc: usize,
    pub halted: bool,
}

//////////////////////////////////
/// PROGRAM Implementation
//////////////////////////////////
impl Program {
    pub fn from_source(source: &str) -> Result<Self, RuntimeError> {
        let mut lines = Vec::new();
        let mut line_index = HashMap::new();

        for (i, raw_line) in source.lines().enumerate() {
            let physical_line = i + 1;
            let raw = raw_line.to_string();
            let trimmed = raw_line.trim();

            if trimmed.is_empty() {
                lines.push(ProgramLine {
                    basic_line: None,
                    physical_line,
                    raw,
                    stmt: String::new(),
                });
                continue;
            }

            // Optional BASIC line number prefix
            let (basic_line, stmt) = parse_optional_line_number(trimmed);

            if let Some(n) = basic_line {
                if line_index.contains_key(&n) {
                    return Err(RuntimeError {
                        kind: RuntimeErrorKind::Syntax,
                        message: format!("Duplicate BASIC line number {}", n),
                    });
                }
                line_index.insert(n, lines.len());
            }

            lines.push(ProgramLine {
                basic_line,
                physical_line,
                raw,
                stmt: stmt.to_string(),
            });
        }

        Ok(Self {
            lines,
            line_index,
            pc: 0,
            halted: false,
        })
    }

    pub fn step(&mut self, rt: &mut Runtime) -> Result<Option<VmEffect>, RuntimeError> {
        if self.halted {
            return Ok(None);
        }
        if self.pc >= self.lines.len() {
            self.halted = true;
            return Ok(None);
        }

        let cur = &self.lines[self.pc];
        let label = line_label(cur.basic_line, cur.physical_line);

        let cmd = parse_line(&cur.stmt).map_err(|e| RuntimeError {
            kind: e.kind,
            message: format!("{label}: {} | source: {}", e.message, cur.raw.trim()),
        })?;

        let cmd = parse_line(&cur.stmt).map_err(|e| RuntimeError {
            kind: RuntimeErrorKind::Syntax,
            message: format!("{label}: {} | source: {}", e, cur.raw.trim()),
        })?;

        match cmd {
            Command::Empty => {
                self.pc += 1;
                Ok(None)
            },
            Command::LetNum { name, expr } => {
                let v = expr.eval(rt)?;
                rt.vars.insert(name, crate::runtime::Value::Num(v));
                self.pc += 1;
                Ok(None)
            }
            Command::LetStr { name, value } => {
                rt.vars.insert(name, crate::runtime::Value::Str(value));
                self.pc += 1;
                Ok(None)
            }
            Command::Print(parts) => {
                let mut out = String::new();
                for p in parts {
                    match p {
                        PrintPart::Text(t) => out.push_str(&t),
                        PrintPart::Expr(expr) => {
                            let v = expr.eval(rt)?;
                            if v.fract() == 0.0 { out.push_str(&(v as i64).to_string()); }
                            else { out.push_str(&v.to_string()); }
                        }
                        PrintPart::StrVar(name) => {
                            match rt.vars.get(&name) {
                                Some(crate::runtime::Value::Str(s)) => out.push_str(s),
                                Some(crate::runtime::Value::Num(_)) => {
                                    return Err(RuntimeError::type_mismatch(
                                        format!("Type Mismatch: numeric variable {} used as string", name)
                                    ));
                                }
                                None => return Err(RuntimeError::syntax(format!("Undefined variable {}", name))),
                            }
                        }
                    }
                }
                rt.print(out);
                self.pc += 1;
                Ok(None)
            }                                
            Command::Cls(color) => {
                rt.cls(color);
                self.pc += 1;
                Ok(None)
            }
            Command::Goto(target) => {
                let Some(&dest) = self.line_index.get(&target) else {
                    return Err(RuntimeError {
                        kind: RuntimeErrorKind::Syntax,
                        message: format!("{label}: GOTO target {} not found", target),
                    });
                };
                self.pc = dest;
                Ok(None)
            }
            Command::Sound { tone, len } => {
                let hz = crate::audio::Audio::sound_to_hz(tone);
                let ms = crate::audio::Audio::sound_len_to_ms(len);
                self.pc += 1;
                Ok(Some(VmEffect::PlayTone { hz, ms }))
            }
            Command::Input { prompt, var } => {
                self.pc += 1;
                Ok(Some(VmEffect::BeginInput { prompt, var }))
            }
        }
    }
}

//////////////////////////////////
/// FUNCTIONS
//////////////////////////////////
fn with_line_context(
    e: RuntimeError,
    label: &str,
    raw: &str,
) -> RuntimeError {
    RuntimeError {
        kind: e.kind,
        message: format!("{label}: {} | source: {}", e.message, raw.trim()),
    }
}

fn parse_optional_line_number(s: &str) -> (Option<u32>, &str) {
    let mut split = s.splitn(2, char::is_whitespace);
    let first = split.next().unwrap_or("");
    let rest = split.next().unwrap_or("").trim_start();

    if let Ok(n) = first.parse::<u32>() {
        (Some(n), rest)
    } else {
        (None, s)
    }
}

fn line_label(basic_line: Option<u32>, physical_line: usize) -> String {
    match basic_line {
        Some(n) => format!("BASIC line {}", n),
        None => format!("file line {}", physical_line),
    }
}