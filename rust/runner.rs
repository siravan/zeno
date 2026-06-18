use num_complex::Complex;

use crate::application::Application;
use crate::bytecode::*;
use crate::config::Config;

pub fn bool_to_f64(b: bool) -> f64 {
    const T: f64 = f64::from_bits(!0);
    const F: f64 = f64::from_bits(0);
    if b {
        T
    } else {
        F
    }
}

pub trait Runner {
    fn evaluate(&mut self, args: &[f64], outs: &mut [f64]);
    fn evaluate_matrix(&mut self, args: &[f64], outs: &mut [f64], n: usize);
    fn add_constant(&mut self, z: Complex<f64>);
}

#[derive(Debug)]
pub struct GenericRealRunner {
    config: Config,
    mem: Vec<f64>,
    code: Vec<u8>,
    words: Vec<u32>,
    next_const: usize,
    count_consts: usize,
    count_params: usize,
    count_outs: usize,
    count_temps: usize,
}

impl GenericRealRunner {
    pub fn new(app: &Application) -> GenericRealRunner {
        let a = &app.analyzer;
        let mem_size = a.count_consts + a.count_params + a.count_outs + a.count_temps;

        GenericRealRunner {
            config: app.config.clone(),
            mem: vec![0.0; mem_size],
            code: app.code.clone(),
            words: app.words.clone(),
            next_const: 0,
            count_consts: a.count_consts,
            count_params: a.count_params,
            count_outs: a.count_outs,
            count_temps: a.count_temps,
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
}

impl Runner for GenericRealRunner {
    pub fn evaluate(&mut self, args: &[f64], outs: &mut [f64]) {
        let first_param = self.count_consts;
        let count_params = self.count_params;
        self.mem[first_param..first_param + count_params].copy_from_slice(args);

        self.exec();

        let first_out = self.count_consts + self.count_params;
        let count_outs = self.count_outs;
        outs.copy_from_slice(&self.mem[first_out..first_out + count_outs]);
    }

    pub fn evaluate_matrix(&mut self, args: &[f64], outs: &mut [f64], n: usize) {
        let first_param = self.count_consts;
        let first_out = self.count_consts + self.count_params;
        let count_params = self.count_params;
        let count_outs = self.count_outs;

        for i in 0..n {
            self.mem[first_param..first_param + count_params]
                .copy_from_slice(&args[i * count_params..(i + 1) * count_params]);

            self.exec();

            outs[i * count_outs..(i + 1) * count_outs]
                .copy_from_slice(&self.mem[first_out..first_out + count_outs]);
        }
    }

    fn add_constant(&mut self, z: Complex<f64>) {
        self.mem[self.next_const] = z.re;
        self.next_const += 1;
    }
}
