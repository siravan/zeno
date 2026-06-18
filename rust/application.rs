use anyhow::{anyhow, Result};
use num_complex::Complex;

use crate::analyzer::Analyzer;
use crate::composer::Composer;
use crate::config::Config;
use crate::instruction::{BuiltinSymbol, Slot};
use crate::runner::{GenericRealRunner, Runner};

use crate::bytecode::*;

pub fn bool_to_f64(b: bool) -> f64 {
    const T: f64 = f64::from_bits(!0);
    const F: f64 = f64::from_bits(0);
    if b {
        T
    } else {
        F
    }
}

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
    pub mem: Vec<f64>,
    pub code: Vec<u8>,
    pub words: Vec<u32>,
    pub next_const: usize,
    pub labels: Vec<Label>,
}

impl Application {
    pub fn new(config: Config, analyzer: Analyzer) -> Application {
        let mem_size = analyzer.count_consts
            + analyzer.count_params
            + analyzer.count_outs
            + analyzer.count_temps;

        let count_labels = analyzer.count_labels;

        Application {
            config,
            analyzer,
            mem: vec![0.0; mem_size],
            code: Vec::new(),
            words: Vec::new(),
            next_const: 0,
            labels: vec![Label::default(); count_labels],
        }
    }

    pub fn seal(&mut self) {
        self.link();
        self.append_code(RET);
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
        self.analyzer.count_params
    }

    pub fn count_outs(&self) -> usize {
        self.analyzer.count_outs
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

    fn exec(&mut self) {
        let mut ip: usize = 0;
        let mut pos: usize = 0;
        let mut x: f64 = 0.0;
        let mut y: f64 = 0.0;
        let mut z: f64 = 0.0;

        loop {
            let cmd = self.code[ip];
            // println!("{}, ip = {}, pos = {}", cmd, ip, pos);
            ip += 1;

            if cmd & LDX != 0 {
                x = self.mem[self.words[pos] as usize];
                pos += 1;
            }

            if cmd & BINOP != 0 {
                if cmd & LDY != 0 {
                    y = self.mem[self.words[pos] as usize];
                    pos += 1;
                }

                match cmd & (0x0f | BINOP) {
                    MUL => x *= y,
                    ADD => x += y,
                    SUB => x -= y,
                    DIV => x /= y,
                    POWF => x = x.powf(y),
                    AND => x = f64::from_bits(x.to_bits() & y.to_bits()),
                    OR => x = f64::from_bits(x.to_bits() | y.to_bits()),
                    XOR => x = f64::from_bits(x.to_bits() ^ y.to_bits()),
                    COMPLEX => {}
                    MOVZ => z = x,
                    _ => panic!("unrecognized binary op-code: {}", cmd),
                }
            } else {
                match cmd & 0x1f {
                    ASSIGN => {}
                    NEG => x = -x,
                    NOT => x = f64::from_bits(!x.to_bits()),
                    RECIP => x = 1.0 / x,
                    ABS => x = x.abs(),
                    ROOT | ROOT_REAL => x = x.sqrt(),
                    POW => {
                        let p = self.words[pos] as i32;
                        pos += 1;
                        x = x.powi(p);
                    }
                    ROUND => x = x.round(),
                    FLOOR => x = x.floor(),
                    REAL => {}
                    IMAGINARY => x = 0.0,
                    CONJUGATE => {}
                    ISZERO => x = bool_to_f64(x == 0.0),
                    GOTO => {
                        ip = self.words[pos] as usize;
                        pos = self.words[pos + 1] as usize;
                    }
                    BRANCH_IF => {
                        if x != 0.0 {
                            ip = self.words[pos] as usize;
                            pos = self.words[pos + 1] as usize;
                        } else {
                            pos += 2;
                        }
                    }
                    BRANCH_ELSE => {
                        if x == 0.0 {
                            ip = self.words[pos] as usize;
                            pos = self.words[pos + 1] as usize;
                        } else {
                            pos += 2;
                        }
                    }
                    JOIN => x = if x != 0.0 { z } else { y },
                    GT => x = bool_to_f64(x > y),
                    GEQ => x = bool_to_f64(x >= y),
                    LT => x = bool_to_f64(x < y),
                    LEQ => x = bool_to_f64(x <= y),
                    EQ => x = bool_to_f64(x == y),
                    NEQ => x = bool_to_f64(x != y),
                    DUP => y = x,
                    RET => break,
                    _ => panic!("unrecognized unary op-code: {}", cmd),
                }
            }

            if cmd & STX != 0 {
                self.mem[self.words[pos] as usize] = x;
                pos += 1;
            }
        }
    }

    pub fn evaluate(&mut self, args: &[f64], outs: &mut [f64]) {
        let first_param = self.analyzer.count_consts;
        let count_params = self.analyzer.count_params;
        self.mem[first_param..first_param + count_params].copy_from_slice(args);

        self.exec();

        let first_out = self.analyzer.count_consts + self.analyzer.count_params;
        let count_outs = self.analyzer.count_outs;
        outs.copy_from_slice(&self.mem[first_out..first_out + count_outs]);
    }

    pub fn evaluate_matrix(&mut self, args: &[f64], outs: &mut [f64], n: usize) {
        let first_param = self.analyzer.count_consts;
        let first_out = self.analyzer.count_consts + self.analyzer.count_params;
        let count_params = self.analyzer.count_params;
        let count_outs = self.analyzer.count_outs;

        for i in 0..n {
            self.mem[first_param..first_param + count_params]
                .copy_from_slice(&args[i * count_params..(i + 1) * count_params]);

            self.exec();

            outs[i * count_outs..(i + 1) * count_outs]
                .copy_from_slice(&self.mem[first_out..first_out + count_outs]);
        }
    }
}

impl Composer for Application {
    fn append_constant(&mut self, z: Complex<f64>) -> Result<usize> {
        self.mem[self.next_const] = z.re;
        self.next_const += 1;
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
                "neg" => NEG,
                "recip" => RECIP,
                "not" => NOT,
                "root" => ROOT,
                "root_real" => ROOT_REAL,
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
