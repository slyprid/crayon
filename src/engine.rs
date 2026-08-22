use std::collections::HashMap;

use crate:: {
    audio::Audio,
    interpreter:: {
        parse_line,
        CmpOp,
        Command,
        CondExpr,
        LogicOp,
        Predicate,
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
/// STRUCTS and ENUMS
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
    pub last_basic_line: Option<u32>,
    for_stack: Vec<ForFrame>,
}

#[derive(Debug, Clone)]
struct ForFrame {
    var: String,
    end: f64,
    step: f64,
    for_pc: usize,
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
            last_basic_line: None,
            for_stack: Vec::new(),
        })
    }

    pub fn step(&mut self, rt: &mut Runtime) -> Result<Option<VmEffect>, RuntimeError> {
        if self.halted { return Ok(None); }
        if self.pc >= self.lines.len() {
            self.halted = true;
            return Ok(None);
        }

        let cur = self.lines[self.pc].clone();
        self.last_basic_line = cur.basic_line;
        let label = line_label(cur.basic_line, cur.physical_line);

        let cmd = parse_line(&cur.stmt).map_err(|e| RuntimeError {
            kind: e.kind,
            message: format!("{label}: {} | source: {}", e.message, cur.raw.trim()),
        })?;

        self.execute_inline_command(rt, &cmd, &cur, &label)
    }

    fn execute_inline_command(
        &mut self,
        rt: &mut Runtime,
        cmd: &Command,
        cur: &ProgramLine,
        label: &str,
    ) -> Result<Option<VmEffect>, RuntimeError> {
        match cmd {
            Command::Empty => {
                self.pc += 1;
                Ok(None)
            }

            Command::Print(parts) => {
                let mut out = String::new();
                for p in parts {
                    match p {
                        PrintPart::Text(t) => out.push_str(t),
                        PrintPart::Expr(expr) => {
                            let v = expr.eval(rt).map_err(|e| RuntimeError {
                                kind: e.kind,
                                message: format!("{label}: {} | source: {}", e.message, cur.raw.trim()),
                            })?;
                            if v.fract() == 0.0 { out.push_str(&(v as i64).to_string()); }
                            else { out.push_str(&v.to_string()); }
                        }
                        PrintPart::StrVar(name) => match rt.vars.get(name) {
                            Some(Value::Str(s)) => out.push_str(s),
                            Some(Value::Num(_)) => {
                                return Err(RuntimeError::type_mismatch(format!(
                                    "{label}: Type Mismatch: numeric variable {} used as string | source: {}",
                                    name, cur.raw.trim()
                                )));
                            }
                            None => {
                                return Err(RuntimeError::syntax(format!(
                                    "{label}: Undefined variable {} | source: {}",
                                    name, cur.raw.trim()
                                )));
                            }
                        },
                    }
                }
                rt.print(out);
                self.pc += 1;
                Ok(None)
            }

            Command::LetNum { name, expr } => {
                let v = expr.eval(rt).map_err(|e| RuntimeError {
                    kind: e.kind,
                    message: format!("{label}: {} | source: {}", e.message, cur.raw.trim()),
                })?;
                rt.vars.insert(name.clone(), Value::Num(v));
                self.pc += 1;
                Ok(None)
            }

            Command::LetStr { name, value } => {
                rt.vars.insert(name.clone(), Value::Str(value.clone()));
                self.pc += 1;
                Ok(None)
            }

            Command::For { name, start, end, step } => {
                let start_value = start.eval(rt).map_err(|e| RuntimeError {
                    kind: e.kind,
                    message: format!("{label}: {} | source: {}", e.message, cur.raw.trim()),
                })?;
                let end_value = end.eval(rt).map_err(|e| RuntimeError {
                    kind: e.kind,
                    message: format!("{label}: {} | source: {}", e.message, cur.raw.trim()),
                })?;
                let step_v = step.eval(rt).map_err(|e| RuntimeError {
                    kind: e.kind,
                    message: format!("{label}: {} | source: {}", e.message, cur.raw.trim()),
                })?;

                if step_v == 0.0 {
                    return Err(RuntimeError::syntax(format!(
                        "{label}: FOR STEP cannot be 0 | source: {}",
                        cur.raw.trim()
                    )));
                }

                rt.vars.insert(name.clone(), Value::Num(start_value));

                if start_value > end_value {
                    let next_pc = self.find_matching_next(self.pc).map_err(|e| RuntimeError {
                        kind: e.kind,
                        message: format!("{label}: {} | source: {}", e.message, cur.raw.trim()),
                    })?;
                    self.pc = next_pc + 1;
                } else {
                    self.for_stack.push(ForFrame {
                        var: name.clone(),
                        end: end_value,
                        step: step_v,
                        for_pc: self.pc,
                    });
                    self.pc += 1;
                }
                Ok(None)
            }

            Command::Next { name } => {
                let Some(frame) = self.for_stack.last().cloned() else {
                    return Err(RuntimeError::syntax(format!(
                        "{label}: NEXT {} without FOR | source: {}",
                        name, cur.raw.trim()
                    )));
                };

                if frame.var != *name {
                    return Err(RuntimeError::syntax(format!(
                        "{label}: NEXT {} does not match FOR {} | source: {}",
                        name, frame.var, cur.raw.trim()
                    )));
                }

                let current = match rt.vars.get(name) {
                    Some(Value::Num(v)) => *v,
                    Some(Value::Str(_)) => {
                        return Err(RuntimeError::type_mismatch(format!(
                            "{label}: Type Mismatch: string variable {} used as FOR counter | source: {}",
                            name, cur.raw.trim()
                        )));
                    }
                    None => {
                        return Err(RuntimeError::syntax(format!(
                            "{label}: Undefined FOR counter {} | source: {}",
                            name, cur.raw.trim()
                        )));
                    }
                };

                let next_value = current + frame.step;
                rt.vars.insert(name.clone(), Value::Num(next_value));

                let keep_going = if frame.step > 0.0 {
                    next_value <= frame.end
                } else {
                    next_value >= frame.end
                };

                if keep_going {
                    self.pc = frame.for_pc + 1;
                } else {
                    self.for_stack.pop();
                    self.pc += 1;
                }
                Ok(None)
            }

            Command::Input { prompt, var } => {
                self.pc += 1;
                Ok(Some(VmEffect::BeginInput { prompt: prompt.clone(), var: var.clone() }))
            }

            Command::IfThenElse { cond, then_cmd, else_cmd } => {
                let ok = eval_cond_expr(rt, cond)?;
                if ok {
                    self.execute_inline_command(rt, then_cmd, cur, label)
                } else if let Some(ec) = else_cmd {
                    self.execute_inline_command(rt, ec, cur, label)
                } else {
                    self.pc += 1;
                    Ok(None)
                }
            }

            Command::Cls(color) => {
                rt.cls(*color);
                self.pc += 1;
                Ok(None)
            }

            Command::Goto(target) => {
                let Some(&dest) = self.line_index.get(target) else {
                    return Err(RuntimeError::syntax(format!(
                        "{label}: GOTO target {} not found | source: {}",
                        target, cur.raw.trim()
                    )));
                };
                self.pc = dest;
                Ok(None)
            }

            Command::Sound { tone, len } => {
                let tone_v = tone.eval(rt).map_err(|e| RuntimeError {
                    kind: e.kind,
                    message: format!("{label}: {} | source: {}", e.message, cur.raw.trim()),
                })?;
                let len_v = len.eval(rt).map_err(|e| RuntimeError {
                    kind: e.kind,
                    message: format!("{label}: {} | source: {}", e.message, cur.raw.trim()),
                })?;

                // BASIC-like integer coercion; feel free to switch to strict integer-only checks
                let tone_i = tone_v.round() as i64;
                let len_i = len_v.round() as i64;

                if !(1..=255).contains(&tone_i) {
                    return Err(RuntimeError::syntax(format!(
                        "{label}: SOUND tone out of range 1..255, got {} | source: {}",
                        tone_i, cur.raw.trim()
                    )));
                }
                if !(1..=255).contains(&len_i) {
                    return Err(RuntimeError::syntax(format!(
                        "{label}: SOUND length out of range 1..255, got {} | source: {}",
                        len_i, cur.raw.trim()
                    )));
                }

                let hz = Audio::sound_to_hz(tone_i as u8);
                let ms = Audio::sound_len_to_ms(len_i as u8);
                self.pc += 1;
                Ok(Some(VmEffect::PlayTone { hz, ms }))
            }

            Command::End => {
                self.halted = true;
                Ok(None)
            }
        }
    }

    fn find_matching_next(&self, for_pc: usize) -> Result<usize, RuntimeError> {
        let mut depth = 0usize;

        for i in for_pc + 1..self.lines.len() {
            let line = &self.lines[i];
            match parse_line(&line.stmt)? {
                Command::For { .. } => depth += 1,
                Command::Next { .. } => {
                    if depth == 0 {
                        return Ok(i);
                    }
                    depth -= 1;
                }
                _ => {}
            }
        }

        Err(RuntimeError::syntax("FOR without matching NEXT"))
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

fn eval_cond_expr(rt: &Runtime, c: &CondExpr) -> Result<bool, RuntimeError> {
    let mut acc = eval_pred(rt, &c.first)?;
    for (op, pred) in &c.rest {
        let rhs = eval_pred(rt, pred)?;
        acc = match op {
            LogicOp::And => acc && rhs,
            LogicOp::Or => acc || rhs,
        };
    }
    Ok(acc)
}

fn eval_pred(rt: &Runtime, p: &Predicate) -> Result<bool, RuntimeError> {
    match p {
        Predicate::NumCmp { left, op, right } => {
            let a = left.eval(rt)?;
            let b = right.eval(rt)?;
            Ok(match op {
                CmpOp::Eq => a == b,
                CmpOp::Ne => a != b,
                CmpOp::Lt => a < b,
                CmpOp::Le => a <= b,
                CmpOp::Gt => a > b,
                CmpOp::Ge => a >= b,
            })
        }
        Predicate::StrCmp { left_var, op, right_lit } => {
            let lv = rt.vars.get(left_var).ok_or_else(|| RuntimeError::syntax(format!(
                "Undefined variable {}", left_var
            )))?;
            let s = match lv {
                Value::Str(v) => v.as_str(),
                Value::Num(_) => {
                    return Err(RuntimeError::type_mismatch(format!(
                        "Type Mismatch: numeric variable {} used in string comparison", left_var
                    )));
                }
            };
            Ok(match op {
                CmpOp::Eq => s == right_lit,
                CmpOp::Ne => s != right_lit,
                CmpOp::Lt => s < right_lit.as_str(),
                CmpOp::Le => s <= right_lit.as_str(),
                CmpOp::Gt => s > right_lit.as_str(),
                CmpOp::Ge => s >= right_lit.as_str(),
            })
        }
    }
}
