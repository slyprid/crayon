use crate::runtime::ClsColor;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeErrorKind {
    Syntax,
    DivideByZero,
    TypeMismatch,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeError {
    pub kind: RuntimeErrorKind,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Op { Add, Sub, Mul, Div }

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Num(f64),
    Var(String),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum PrintPart {
    Text(String),
    Expr(Expr),
    StrVar(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Print(Vec<PrintPart>),
    LetNum { name: String, expr: Expr },
    LetStr { name: String, value: String },
    Cls(Option<ClsColor>), // None = no arg
    Goto(u32),
    Sound { tone: u8, len: u8 },
    Empty,
}

pub fn parse_line(input: &str) -> Result<Command, RuntimeError> {
    let mut s = input.trim();
    if s.is_empty() {
        return Ok(Command::Empty);
    }

    let upper = s.to_ascii_uppercase();

    // Explicit LET: LET A = 10 / LET X$ = "HELLO"
    if upper.starts_with("LET ") {
        let rest = s[3..].trim_start(); // everything after LET
        return parse_assignment_core(rest);
    }

    if upper == "CLS" {
        eprintln!(">> COMMAND: CLS");
        return Ok(Command::Cls(None));
    }

    if upper.len() == 4 && upper.starts_with("CLS") {
        let digit = upper.as_bytes()[3];
        if digit.is_ascii_digit() {
            let n = (digit - b'0') as u8;
            let color = ClsColor::from_u8(n)
                .ok_or_else(|| RuntimeError::syntax(format!("CLS argument out of range 0..8, got '{n}'")))?;
            eprintln!(">> COMMAND: CLS{n}");
            return Ok(Command::Cls(Some(color)));
        }
    }

    if upper.starts_with("CLS ") {
        let arg = s[3..].trim(); // text after CLS
        let n: u8 = arg
            .parse()
            .map_err(|_| RuntimeError::syntax(format!("CLS argument must be integer 0..8, got '{arg}'")))?;
        let color = ClsColor::from_u8(n)
            .ok_or_else(|| RuntimeError::syntax(format!("CLS argument out of range 0..8, got '{n}'")))?;
        eprintln!(">> COMMAND: CLS {arg} [{n}]");
        return Ok(Command::Cls(Some(color)));
    }

    if upper.starts_with("PRINT") {
        let rest = s[5..].trim_start();
        if rest.is_empty() {
            return Ok(Command::Print(vec![PrintPart::Text(String::new())]));
        }
        let parts = parse_print_parts(rest)?;
        return Ok(Command::Print(parts));
    }

    if upper.starts_with("GOTO") {
        let arg = s[4..].trim_start();
        if arg.is_empty() {
            return Err(RuntimeError::syntax("GOTO requires a target line number"));
        }
        let target: u32 = arg
            .parse()
            .map_err(|_| RuntimeError::syntax(format!("GOTO target must be a line number, got '{arg}'")))?;
        return Ok(Command::Goto(target));
    }

    if upper.starts_with("SOUND") {
        let arg_text = s[5..].trim_start();
        if arg_text.is_empty() {
            return Err(RuntimeError::syntax("SOUND requires two arguments: SOUND tone,length"));
        }

        let mut parts = arg_text.split(',').map(|p| p.trim());
        let tone_s = parts.next().unwrap_or_default();
        let len_s = parts.next().unwrap_or_default();

        // reject missing or extra args
        if tone_s.is_empty() || len_s.is_empty() || parts.next().is_some() {
            return Err(RuntimeError::syntax("SOUND format is SOUND tone,length with both values 1..255, got '{arg_text}'"));
        }

        let tone_u16: u16 = tone_s
            .parse()
            .map_err(|_| RuntimeError::syntax(format!("SOUND tone must be integer 1..255, got '{tone_s}'")))?;
        let len_u16: u16 = len_s
            .parse()
            .map_err(|_| RuntimeError::syntax(format!("SOUND length must be integer 1..255, got '{len_s}'")))?;

        if !(1..=255).contains(&tone_u16) {
            return Err(RuntimeError::syntax(format!("SOUND tone out of range 1..255, got {}", tone_u16)));
        }
        if !(1..=255).contains(&len_u16) {
            return Err(RuntimeError::syntax(format!("SOUND length out of range 1..255, got {}", len_u16)));
        }

        return Ok(Command::Sound {
            tone: tone_u16 as u8,
            len: len_u16 as u8,
        });
    }

    // Implicit assignment (no LET): A = 10 / X$ = "HELLO"
    // Kept late so PRINT/GOTO/etc. are not mistaken for assignment statements.
    if let Some((lhs_raw, _rhs_raw)) = s.split_once('=') {
        let lhs = lhs_raw.trim().to_ascii_uppercase();
        if is_valid_num_var_name(&lhs) || is_valid_str_var_name(&lhs) {
            return parse_assignment_core(s);
        }
    }

    Err(RuntimeError::syntax(("?SN ERROR: {s}")))
}

fn parse_print_parts(mut s: &str) -> Result<Vec<PrintPart>, RuntimeError> {
    let mut parts = Vec::new();

    while !s.trim_start().is_empty() {
        s = s.trim_start();

        // 1) quoted string literal
        if s.starts_with('"') {
            let rest = &s[1..];
            let Some(end) = rest.find('"') else {
                return Err(RuntimeError::syntax("Unterminated string in PRINT"));
            };
            parts.push(PrintPart::Text(rest[..end].to_string()));
            s = &rest[end + 1..];
            continue;
        }

        // 2) string variable token like X$, AB$, A1$
        // token = up to whitespace or quote
        let token_end = s
            .find(|ch: char| ch.is_whitespace() || ch == '"')
            .unwrap_or(s.len());
        let token = &s[..token_end];
        let token_up = token.to_ascii_uppercase();

        if is_valid_str_var_name(&token_up) {
            parts.push(PrintPart::StrVar(token_up));
            s = &s[token_end..];
            continue;
        }

        // 3) otherwise parse numeric expression segment until next quote
        let end = s.find('"').unwrap_or(s.len());
        let expr_src = s[..end].trim();
        if !expr_src.is_empty() {
            let expr = parse_expr(&expr_src.to_ascii_uppercase())?;
            parts.push(PrintPart::Expr(expr));
        }
        s = &s[end..];
    }

    Ok(parts)
}

fn precedence(op: Op) -> u8 {
    match op {
        Op::Add | Op::Sub => 1,
        Op::Mul | Op::Div => 2,
    }
}

fn parse_expr(input: &str) -> Result<Expr, RuntimeError> {
    // tokenizer: numbers + operators + spaces
    let mut vals: Vec<Expr> = Vec::new();
    let mut ops: Vec<Op> = Vec::new();

    let chars: Vec<char> = input.chars().collect();
    let mut i = 0usize;

    fn apply_top(vals: &mut Vec<Expr>, ops: &mut Vec<Op>) -> Result<(), RuntimeError> {
        let op = ops
            .pop()
            .ok_or_else(|| RuntimeError::syntax("operator stack underflow"))?;
        let b = vals
            .pop()
            .ok_or_else(|| RuntimeError::syntax("missing right operand"))?;
        let a = vals
            .pop()
            .ok_or_else(|| RuntimeError::syntax("missing left operand"))?;
        let node = match op {
            Op::Add => Expr::Add(Box::new(a), Box::new(b)),
            Op::Sub => Expr::Sub(Box::new(a), Box::new(b)),
            Op::Mul => Expr::Mul(Box::new(a), Box::new(b)),
            Op::Div => Expr::Div(Box::new(a), Box::new(b)),
        };
        vals.push(node);
        Ok(())
    }

    while i < chars.len() {
        let c = chars[i];

        if c.is_whitespace() {
            i += 1;
            continue;
        }

        if c.is_ascii_digit() || c == '.' {
            let start = i;
            i += 1;
            while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                i += 1;
            }
            let num_str: String = chars[start..i].iter().collect();
            let n: f64 = num_str
                .parse()
                .map_err(|_| RuntimeError::syntax(format!("Invalid number '{num_str}'")))?;
            vals.push(Expr::Num(n));
            continue;
        }

        // identifier: A .. Z, optional second char [A-Z0-9]
        if c.is_ascii_uppercase() {
            let start = i;
            i += 1;

            if i < chars.len() && (chars[i].is_ascii_uppercase() || chars[i].is_ascii_digit()) {
                i += 1;
            }

            // third alnum char is invalid for your 1-2 char rule
            if i < chars.len() && (chars[i].is_ascii_uppercase() || chars[i].is_ascii_digit()) {
                let bad: String = chars[start..=i].iter().collect();
                return Err(RuntimeError::syntax(format!(
                    "Invalid variable name '{}': max 2 chars",
                    bad
                )));
            }

            let name: String = chars[start..i].iter().collect();
            vals.push(Expr::Var(name));
            continue;
        }

        if c == '$' {
            return Err(RuntimeError::type_mismatch(
                "Type Mismatch: string variable used in numeric expression",
            ));
        }

        let op = match c {
            '+' => Op::Add,
            '-' => Op::Sub,
            '*' => Op::Mul,
            '/' => Op::Div,
            _ => return Err(RuntimeError::syntax(format!("Unexpected token '{}' in expression", c))),
        };

        while let Some(top) = ops.last().copied() {
            if precedence(top) >= precedence(op) {
                apply_top(&mut vals, &mut ops)?;
            } else {
                break;
            }
        }

        ops.push(op);
        i += 1;
    }

    while !ops.is_empty() {
        apply_top(&mut vals, &mut ops)?;
    }

    if vals.len() != 1 {
        return Err(RuntimeError::syntax(format!("Invalid expression '{}'", input)));
    }

    Ok(vals.pop().unwrap())
}

impl RuntimeError {
    pub fn syntax(msg: impl Into<String>) -> Self {
        Self { kind: RuntimeErrorKind::Syntax, message: msg.into() }
    }
    pub fn divide_by_zero(msg: impl Into<String>) -> Self {
        Self { kind: RuntimeErrorKind::DivideByZero, message: msg.into() }
    }

    pub fn type_mismatch(msg: impl Into<String>) -> Self {
        Self { kind: RuntimeErrorKind::TypeMismatch, message: msg.into() }
    }
}

impl Expr {
    pub fn eval(&self, rt: &crate::runtime::Runtime) -> Result<f64, RuntimeError> {
        match self {
            Expr::Num(n) => Ok(*n),
            Expr::Var(name) => {
                match rt.vars.get(name) {
                    Some(crate::runtime::Value::Num(v)) => Ok(*v),
                    Some(crate::runtime::Value::Str(_)) => Err(RuntimeError::type_mismatch(
                        format!("Type Mismatch: string variable {} used in numeric expression", name)
                    )),
                    None => Err(RuntimeError::syntax(format!("Undefined variable {}", name))),
                }
            }
            Expr::Add(a,b) => Ok(a.eval(rt)? + b.eval(rt)?),
            Expr::Sub(a,b) => Ok(a.eval(rt)? - b.eval(rt)?),
            Expr::Mul(a,b) => Ok(a.eval(rt)? * b.eval(rt)?),
            Expr::Div(a,b) => {
                let d = b.eval(rt)?;
                if d == 0.0 {
                    return Err(RuntimeError::divide_by_zero("Division by zero"));
                }
                Ok(a.eval(rt)? / d)
            }
        }
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for RuntimeError {}

fn is_valid_num_var_name(name: &str) -> bool {
    let b = name.as_bytes();
    if b.is_empty() || b.len() > 2 {
        return false;
    }
    if !b[0].is_ascii_uppercase() {
        return false;
    }
    if b.len() == 2 && !(b[1].is_ascii_uppercase() || b[1].is_ascii_digit()) {
        return false;
    }
    true
}

fn is_valid_str_var_name(name: &str) -> bool {
    // X$, AB$, A1$
    if !name.ends_with('$') {
        return false;
    }
    let core = &name[..name.len() - 1];
    is_valid_num_var_name(core)
}

fn parse_assignment_core(text: &str) -> Result<Command, RuntimeError> {
    let Some((lhs_raw, rhs_raw)) = text.split_once('=') else {
        return Err(RuntimeError::syntax("Assignment requires '='"));
    };

    let lhs = lhs_raw.trim().to_ascii_uppercase();
    let rhs = rhs_raw.trim();

    if lhs.ends_with('$') {
        if !is_valid_str_var_name(&lhs) {
            return Err(RuntimeError::syntax(format!("Invalid string variable '{}'", lhs)));
        }
        if !(rhs.starts_with('"') && rhs.ends_with('"') && rhs.len() >= 2) {
            return Err(RuntimeError::type_mismatch(
                "Type Mismatch: string variable requires quoted string",
            ));
        }
        let value = rhs[1..rhs.len() - 1].to_string();
        Ok(Command::LetStr { name: lhs, value })
    } else {
        if !is_valid_num_var_name(&lhs) {
            return Err(RuntimeError::syntax(format!("Invalid numeric variable '{}'", lhs)));
        }
        let expr = parse_expr(&rhs.to_ascii_uppercase())?;
        Ok(Command::LetNum { name: lhs, expr })
    }
}