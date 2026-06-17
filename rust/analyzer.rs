use anyhow::{anyhow, Result};
use num_complex::Complex;

use crate::composer::Composer;
use crate::config::Config;
use crate::instruction::{BuiltinSymbol, Slot};

#[derive(Debug)]
pub struct Analyzer {
    pub config: Config,
    pub count_consts: usize,
    pub count_params: usize,
    pub count_outs: usize,
    pub count_temps: usize,
    pub count_labels: usize,
}

impl Analyzer {
    pub fn new(config: Config) -> Analyzer {
        Analyzer {
            config,
            count_consts: 0,
            count_params: 0,
            count_outs: 0,
            count_temps: 0,
            count_labels: 0,
        }
    }

    fn process_slot(&mut self, slot: &Slot) -> Result<()> {
        match slot {
            Slot::Const(idx) => {
                self.count_consts = self.count_consts.max(idx + 1);
            }
            Slot::Param(idx) => {
                self.count_params = self.count_params.max(idx + 1);
            }
            Slot::Out(idx) => {
                self.count_outs = self.count_outs.max(idx + 1);
            }
            Slot::Temp(idx) => {
                self.count_temps = self.count_temps.max(idx + 1);
            }
            _ => return Err(anyhow!("Invalid Slot type: {:?}", slot)),
        }

        Ok(())
    }

    fn process_slots(&mut self, slots: &[Slot]) -> Result<()> {
        for slot in slots.iter() {
            self.process_slot(slot)?
        }
        Ok(())
    }
}

impl Composer for Analyzer {
    fn append_constant(&mut self, z: Complex<f64>) -> Result<usize> {
        Ok(0)
    }

    fn append_add(&mut self, lhs: &Slot, args: &[Slot], num_reals: usize) -> Result<()> {
        self.process_slot(lhs)?;
        self.process_slots(args)?;
        Ok(())
    }

    fn append_mul(&mut self, lhs: &Slot, args: &[Slot], num_reals: usize) -> Result<()> {
        self.process_slot(lhs)?;
        self.process_slots(args)?;
        Ok(())
    }

    fn append_pow(&mut self, lhs: &Slot, arg: &Slot, p: i64, is_real: bool) -> Result<()> {
        self.process_slot(lhs)?;
        self.process_slot(arg)?;
        Ok(())
    }

    fn append_powf(&mut self, lhs: &Slot, arg: &Slot, p: &Slot, is_real: bool) -> Result<()> {
        self.process_slot(lhs)?;
        self.process_slot(arg)?;
        self.process_slot(p)?;
        Ok(())
    }

    fn append_assign(&mut self, lhs: &Slot, rhs: &Slot) -> Result<()> {
        self.process_slot(lhs)?;
        self.process_slot(rhs)?;
        Ok(())
    }

    fn append_label(&mut self, id: usize) -> Result<()> {
        self.count_labels = self.count_labels.max(id);
        Ok(())
    }

    fn append_if_else(&mut self, cond: &Slot, id: usize) -> Result<()> {
        self.process_slot(cond)?;
        self.count_labels = self.count_labels.max(id);
        Ok(())
    }

    fn append_goto(&mut self, id: usize) -> Result<()> {
        self.count_labels = self.count_labels.max(id);
        Ok(())
    }

    fn append_external_fun(&mut self, lhs: &Slot, op: &str, args: &[Slot]) -> Result<()> {
        self.process_slot(lhs)?;
        self.process_slots(args)?;
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
        self.process_slot(lhs)?;
        self.process_slots(args)?;
        Ok(())
    }

    fn append_join(
        &mut self,
        lhs: &Slot,
        cond: &Slot,
        true_val: &Slot,
        false_val: &Slot,
    ) -> Result<()> {
        self.process_slot(lhs)?;
        self.process_slot(cond)?;
        self.process_slot(true_val)?;
        self.process_slot(false_val)?;
        Ok(())
    }
}
