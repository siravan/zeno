use anyhow::{anyhow, Result};
use num_complex::Complex;
use std::collections::HashSet;

use crate::config::{Config, SLICE_CAP};
use crate::instruction::{BuiltinSymbol, Slot};

pub trait Composer {
    fn append_constant(&mut self, z: Complex<f64>) -> Result<usize>;
    fn append_add(&mut self, lhs: &Slot, args: &[Slot], num_reals: usize) -> Result<()>;
    fn append_mul(&mut self, lhs: &Slot, args: &[Slot], num_reals: usize) -> Result<()>;
    fn append_pow(&mut self, lhs: &Slot, arg: &Slot, p: i64, is_real: bool) -> Result<()>;
    fn append_powf(&mut self, lhs: &Slot, arg: &Slot, p: &Slot, is_real: bool) -> Result<()>;
    fn append_assign(&mut self, lhs: &Slot, rhs: &Slot) -> Result<()>;
    fn append_label(&mut self, id: usize) -> Result<()>;
    fn append_if_else(&mut self, cond: &Slot, id: usize) -> Result<()>;
    fn append_goto(&mut self, id: usize) -> Result<()>;
    fn append_external_fun(&mut self, lhs: &Slot, op: &str, args: &[Slot]) -> Result<()>;
    fn append_fun(&mut self, lhs: &Slot, fun: &str, args: &[Slot], is_real: bool) -> Result<()>;
    fn append_fun_v1(
        &mut self,
        lhs: &Slot,
        fun: &BuiltinSymbol,
        arg: &Slot,
        is_real: bool,
    ) -> Result<()>;
    fn append_join(
        &mut self,
        lhs: &Slot,
        cond: &Slot,
        true_val: &Slot,
        false_val: &Slot,
    ) -> Result<()>;
}
