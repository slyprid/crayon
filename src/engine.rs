use std::collections::HashMap;

use crate::interpreter::{parse_line, Command};
use crate::runtime::Runtime;
use crate::audio;

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

#[derive(Debug, Clone, Copy)]
pub enum VmEffect {
    PlayTone { hz: u32, ms: u64 },
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

fn line_label(basic_line: Option<u32>, physical_line: usize) -> String {
    match basic_line {
        Some(n) => format!("BASIC line {}", n),
        None => format!("file line {}", physical_line),
    }
}

impl Program {
    pub fn from_source(source: &str) -> Result<Self, String> {
        let mut lines = Vec::new();
        let mut line_index = HashMap::new();

        for (idx, raw) in source.lines().enumerate() {
            let physical_line = idx + 1;
            let (basic_line, stmt) = split_basic_line(raw);

            if let Some(n) = basic_line {
                if line_index.insert(n, lines.len()).is_some() {
                    return Err(format!("Duplicate BASIC line number {}", n));
                }
            }

            lines.push(ProgramLine {
                basic_line,
                stmt: stmt.to_string(),
                raw: raw.to_string(),
                physical_line,
            });
        }

        Ok(Self {
            lines,
            line_index,
            pc: 0,
            halted: false,
        })
    }

    pub fn step(&mut self, rt: &mut Runtime) -> Result<Option<VmEffect>, String> {
        if self.halted {
            return Ok(None);
        }
        if self.pc >= self.lines.len() {
            self.halted = true;
            return Ok(None);
        }

        let cur = &self.lines[self.pc];
        let label = line_label(cur.basic_line, cur.physical_line);

        let cmd = parse_line(&cur.stmt)
            .map_err(|e| format!("{label}: {e} | source: {}", cur.raw.trim()))?;

        match cmd {
            Command::Empty => {
                self.pc += 1;
                Ok(None)
            },
            Command::Print(s) => {
                rt.print(s);
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
                    return Err(format!("{label}: GOTO target {} not found", target));
                };
                self.pc = dest;
                Ok(None)
            },
            Command::Sound { tone, len } => {
                let hz = crate::audio::Audio::sound_to_hz(tone);
                let ms = crate::audio::Audio::sound_len_to_ms(len);
                self.pc += 1;
                Ok(Some(VmEffect::PlayTone { hz, ms }))
            }
        }
    }
}