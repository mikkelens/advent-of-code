use crate::Runnable;
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;
use std::fmt::Debug;

pub struct Solution;
impl Runnable for Solution {
    fn run_with_input(&self, input: String) {
        println!(
            "Crates on top of each stack after moves: {}",
            part_1_solve(input.as_str())
        );
    }
}
impl CrateStack {
    #[allow(dead_code)]
    fn new() -> Self {
        CrateStack { crates: Vec::new() }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct CrateStack {
    crates: Vec<Crate>,
}
impl From<&str> for CrateStack {
    fn from(value: &str) -> Self {
        CrateStack {
            crates: value.chars().map(Crate::from).collect(),
        }
    }
}

#[derive(PartialEq, Copy, Clone, Debug)]
struct Crate {
    label: char,
}

impl From<char> for Crate {
    fn from(label: char) -> Crate {
        Crate { label }
    }
}
impl TryFrom<&str> for Crate {
    type Error = &'static str;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        lazy_static! {
            static ref RE_LABEL: Regex = Regex::new(r"[A-Z]").unwrap();
        }

        let label = RE_LABEL
            .find(s)
            .ok_or("Could not find a label in string literal.")?
            .as_str();
        let label: char = label.chars().next().ok_or("Label was not a char.")?;
        Ok(Crate { label })
    }
}

fn add_crate_line_to_stacks(stacks: &mut HashMap<usize, CrateStack>, line: &str) {
    lazy_static! {
        static ref RE_CRATE_PATTERN: Regex = Regex::new(r"    |\[[A-Z]\]").unwrap();
    }

    let line_crates: Vec<Option<Crate>> = RE_CRATE_PATTERN
        .find_iter(line)
        .map(|c| Crate::try_from(c.as_str()).ok())
        .collect();
    for crate_key in 1..=line_crates.len() {
        let option_crate = line_crates[crate_key - 1]; // copy
        if let Some(c) = option_crate {
            stacks.entry(crate_key).or_insert_with(CrateStack::new);
            stacks.get_mut(&crate_key).unwrap().crates.push(c);
        }
    }
    println!("Finished line!");
}

#[derive(Debug)]
struct Move {
    count: usize,
    source: usize,
    target: usize,
}
impl TryFrom<&str> for Move {
    type Error = &'static str;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        lazy_static! {
            static ref RE_COUNT: Regex = Regex::new(r"move [0-99]").unwrap(); // replace with unspecific number?
            static ref RE_FROM_INDEX: Regex = Regex::new(r"from [0-99]").unwrap(); // replace with unspecific number?
            static ref RE_TO_INDEX: Regex = Regex::new(r"to [0-99]").unwrap(); // replace with unspecific number?
        }
        let from = RE_FROM_INDEX.find(s).ok_or("Missing segment: 'from'")?.as_str();
        let from: usize = from.split_once(' ').unwrap().1.parse().unwrap();
        let count = RE_COUNT.find(s).ok_or("Missing segment: 'count'")?.as_str();
        let count: usize = count.split_once(' ').unwrap().1.parse().unwrap();
        let to = RE_TO_INDEX.find(s).ok_or("Missing segmnet: 'to'")?.as_str();
        let to: usize = to.split_once(' ').unwrap().1.parse().unwrap();
        Ok(Move {
            source: from,
            count,
            target: to,
        })
    }
}

fn remove_crate_from_stack(stack: &mut CrateStack) -> Crate {
    stack.crates.remove(stack.crates.len() - 1)
}
fn process_move_on_stacks(stacks: &mut HashMap<usize, CrateStack>, mut move_to_process: Move) {
    // build move
    let max_crates_to_take = stacks.get(&move_to_process.source).unwrap().crates.len();
    if max_crates_to_take < move_to_process.count {
        move_to_process.count = max_crates_to_take;
    }
    // use move
    let mut crates_taken: Vec<Crate> = Vec::new();
    let from_stack = stacks.get_mut(&move_to_process.source).unwrap();
    println!(
        "Stack {} before move: {:?}",
        &move_to_process.source, &from_stack.crates
    );
    for _ in 0..move_to_process.count {
        if !from_stack.crates.is_empty() {
            crates_taken.push(remove_crate_from_stack(from_stack));
        }
    }
    println!(
        "Moved {} crates from stack {} to stack {}: {:?}",
        &move_to_process.count, &move_to_process.source, &move_to_process.target, &crates_taken
    );

    let to_stack = stacks.get_mut(&move_to_process.target).unwrap();
    for crate_taken in crates_taken {
        to_stack.crates.push(crate_taken);
    }
}

fn part_1_solve(input: &str) -> String {
    let parts = match input.split_once("\r\n\r\n") {
        Some(p) => p, // string literal from .txt file
        None => input.split_once("\n\n").unwrap(), // string literal in .rs file
    };

    // create starting condition
    let mut stacks: HashMap<usize, CrateStack> = HashMap::new();
    for line_of_crates in parts.0.lines().rev() {
        add_crate_line_to_stacks(&mut stacks, line_of_crates);
    }

    // operate on state using moves (lines)
    for line_of_move in parts.1.lines() {
        let new_move = Move::try_from(line_of_move).expect("Invalid line for move");
        process_move_on_stacks(&mut stacks, new_move);
    }

    // print the top crates of each stack (skip over empty)
    get_top_labels(&stacks)
}

fn get_top_labels(stacks: &HashMap<usize, CrateStack>) -> String {
    let mut top: String = String::new();
    for stack_index in 1..=stacks.keys().len() {
        let stack = stacks.get(&stack_index).unwrap();
        let top_crate = stack.crates.last();
        if let Some(top_crate) = top_crate {
            top.push(top_crate.label);
        };
    }
    top
}

#[cfg(test)]
mod tests {
    use super::*;
    const TEST_INPUT: &str = "    [D]
[N] [C]
[Z] [M] [P]
 1   2   3

move 1 from 2 to 1
move 3 from 1 to 3
move 2 from 2 to 1
move 1 from 1 to 2";
    #[test]
    fn part_1_example_works() {
        assert_eq!("CMZ", part_1_solve(TEST_INPUT))
    }
    #[test]
    fn creates_empty_crate() {
        assert_eq!(None, Crate::try_from("   ").ok());
    }
    #[test]
    fn creates_real_crate() {
        assert_eq!(Some(Crate::from('C')), Crate::try_from("[C]").ok());
    }
    #[test]
    fn adds_crates_properly() {
        let line = "        [T]";
        let mut stacks: HashMap<usize, CrateStack> = HashMap::from([
            (1, CrateStack::from("CF")),
            (2, CrateStack::from("L")),
            (3, CrateStack::from("AQ")),
        ]);
        add_crate_line_to_stacks(&mut stacks, line);
        let expected_stacks: HashMap<usize, CrateStack> = HashMap::from([
            (1, CrateStack::from("CF")),
            (2, CrateStack::from("L")),
            (3, CrateStack::from("AQT")),
        ]);
        assert_eq!(&expected_stacks, &stacks);
    }
    #[test]
    fn crates_move_correctly() {
        let stacks_source: HashMap<usize, CrateStack> = HashMap::from([
            (1, CrateStack::from("CF")),
            (2, CrateStack::from("L")),
            (3, CrateStack::from("A")),
        ]);

        // too big for stack
        let big_move = Move {
            count: 10,
            source: 1,
            target: 3,
        };
        let mut big_move_stacks = stacks_source.clone();
        process_move_on_stacks(&mut big_move_stacks, big_move);
        let big_move_expectation: HashMap<usize, CrateStack> = HashMap::from([
            (1, CrateStack::new()),
            (2, CrateStack::from("L")),
            (3, CrateStack::from("AFC")),
        ]);
        assert_eq!(&big_move_expectation, &big_move_stacks);

        // smaller than stack
        let small_move = Move {
            count: 1,
            source: 1,
            target: 3,
        };
        let mut small_move_stacks = stacks_source.clone();
        process_move_on_stacks(&mut small_move_stacks, small_move);
        let small_move_expectation: HashMap<usize, CrateStack> = HashMap::from([
            (1, CrateStack::from("C")),
            (2, CrateStack::from("L")),
            (3, CrateStack::from("AF")),
        ]);
        assert_eq!(&small_move_expectation, &small_move_stacks);

        // move doesn't take crates (count is 0)
        let empty_move = Move {
            count: 0,
            source: 1,
            target: 3,
        };
        let mut empty_move_stacks = stacks_source.clone();
        process_move_on_stacks(&mut empty_move_stacks, empty_move);
        let empty_move_expectation: HashMap<usize, CrateStack> = stacks_source; // we expect that nothing changes
        assert_eq!(&empty_move_expectation, &empty_move_stacks);
    }
    #[test]
    fn top_is_correct() {
        let stacks: HashMap<usize, CrateStack> = HashMap::from([
            (1, CrateStack::from("HL")),
            (2, CrateStack::new()),
            (3, CrateStack::from("AF")),
            (4, CrateStack::from("SIQ")),
        ]);
        assert_eq!("LFQ".to_string(), get_top_labels(&stacks))
    }
}