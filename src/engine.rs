use crate::interpreter::{parse_line, Command};
use crate::runtime::Runtime;

pub fn run_program(source: &str, rt: &mut Runtime) -> Result<(), String> {
    for raw in source.lines() {
        match parse_line(raw)? {
            Command::Empty => {}
            Command::Cls => rt.cls(),
            Command::Print(s) => rt.print(s),
        }
    }
    Ok(())
}