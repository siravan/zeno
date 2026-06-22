use num_complex::Complex;

use crate::application::Application;
use crate::bytecode::*;
use crate::config::Config;

use wide::f64x4;

pub fn bool_to_f64(b: bool) -> f64 {
    const T: f64 = f64::from_bits(!0);
    const F: f64 = f64::from_bits(0);
    if b {
        T
    } else {
        F
    }
}

pub fn bool_to_c128(b: bool) -> Complex<f64> {
    const T: f64 = f64::from_bits(!0);
    const F: f64 = f64::from_bits(0);
    if b {
        Complex::new(T, T)
    } else {
        Complex::new(F, F)
    }
}

pub fn recast_as_f64<T>(v: &[T]) -> &[f64]
where
    T: Sized,
{
    let s = std::mem::size_of::<T>() / std::mem::size_of::<f64>();
    let p: *const f64 = v.as_ptr() as _;
    let q: &[f64] = unsafe { std::slice::from_raw_parts(p, s * v.len()) };
    q
}

pub fn recast_as_f64_mut<T>(v: &mut [T]) -> &mut [f64]
where
    T: Sized,
{
    let s = std::mem::size_of::<T>() / std::mem::size_of::<f64>();
    let p: *mut f64 = v.as_ptr() as _;
    let q: &mut [f64] = unsafe { std::slice::from_raw_parts_mut(p, s * v.len()) };
    q
}

pub fn recast_as_c128(v: &[f64]) -> &[Complex<f64>] {
    let p: *const Complex<f64> = v.as_ptr() as _;
    let q: &[Complex<f64>] = unsafe { std::slice::from_raw_parts(p, v.len() / 2) };
    q
}

pub fn recast_as_c128_mut(v: &mut [f64]) -> &mut [Complex<f64>] {
    let p: *mut Complex<f64> = v.as_ptr() as _;
    let q: &mut [Complex<f64>] = unsafe { std::slice::from_raw_parts_mut(p, v.len() / 2) };
    q
}

pub trait Runner: std::fmt::Debug {
    fn evaluate(&mut self, args: &[f64], outs: &mut [f64]);
    fn evaluate_matrix(&mut self, args: &[f64], outs: &mut [f64], n: usize);
    fn add_constant(&mut self, z: Complex<f64>);
}

/* RealRunner */

#[derive(Debug)]
pub struct GenericRealRunner {
    mem: Vec<f64>,
    mem_f64x4: Vec<f64x4>,
    code: Vec<u8>,
    words: Vec<u32>,
    next_const: usize,
    count_consts: usize,
    count_params: usize,
    count_outs: usize,
    ker: fn(code: &[u8], words: &[u32], mem: &mut [f64]),
    ker_f64x4: Option<fn(code: &[u8], words: &[u32], mem: &mut [f64x4])>,
    num_lanes: usize,
}

impl GenericRealRunner {
    pub fn new(app: &Application) -> GenericRealRunner {
        let a = &app.analyzer;
        let mem_size = a.count_consts + a.count_params + a.count_outs + a.count_temps;

        let ker = if app.config.is_bytecode() {
            kernel_scalar
        } else {
            stub_x64_scalar
        };

        GenericRealRunner {
            mem: vec![0.0; mem_size],
            mem_f64x4: vec![f64x4::default(); mem_size],
            code: app.code.clone(),
            words: app.words.clone(),
            next_const: 0,
            count_consts: a.count_consts,
            count_params: a.count_params,
            count_outs: a.count_outs,
            ker,
            ker_f64x4: None,
            num_lanes: 1,
        }
    }
}

fn kernel_scalar(code: &[u8], words: &[u32], mem: &mut [f64]) {
    let mut ip: usize = 0;
    let mut pos: usize = 0;
    let mut x: f64 = 0.0;
    let mut y: f64 = 0.0;
    let mut z: f64 = 0.0;

    loop {
        let cmd = code[ip];
        // println!("{}, ip = {}, pos = {}", cmd, ip, pos);
        ip += 1;

        if cmd & LDX != 0 {
            x = mem[words[pos] as usize];
            pos += 1;
        }

        if cmd & BINOP != 0 {
            if cmd & LDY != 0 {
                y = mem[words[pos] as usize];
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
                    let p = words[pos] as i32;
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
                    ip = words[pos] as usize;
                    pos = words[pos + 1] as usize;
                }
                BRANCH_IF => {
                    if x != 0.0 {
                        ip = words[pos] as usize;
                        pos = words[pos + 1] as usize;
                    } else {
                        pos += 2;
                    }
                }
                BRANCH_ELSE => {
                    if x == 0.0 {
                        ip = words[pos] as usize;
                        pos = words[pos + 1] as usize;
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
            mem[words[pos] as usize] = x;
            pos += 1;
        }
    }
}

extern "C" {
    fn ker_scalar(code: *const u8, words: *const u32, mem: *mut f64) -> u64;
}

fn stub_x64_scalar(code: &[u8], words: &[u32], mem: &mut [f64]) {
    unsafe {
        let code: *const u8 = code.as_ptr();
        let words: *const u32 = words.as_ptr();
        let mem: *mut f64 = mem.as_mut_ptr();
        let ret = ker_scalar(code, words, mem);

        if ret != 0 {
            panic!("asm scalar kernel returns error: {:x}", ret);
        }
    }
}

impl Runner for GenericRealRunner {
    fn evaluate(&mut self, args: &[f64], outs: &mut [f64]) {
        let first_param = self.count_consts;
        let count_params = self.count_params;
        self.mem[first_param..first_param + count_params].copy_from_slice(args);

        (self.ker)(&self.code, &self.words, &mut self.mem);

        let first_out = self.count_consts + self.count_params;
        let count_outs = self.count_outs;
        outs.copy_from_slice(&self.mem[first_out..first_out + count_outs]);
    }

    fn evaluate_matrix(&mut self, args: &[f64], outs: &mut [f64], n: usize) {
        let first_param = self.count_consts;
        let first_out = self.count_consts + self.count_params;
        let count_params = self.count_params;
        let count_outs = self.count_outs;

        for i in 0..n {
            self.mem[first_param..first_param + count_params]
                .copy_from_slice(&args[i * count_params..(i + 1) * count_params]);

            (self.ker)(&self.code, &self.words, &mut self.mem);

            outs[i * count_outs..(i + 1) * count_outs]
                .copy_from_slice(&self.mem[first_out..first_out + count_outs]);
        }
    }

    fn add_constant(&mut self, z: Complex<f64>) {
        self.mem[self.next_const] = z.re;
        self.next_const += 1;
    }
}

/* ComplexRunner */

#[derive(Debug)]
pub struct GenericComplexRunner {
    mem: Vec<Complex<f64>>,
    mem_f64x4: Vec<Complex<f64x4>>,
    code: Vec<u8>,
    words: Vec<u32>,
    next_const: usize,
    count_consts: usize,
    count_params: usize,
    count_outs: usize,
    ker: fn(code: &[u8], words: &[u32], mem: &mut [Complex<f64>]),
    ker_f64x4: Option<fn(code: &[u8], words: &[u32], mem: &mut [Complex<f64x4>])>,
    num_lanes: usize,
}

impl GenericComplexRunner {
    pub fn new(app: &Application) -> GenericComplexRunner {
        let a = &app.analyzer;
        let mem_size = a.count_consts + a.count_params + a.count_outs + a.count_temps;

        let ker = if app.config.is_bytecode() {
            kernel_complex
        } else {
            stub_x64_complex
        };

        let ker_f64x4: Option<fn(code: &[u8], words: &[u32], mem: &mut [Complex<f64x4>])> =
            if app.config.is_bytecode() || !app.config.use_simd() {
                None
            } else {
                Some(stub_x64x4_complex)
            };

        GenericComplexRunner {
            mem: vec![Complex::<f64>::default(); mem_size],
            mem_f64x4: vec![Complex::<f64x4>::default(); mem_size],
            code: app.code.clone(),
            words: app.words.clone(),
            next_const: 0,
            count_consts: a.count_consts,
            count_params: a.count_params,
            count_outs: a.count_outs,
            ker,
            ker_f64x4,
            num_lanes: 4,
        }
    }

    fn pre_transpose(src: &[Complex<f64>], dst: &mut [Complex<f64x4>]) {
        assert!(src.len() == dst.len() * 4);
        let r = dst.len();

        for i in 0..r {
            let re = f64x4::new([
                src[i].re,
                src[i + r].re,
                src[i + 2 * r].re,
                src[i + 3 * r].re,
            ]);
            let im = f64x4::new([
                src[i].im,
                src[i + r].im,
                src[i + 2 * r].im,
                src[i + 3 * r].im,
            ]);
            dst[i] = Complex::new(re, im);
        }
    }

    fn post_transpose(src: &[Complex<f64x4>], dst: &mut [Complex<f64>]) {
        assert!(dst.len() == src.len() * 4);
        let r = src.len();

        for i in 0..r {
            let [x0, x1, x2, x3] = src[i].re.as_array();
            let [y0, y1, y2, y3] = src[i].im.as_array();

            dst[i] = Complex::new(*x0, *y0);
            dst[i + r] = Complex::new(*x1, *y1);
            dst[i + 2 * r] = Complex::new(*x2, *y2);
            dst[i + 3 * r] = Complex::new(*x3, *y3);
        }
    }
}

fn kernel_complex(code: &[u8], words: &[u32], mem: &mut [Complex<f64>]) {
    let mut ip: usize = 0;
    let mut pos: usize = 0;
    let mut x = Complex::<f64>::default();
    let mut y = Complex::<f64>::default();
    let mut z = Complex::<f64>::default();

    loop {
        let cmd = code[ip];
        // println!("{}, ip = {}, pos = {}", cmd, ip, pos);
        ip += 1;

        if cmd & LDX != 0 {
            x = mem[words[pos] as usize];
            pos += 1;
        }

        if cmd & BINOP != 0 {
            if cmd & LDY != 0 {
                y = mem[words[pos] as usize];
                pos += 1;
            }

            match cmd & (0x0f | BINOP) {
                MUL => x *= y,
                ADD => x += y,
                SUB => x -= y,
                DIV => x /= y,
                POWF => x = x.powf(y.re),
                AND => {
                    x.re = f64::from_bits(x.re.to_bits() & y.re.to_bits());
                    x.im = f64::from_bits(x.im.to_bits() & y.im.to_bits());
                }
                OR => {
                    x.re = f64::from_bits(x.re.to_bits() | y.re.to_bits());
                    x.im = f64::from_bits(x.im.to_bits() | y.im.to_bits());
                }
                XOR => {
                    x.re = f64::from_bits(x.re.to_bits() ^ y.re.to_bits());
                    x.im = f64::from_bits(x.im.to_bits() ^ y.im.to_bits());
                }
                COMPLEX => x = Complex::new(x.re, y.re),
                MOVZ => z = x,
                _ => panic!("unrecognized binary op-code: {}", cmd),
            }
        } else {
            match cmd & 0x1f {
                ASSIGN => {}
                NEG => x = -x,
                NOT => {
                    x.re = f64::from_bits(!x.re.to_bits());
                    x.im = f64::from_bits(!x.im.to_bits());
                }
                RECIP => x = 1.0 / x,
                ABS => x = Complex::new(x.norm(), 0.0),
                ROOT => x = Complex::new(x.re.sqrt(), 0.0),
                ROOT_REAL => x = x.sqrt(),
                POW => {
                    let p = words[pos] as i32;
                    pos += 1;
                    x = x.powi(p);
                }
                ROUND => {
                    x.re = x.re.round();
                    x.im = x.im.round();
                }
                FLOOR => {
                    x.re = x.re.floor();
                    x.im = x.im.floor();
                }
                REAL => x = Complex::new(x.re, 0.0),
                IMAGINARY => x = Complex::new(x.im, 0.0),
                CONJUGATE => x = x.conj(),
                ISZERO => x = bool_to_c128(x.re == 0.0),
                GOTO => {
                    ip = words[pos] as usize;
                    pos = words[pos + 1] as usize;
                }
                BRANCH_IF => {
                    if x.re != 0.0 {
                        ip = words[pos] as usize;
                        pos = words[pos + 1] as usize;
                    } else {
                        pos += 2;
                    }
                }
                BRANCH_ELSE => {
                    if x.re == 0.0 {
                        ip = words[pos] as usize;
                        pos = words[pos + 1] as usize;
                    } else {
                        pos += 2;
                    }
                }
                JOIN => x = if x.re != 0.0 { z } else { y },
                GT => x = bool_to_c128(x.re > y.re),
                GEQ => x = bool_to_c128(x.re >= y.re),
                LT => x = bool_to_c128(x.re < y.re),
                LEQ => x = bool_to_c128(x.re <= y.re),
                EQ => x = bool_to_c128(x.re == y.re),
                NEQ => x = bool_to_c128(x.re != y.re),
                DUP => y = x,
                RET => break,
                _ => panic!("unrecognized unary op-code: {}", cmd),
            }
        }

        if cmd & STX != 0 {
            mem[words[pos] as usize] = x;
            pos += 1;
        }
    }
}

fn kernel_f64x4_complex(code: &[u8], words: &[u32], mem: &mut [Complex<f64x4>]) {
    unreachable!()
}

extern "C" {
    fn ker_complex(code: *const u8, words: *const u32, mem: *mut Complex<f64>) -> u64;
    fn ker_f64x4_complex(code: *const u8, words: *const u32, mem: *mut Complex<f64x4>) -> u64;
}

fn stub_x64_complex(code: &[u8], words: &[u32], mem: &mut [Complex<f64>]) {
    unsafe {
        let code: *const u8 = code.as_ptr();
        let words: *const u32 = words.as_ptr();
        let mem: *mut Complex<f64> = mem.as_mut_ptr();
        let ret = ker_complex(code, words, mem);

        if ret != 0 {
            panic!("asm complex kernel returns error: {:x}", ret);
        }
    }
}

fn stub_x64x4_complex(code: &[u8], words: &[u32], mem: &mut [Complex<f64x4>]) {
    unsafe {
        let code: *const u8 = code.as_ptr();
        let words: *const u32 = words.as_ptr();
        let mem: *mut Complex<f64x4> = mem.as_mut_ptr();
        let ret = ker_f64x4_complex(code, words, mem);

        if ret != 0 {
            panic!("asm simd complex kernel returns error: {:x}", ret);
        }
    }
}

impl Runner for GenericComplexRunner {
    fn evaluate(&mut self, args: &[f64], outs: &mut [f64]) {
        let first_param = self.count_consts;
        let count_params = self.count_params;
        self.mem[first_param..first_param + count_params].copy_from_slice(recast_as_c128(args));

        (self.ker)(&self.code, &self.words, &mut self.mem);

        let first_out = self.count_consts + self.count_params;
        let count_outs = self.count_outs;
        recast_as_c128_mut(outs).copy_from_slice(&self.mem[first_out..first_out + count_outs]);
    }

    fn evaluate_matrix(&mut self, args: &[f64], outs: &mut [f64], n: usize) {
        let first_param = self.count_consts;
        let first_out = self.count_consts + self.count_params;
        let count_params = self.count_params;
        let count_outs = self.count_outs;
        let l = self.num_lanes;

        let args = recast_as_c128(args);
        let outs = recast_as_c128_mut(outs);

        let mut m = 0;

        if let Some(ker_f64x4) = self.ker_f64x4 {
            let num_batches = n / l;
            m = num_batches * l;

            for t in 0..num_batches {
                Self::pre_transpose(
                    &args[t * l * count_params..(t + 1) * l * count_params],
                    &mut self.mem_f64x4[first_param..first_param + count_params],
                );

                ker_f64x4(&self.code, &self.words, &mut self.mem_f64x4);

                Self::post_transpose(
                    &self.mem_f64x4[first_out..first_out + count_outs],
                    &mut outs[t * l * count_outs..(t + 1) * l * count_outs],
                );
            }
        }

        for i in m..n {
            self.mem[first_param..first_param + count_params]
                .copy_from_slice(&args[i * count_params..(i + 1) * count_params]);

            (self.ker)(&self.code, &self.words, &mut self.mem);

            outs[i * count_outs..(i + 1) * count_outs]
                .copy_from_slice(&self.mem[first_out..first_out + count_outs]);
        }
    }

    fn add_constant(&mut self, z: Complex<f64>) {
        self.mem[self.next_const] = z;
        self.mem_f64x4[self.next_const] = Complex::new(f64x4::splat(z.re), f64x4::splat(z.im));
        self.next_const += 1;
    }
}
