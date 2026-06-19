use anyhow::{anyhow, Result};
use num_complex::Complex;

use crate::analyzer::Analyzer;
use crate::application::Application;
use crate::composer::Composer;
use crate::config::Config;
use crate::instruction::{Instruction, SymbolicaModel};
use crate::parser::Parser;

pub struct Translator {
    config: Config,
}

impl Translator {
    pub fn new(config: Config) -> Translator {
        Translator { config }
    }

    pub fn translate(&mut self, instructions: String, num_params: usize) -> Result<Application> {
        let model: SymbolicaModel = Parser::new(instructions).parse()?;
        let analyzer = self.anazlyze(&model)?;
        // println!("{:?}", &analyzer);
        // translator.set_num_params(num_params);
        let app = self.compile(&model, analyzer)?;
        Ok(app)
    }

    fn anazlyze(&mut self, model: &SymbolicaModel) -> Result<Analyzer> {
        let mut analyzer = Analyzer::new(self.config.clone());
        self.convert(model, &mut analyzer)?;
        Ok(analyzer)
    }

    fn compile(&mut self, model: &SymbolicaModel, analyzer: Analyzer) -> Result<Application> {
        let mut app = Application::new(self.config.clone(), analyzer);

        for c in model.2.iter() {
            let val = Complex::new(c.value().re, c.value().im);
            app.append_constant(val)?;
        }

        self.convert(model, &mut app)?;
        app.seal();

        Ok(app)
    }

    fn convert<C: Composer>(&mut self, model: &SymbolicaModel, composer: &mut C) -> Result<()> {
        for line in model.0.iter() {
            match line {
                Instruction::Add(lhs, args, num_reals) => {
                    composer.append_add(lhs, args, *num_reals)?
                }
                Instruction::Mul(lhs, args, num_reals) => {
                    composer.append_mul(lhs, args, *num_reals)?
                }
                Instruction::Pow(lhs, arg, p, is_real) => {
                    composer.append_pow(lhs, arg, *p, *is_real)?
                }
                Instruction::Powf(lhs, arg, p, is_real) => {
                    composer.append_powf(lhs, arg, p, *is_real)?
                }
                Instruction::Assign(lhs, rhs) => composer.append_assign(lhs, rhs)?,
                Instruction::Fun(lhs, fun, args, is_real) => {
                    composer.append_fun(lhs, fun, args, *is_real)?
                }
                Instruction::Join(lhs, cond, true_val, false_val) => {
                    // self.depth -= 1;
                    composer.append_join(lhs, cond, true_val, false_val)?
                }
                Instruction::Label(id) => composer.append_label(*id)?,
                Instruction::IfElse(cond, id) => {
                    composer.append_if_else(cond, *id)?;
                    // self.depth += 1;
                }
                Instruction::Goto(id) => composer.append_goto(*id)?,
                Instruction::ExternalFun(lhs, op, args) => {
                    composer.append_external_fun(lhs, op, args)?
                }
            }
        }

        Ok(())
    }
}
