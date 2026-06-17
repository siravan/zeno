use anyhow::{anyhow, Result};
use num_complex::Complex;

use crate::analyzer::Analyzer;
use crate::composer::Composer;
use crate::config::Config;
use crate::instruction::{BuiltinSymbol, Slot};

#[derive(Debug)]
pub struct Application {
    config: Config,
    analyzer: Analyzer,
    mem: Vec<f64>,
    code: Vec<u8>,
    words: Vec<u32>,
    ip: usize,
    pos: usize,
}

impl Application {
    pub fn new(config: Config, analyzer: Analyzer) -> Application {
        let mem_size = analyzer.count_consts
            + analyzer.count_params
            + analyzer.count_outs
            + analyzer.count_temps;

        Application {
            config,
            analyzer,
            mem: vec![0.0; mem_size],
            code: Vec::new(),
            words: Vec::new(),
            pos: 0,
            ip: 0,
        }
    }
}

impl Composer for Application {
    fn append_constant(&mut self, z: Complex<f64>) -> Result<usize> {
        self.mem[self.pos] = z.re;
        self.pos += 1;
        Ok(0)
    }

    fn append_add(&mut self, lhs: &Slot, args: &[Slot], num_reals: usize) -> Result<()> {
        Ok(())
    }

    fn append_mul(&mut self, lhs: &Slot, args: &[Slot], num_reals: usize) -> Result<()> {
        Ok(())
    }

    fn append_pow(&mut self, lhs: &Slot, arg: &Slot, p: i64, is_real: bool) -> Result<()> {
        Ok(())
    }

    fn append_powf(&mut self, lhs: &Slot, arg: &Slot, p: &Slot, is_real: bool) -> Result<()> {
        Ok(())
    }

    fn append_assign(&mut self, lhs: &Slot, rhs: &Slot) -> Result<()> {
        Ok(())
    }

    fn append_label(&mut self, id: usize) -> Result<()> {
        Ok(())
    }

    fn append_if_else(&mut self, cond: &Slot, id: usize) -> Result<()> {
        Ok(())
    }

    fn append_goto(&mut self, id: usize) -> Result<()> {
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
        Ok(())
    }

    fn append_join(
        &mut self,
        lhs: &Slot,
        cond: &Slot,
        true_val: &Slot,
        false_val: &Slot,
    ) -> Result<()> {
        Ok(())
    }
}
