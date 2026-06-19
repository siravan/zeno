use anyhow::{anyhow, Result};
use num_complex::Complex;

use crate::analyzer::Analyzer;
use crate::composer::Composer;
use crate::config::Config;
use crate::instruction::{BuiltinSymbol, Slot};
use crate::runner::{GenericComplexRunner, GenericRealRunner, Runner};

use crate::bytecode::*;

#[derive(Debug, Clone)]
pub struct Label {
    ip: u32,
    pos: u32,
    jumps: Vec<usize>,
}

impl Default for Label {
    fn default() -> Self {
        Label {
            ip: 0,
            pos: 0,
            jumps: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct Application {
    pub config: Config,
    pub analyzer: Analyzer,
    pub code: Vec<u8>,
    pub words: Vec<u32>,
    pub labels: Vec<Label>,
    pub consts: Vec<Complex<f64>>,
    pub runner: Option<Box<dyn Runner>>,
}

impl Application {
    pub fn new(config: Config, analyzer: Analyzer) -> Application {
        let count_labels = analyzer.count_labels;

        Application {
            config,
            analyzer,
            code: Vec::new(),
            words: Vec::new(),
            labels: vec![Label::default(); count_labels],
            consts: Vec::new(),
            runner: None,
        }
    }

    pub fn seal(&mut self) {
        self.link();
        self.append_code(RET);

        println!("bytecode length: {}", self.code.len());
        println!("words    length: {}", self.words.len());
        println!("number of temps: {}", self.analyzer.count_temps);

        let mut runner: Box<dyn Runner> = if self.config.is_complex() {
            Box::new(GenericComplexRunner::new(&self))
        } else {
            Box::new(GenericRealRunner::new(&self))
        };

        for z in self.consts.iter() {
            runner.add_constant(*z);
        }

        self.runner = Some(runner);
    }

    fn link(&mut self) {
        for label in self.labels.iter() {
            for j in label.jumps.iter() {
                self.words[*j] = label.ip;
                self.words[*j + 1] = label.pos;
            }
        }
    }

    pub fn count_params(&self) -> usize {
        let k = if self.config.is_complex() { 2 } else { 1 };
        self.analyzer.count_params * k
    }

    pub fn count_outs(&self) -> usize {
        let k = if self.config.is_complex() { 2 } else { 1 };
        self.analyzer.count_outs * k
    }

    fn append_code(&mut self, cmd: u8) {
        self.code.push(cmd);
    }

    fn append_word(&mut self, word: u32) {
        self.words.push(word);
    }

    fn append_slot(&mut self, slot: &Slot) {
        let idx = match slot {
            Slot::Const(idx) => *idx,
            Slot::Param(idx) => *idx + self.analyzer.count_consts,
            Slot::Out(idx) => *idx + self.analyzer.count_consts + self.analyzer.count_params,
            Slot::Temp(idx) => {
                *idx + self.analyzer.count_consts
                    + self.analyzer.count_params
                    + self.analyzer.count_outs
            }
            _ => 0,
        };
        self.append_word(idx as u32);
    }

    fn append_slots(&mut self, slots: &[Slot]) {
        for slot in slots.iter() {
            self.append_slot(slot);
        }
    }

    pub fn evaluate(&mut self, args: &[f64], outs: &mut [f64]) {
        if let Some(runner) = &mut self.runner {
            runner.evaluate(args, outs);
        }
    }

    pub fn evaluate_matrix(&mut self, args: &[f64], outs: &mut [f64], n: usize) {
        if let Some(runner) = &mut self.runner {
            runner.evaluate_matrix(args, outs, n);
        }
    }
}

impl Composer for Application {
    fn append_constant(&mut self, z: Complex<f64>) -> Result<usize> {
        self.consts.push(z);
        Ok(0)
    }

    fn append_add(&mut self, lhs: &Slot, args: &[Slot], num_reals: usize) -> Result<()> {
        let n = args.len();
        assert!(n > 1);

        if n == 2 {
            self.append_code(ADD | LDX | LDY | STX);
        } else {
            self.append_code(ADD | LDX | LDY);
            for _ in 0..n - 3 {
                self.append_code(ADD | LDY);
            }
            self.append_code(ADD | LDY | STX);
        }

        self.append_slots(args);
        self.append_slot(lhs);
        Ok(())
    }

    fn append_mul(&mut self, lhs: &Slot, args: &[Slot], num_reals: usize) -> Result<()> {
        let n = args.len();
        assert!(n > 1);

        if n == 2 {
            self.append_code(MUL | LDX | LDY | STX);
        } else {
            self.append_code(MUL | LDX | LDY);
            for _ in 0..n - 3 {
                self.append_code(MUL | LDY);
            }
            self.append_code(MUL | LDY | STX);
        }

        self.append_slots(args);
        self.append_slot(lhs);
        Ok(())
    }

    fn append_pow(&mut self, lhs: &Slot, arg: &Slot, p: i64, is_real: bool) -> Result<()> {
        self.append_code(POW | LDX | STX);
        self.append_slot(arg);
        self.append_word(p as i32 as u32);
        self.append_slot(lhs);
        Ok(())
    }

    fn append_powf(&mut self, lhs: &Slot, arg: &Slot, p: &Slot, is_real: bool) -> Result<()> {
        self.append_code(POWF | LDX | LDY | STX);
        self.append_slot(arg);
        self.append_slot(p);
        self.append_slot(lhs);
        Ok(())
    }

    fn append_assign(&mut self, lhs: &Slot, rhs: &Slot) -> Result<()> {
        self.append_code(ASSIGN | LDX | STX);
        self.append_slot(rhs);
        self.append_slot(lhs);
        Ok(())
    }

    fn append_label(&mut self, id: usize) -> Result<()> {
        self.labels[id].ip = self.code.len() as u32;
        self.labels[id].pos = self.words.len() as u32;
        Ok(())
    }

    fn append_if_else(&mut self, cond: &Slot, id: usize) -> Result<()> {
        self.append_slot(cond);
        self.labels[id].jumps.push(self.words.len());
        self.append_word(0);
        self.append_word(0);
        self.append_code(BRANCH_ELSE | LDX);
        Ok(())
    }

    fn append_goto(&mut self, id: usize) -> Result<()> {
        self.labels[id].jumps.push(self.words.len());
        self.append_word(0);
        self.append_word(0);
        self.append_code(GOTO);
        Ok(())
    }

    fn append_external_fun(&mut self, lhs: &Slot, op: &str, args: &[Slot]) -> Result<()> {
        Ok(())
    }

    fn append_fun_v1(
        &mut self,
        lhs: &Slot,
        fun: &BuiltinSymbol,
        arg: &Slot,
        is_real: bool,
    ) -> Result<()> {
        Ok(())
    }

    fn append_fun(&mut self, lhs: &Slot, fun: &str, args: &[Slot], is_real: bool) -> Result<()> {
        let fun = fun.strip_prefix("symbolica_").unwrap_or(fun);

        if ["lt", "leq", "gt", "geq", "eq", "neq"].contains(&fun) {
            self.append_code(MOVZ | LDX | LDY);
            let cmd = match fun {
                "lt" => LT,
                "leq" => LEQ,
                "gt" => GT,
                "geq" => GEQ,
                "eq" => EQ,
                "neq" => NEQ,
                _ => return Err(anyhow!("undefined comparison: {}", fun)),
            };
            self.append_code(cmd | STX);
        } else if fun == "square" {
            self.append_code(DUP | LDX);
            self.append_code(MUL | STX);
        } else if fun == "cube" {
            self.append_code(DUP | LDX);
            self.append_code(MUL);
            self.append_code(MUL | STX);
        } else {
            let cmd = match fun {
                "abs" => ABS,
                "neg" => NEG,
                "recip" => RECIP,
                "not" => NOT,
                "root" | "sqrt" => ROOT,
                "root_real" | "sqrt_real" => ROOT_REAL,
                "round" => ROUND,
                "floor" => FLOOR,
                "real" => REAL,
                "imaginary" => IMAGINARY,
                "conjugate" => CONJUGATE,
                "iszero" => ISZERO,
                _ => return Err(anyhow!("undefined function: {}", fun)),
            };
            self.append_code(cmd | LDX | STX);
        }

        self.append_slots(args);
        self.append_slot(lhs);

        Ok(())
    }

    fn append_join(
        &mut self,
        lhs: &Slot,
        cond: &Slot,
        true_val: &Slot,
        false_val: &Slot,
    ) -> Result<()> {
        self.append_code(MOVZ | LDX | LDY);
        self.append_code(JOIN | LDX | STX);
        self.append_slot(true_val);
        self.append_slot(false_val);
        self.append_slot(cond);
        self.append_slot(lhs);

        Ok(())
    }
}
