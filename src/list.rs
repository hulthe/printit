use crate::ansi_colors;
use rustyline::completion::Completer;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::Helper;
use std::borrow::Cow;

lazy_static! {
    pub static ref PRINTER_LIST: Vec<&'static str> = {
        let s = include_str!("printers.txt");
        s.lines().collect()
    };
}

pub struct PrinterInput;

impl Completer for PrinterInput {
    type Candidate = &'static str;

    fn complete(
        &self,
        line: &str,
        _pos: usize,
        _ctx: &rustyline::Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        let mut matches: Vec<_> = PRINTER_LIST
            .iter()
            .copied()
            .map(|printer| (printer, fuzzy_compare(printer, line)))
            .collect();

        let max_score = *matches.iter().map(|(_, score)| score).max().unwrap_or(&0);
        matches.retain(|&(_, score)| score == max_score);

        Ok((0, matches.into_iter().map(|(printer, _)| printer).collect()))
    }

    fn update(&self, line: &mut rustyline::line_buffer::LineBuffer, _start: usize, elected: &str) {
        line.update(elected, elected.len());
    }
}

/// Compare two strings and assign a score to how well the search string matches the base string
pub fn fuzzy_compare(base: &str, search: &str) -> i32 {
    let mut base = base.chars();

    // How alike the search string is to self.name
    let mut score = 0;

    for sc in search.chars() {
        let sc = sc.to_ascii_lowercase();
        let mut add = 3;
        let mut base_tmp = base.clone();
        while let Some(bc) = base_tmp.next() {
            let bc = bc.to_ascii_lowercase();
            if bc == sc {
                score += add;
                base = base_tmp;
                break;
            } else {
                add = 2;
            }
        }
    }

    score
}

impl Hinter for PrinterInput {
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, _ctx: &rustyline::Context<'_>) -> Option<Self::Hint> {
        if line.is_empty() {
            return None;
        }

        // Draw the hint as part of the query
        let hint_completion = || {
            if pos != line.len() {
                return None;
            }

            PRINTER_LIST
                .iter()
                .rev()
                .copied()
                .filter(|printer| printer.starts_with(line))
                .filter(|printer| pos <= printer.len())
                .max_by_key(|printer| fuzzy_compare(printer, line))
                .map(|printer| &printer[pos..])
                .map(|printer| printer.to_string())
        };

        // Draw the hint after the query, within parenthesis
        let hint_fuzzy_completion = || {
            PRINTER_LIST
                .iter()
                .rev()
                .copied()
                .max_by_key(|printer| fuzzy_compare(printer, line))
                .map(|printer| format!("  ({})", printer))
        };

        hint_completion().or_else(hint_fuzzy_completion)
    }
}

impl Validator for PrinterInput {}

impl Highlighter for PrinterInput {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> std::borrow::Cow<'l, str> {
        Cow::Owned(ansi_colors::MAGENTA.to_owned() + line + ansi_colors::RESET)
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> std::borrow::Cow<'h, str> {
        Cow::Owned(ansi_colors::DIM.to_owned() + hint + ansi_colors::RESET)
    }
}

impl Helper for PrinterInput {}
