use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::sync::Arc;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum CompilerType {
    /// generates bytecode (interpreter).
    ByteCode,
    /// generates code for the detected CPU (default)
    Native,
    /// generates x86-64 (AMD64) code.
    Amd,
    /// generates AVX code for x86-64 architecture.
    AmdAVX,
    /// generates SSE2 code for x86-64 architecture.
    AmdSSE,
    /// generates aarch64 (ARM64) code.
    Arm,
    /// generates riscv64 (RISC V) code.
    RiscV,
    /// debug mode, generates both bytecode and native codes
    /// and compares the outputs.
    Debug,
}

pub const USE_SIMD: u32 = 0x00000001;
pub const USE_THREADS: u32 = 0x00000002;
pub const CSE: u32 = 0x00000004;
pub const FASTMATH: u32 = 0x00000008;

pub const COMPLEX: u32 = 0x00000020;
pub const SYMBOLICA: u32 = 0x00000040;
pub const SIMD_BRANCH: u32 = 0x00000080;

pub const COMPACT: u32 = 0x00001000;
pub const COMPRESS: u32 = 0x00002000;
pub const DIRECT: u32 = 0x00004000;
pub const FAST_COMPLEX: u32 = 0x00008000;

pub const DEBUG_BYTECODE: u32 = 0x000010000;
pub const DEBUG_SCALAR: u32 = 0x000020000;
pub const DEBUG_SIMD: u32 = 0x000040000;
pub const DEBUG_STATS: u32 = 0x000080000;

pub const HUGE: u32 = 0x00100000;
pub const PARALLEL_MUL: u32 = 0x00200000;

pub const OPT_LEVEL_MASK: u32 = 0x00000f00;
pub const OPT_LEVEL_SHIFT: usize = 8;

pub const SPILL_AREA: usize = 16;
pub const SLICE_CAP: usize = 64;

#[derive(Debug, Clone)]
pub struct Config {
    pub opt: u32,
    pub ty: CompilerType,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
struct ConfigToml {
    ty: String,
    options: Options,
    debug: DebugOptions,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
struct Options {
    use_simd: bool,
    use_threads: bool,
    cse: bool,
    fastmath: bool,
    complex: bool,
    symbolica: bool,
    simd_branch: bool,
    compact: bool,
    compress: bool,
    direct: bool,
    fast_complex: bool,
    huge: bool,
    parallel_mul: bool,
    opt_level: u8,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
struct DebugOptions {
    bytecode: bool,
    scalar: bool,
    simd: bool,
    stats: bool,
}

impl Config {
    const MAGIC: usize = 0x802c3c77c7422e70;

    pub fn new(ty: CompilerType, opt: u32) -> Result<Config> {
        Ok(Config { opt, ty })
    }

    pub fn from_name(ty: &str, opt: u32) -> Result<Config> {
        let ty = match ty {
            "bytecode" => CompilerType::ByteCode,
            "arm" => CompilerType::Arm,
            "riscv" => CompilerType::RiscV,
            "amd" => CompilerType::Amd,
            "amd-avx" => CompilerType::AmdAVX,
            "amd-sse" => CompilerType::AmdSSE,
            "native" => CompilerType::Native,
            "debug" => CompilerType::Debug,
            _ => {
                if ty.ends_with(".toml") {
                    return Self::from_toml(ty, opt);
                } else {
                    return Err(anyhow!("invalid ty"));
                }
            }
        };
        Self::new(ty, opt)
    }

    pub fn from_toml(path: &str, mut opt: u32) -> Result<Config> {
        let toml = std::fs::read_to_string(path)?;
        let c: ConfigToml = toml::from_str(&toml)?;

        opt &= COMPLEX | SYMBOLICA;
        let mut config = Self::from_name(&c.ty, opt)?;

        config.set_simd(c.options.use_simd);
        config.set_threads(c.options.use_threads);
        config.set_cse(c.options.cse);
        config.set_fastmath(c.options.fastmath);
        config.set_complex(c.options.complex | config.is_complex());
        config.set_symbolica(c.options.symbolica | config.symbolica());
        config.set_simd_branch(c.options.simd_branch);
        config.set_compact(c.options.compact);
        config.set_compress(c.options.compress);
        config.set_dicect(c.options.direct);
        config.set_fast_complex(c.options.fast_complex);
        config.set_huge(c.options.huge);
        config.set_parallel_mul(c.options.parallel_mul);

        config.set_opt_level(c.options.opt_level);

        config.set_debug_bytecode(c.debug.bytecode);
        config.set_debug_scalar(c.debug.scalar);
        config.set_debug_simd(c.debug.simd);
        config.set_debug_stats(c.debug.stats);

        Ok(config)
    }

    pub fn to_toml(&self, path: &str) {
        let ty = match self.ty {
            CompilerType::ByteCode => "bytecode",
            CompilerType::Arm => "arm",
            CompilerType::RiscV => "riscv",
            CompilerType::Amd => "amd",
            CompilerType::AmdAVX => "amd-avx",
            CompilerType::AmdSSE => "amd-sse",
            CompilerType::Native => "native",
            CompilerType::Debug => "debug",
        }
        .into();

        let options: Options = Options {
            use_simd: self.use_simd(),
            use_threads: self.use_threads(),
            cse: self.cse(),
            fastmath: self.fastmath(),
            complex: self.is_complex(),
            symbolica: self.symbolica(),
            simd_branch: self.simd_branch(),
            compact: self.compact(),
            compress: self.compress(),
            direct: self.direct(),
            fast_complex: self.fast_complex(),
            opt_level: self.opt_level(),
            huge: self.huge(),
            parallel_mul: self.parallel_mul(),
        };

        let debug: DebugOptions = DebugOptions {
            bytecode: self.debug_bytedode(),
            scalar: self.debug_scalar(),
            simd: self.debug_simd(),
            stats: self.debug_stats(),
        };

        let c: ConfigToml = ConfigToml { ty, options, debug };
        let toml = toml::to_string(&c).unwrap();
        let _ = std::fs::write(path, toml);
    }

    fn test(&self, mask: u32) -> bool {
        self.opt & mask != 0
    }

    pub fn cross_compiled(&self) -> bool {
        (self.is_amd64() && !cfg!(target_arch = "x86_64"))
            || (self.is_arm64() && !cfg!(target_arch = "aarch64"))
            || (self.is_riscv64() && !cfg!(target_arch = "riscv64"))
    }

    pub fn is_amd64(&self) -> bool {
        (matches!(self.ty, CompilerType::Native) && cfg!(target_arch = "x86_64"))
            || matches!(self.ty, CompilerType::Amd)
            || matches!(self.ty, CompilerType::AmdSSE)
            || matches!(self.ty, CompilerType::AmdAVX)
    }

    pub fn is_arm64(&self) -> bool {
        (matches!(self.ty, CompilerType::Native) && cfg!(target_arch = "aarch64"))
            || matches!(self.ty, CompilerType::Arm)
    }

    pub fn is_riscv64(&self) -> bool {
        (matches!(self.ty, CompilerType::Native) && cfg!(target_arch = "riscv64"))
            || matches!(self.ty, CompilerType::RiscV)
    }

    fn cpu_has_avx() -> bool {
        #[cfg(target_arch = "x86_64")]
        return is_x86_feature_detected!("avx");
        #[cfg(not(target_arch = "x86_64"))]
        return false;
    }

    pub fn has_avx(&self) -> bool {
        self.is_amd64() && !matches!(self.ty, CompilerType::AmdSSE) && Self::cpu_has_avx()
    }

    pub fn is_sse(&self) -> bool {
        self.is_amd64() && !self.has_avx()
    }

    pub fn is_bytecode(&self) -> bool {
        matches!(self.ty, CompilerType::ByteCode)
    }

    pub fn is_debug(&self) -> bool {
        matches!(self.ty, CompilerType::Debug)
    }

    pub fn may_fast(&self) -> bool {
        self.is_amd64() || self.is_arm64() || self.is_riscv64()
    }

    pub fn cse(&self) -> bool {
        self.test(CSE)
    }

    pub fn use_simd(&self) -> bool {
        self.test(USE_SIMD) && (self.has_avx() || self.is_arm64())
    }

    pub fn simd_branch(&self) -> bool {
        self.test(SIMD_BRANCH) && (self.has_avx() || self.is_arm64())
    }

    pub fn use_threads(&self) -> bool {
        self.test(USE_THREADS)
    }

    pub fn fastmath(&self) -> bool {
        self.test(FASTMATH) && (self.has_avx() || self.is_arm64() || self.is_riscv64())
    }

    pub fn compact(&self) -> bool {
        self.test(COMPACT)
    }

    pub fn compress(&self) -> bool {
        self.test(COMPRESS)
    }

    pub fn direct(&self) -> bool {
        self.test(DIRECT)
    }

    pub fn huge(&self) -> bool {
        self.test(HUGE)
    }

    pub fn parallel_mul(&self) -> bool {
        self.test(PARALLEL_MUL)
    }

    pub fn debug_bytedode(&self) -> bool {
        self.test(DEBUG_BYTECODE)
    }

    pub fn debug_scalar(&self) -> bool {
        self.test(DEBUG_SCALAR)
    }

    pub fn debug_simd(&self) -> bool {
        self.test(DEBUG_SIMD)
    }

    pub fn debug_stats(&self) -> bool {
        self.test(DEBUG_STATS)
    }

    pub fn opt_level(&self) -> u8 {
        let level = ((self.opt & OPT_LEVEL_MASK) >> OPT_LEVEL_SHIFT) as u8;

        if self.is_sse() {
            level.min(2)
        } else {
            level
        }
    }

    pub fn compiler_type(&self) -> CompilerType {
        if self.has_avx() {
            CompilerType::AmdAVX
        } else if self.is_amd64() {
            CompilerType::AmdSSE
        } else if self.is_arm64() {
            CompilerType::Arm
        } else if self.is_riscv64() {
            CompilerType::RiscV
        } else if self.is_bytecode() {
            CompilerType::ByteCode
        } else if self.is_debug() {
            CompilerType::Debug
        } else {
            unreachable!()
        }
    }

    pub fn native_compiler_type(&self) -> CompilerType {
        let config = Config::new(CompilerType::Native, self.opt).unwrap();
        config.compiler_type()
    }

    pub fn available_registers(&self) -> u8 {
        16
    }

    pub fn count_scratch(&self) -> u8 {
        if !self.is_complex() {
            self.available_registers() - 2
        /*
            } else if self.fast_complex() {
            self.available_registers() - 4
        */
        } else {
            (self.available_registers() - 6) / 2
        }
    }

    pub fn symbolica(&self) -> bool {
        self.test(SYMBOLICA)
    }

    pub fn is_complex(&self) -> bool {
        self.test(COMPLEX)
    }

    pub fn fast_complex(&self) -> bool {
        self.test(FAST_COMPLEX) && (self.has_avx() || self.is_arm64())
    }

    /// Sets of optimization level. The valid values are 0, 1, 2, which roughly correspond to gcc O0, O1, and O2 levels.
    pub fn set_opt_level(&mut self, opt_level: u8) {
        self.opt = (self.opt & !OPT_LEVEL_MASK) | ((opt_level as u32) << OPT_LEVEL_SHIFT);
    }

    /// Enables Common-Subexpression-Elimination.
    pub fn set_cse(&mut self, enabled: bool) {
        self.opt = (self.opt & !CSE) | if enabled { CSE } else { 0 };
    }

    /// Enables fastmath mode. The main effect is to generate fused-multiply-addition
    /// instructions if possible.
    pub fn set_fastmath(&mut self, enabled: bool) {
        self.opt = (self.opt & !FASTMATH) | if enabled { FASTMATH } else { 0 };
    }

    /// Enables SIMD mode.
    pub fn set_simd(&mut self, enabled: bool) {
        self.opt = (self.opt & !USE_SIMD) | if enabled { USE_SIMD } else { 0 };
    }

    /// Enables forced SIMD branching mode.
    pub fn set_simd_branch(&mut self, enabled: bool) {
        self.opt = (self.opt & !SIMD_BRANCH) | if enabled { SIMD_BRANCH } else { 0 };
    }

    /// Enables Complex Numbers.
    pub fn set_complex(&mut self, enabled: bool) {
        self.opt = (self.opt & !COMPLEX) | if enabled { COMPLEX } else { 0 };
    }

    /// Enables Fast Complex (using SIMD instructions in the scalar code).
    pub fn set_fast_complex(&mut self, enabled: bool) {
        self.opt = (self.opt & !FAST_COMPLEX) | if enabled { FAST_COMPLEX } else { 0 };
    }

    /// Enables Multi-threading.
    pub fn set_threads(&mut self, enabled: bool) {
        self.opt = (self.opt & !USE_THREADS) | if enabled { USE_THREADS } else { 0 };
    }

    /// Enables Symbolica Mode.
    pub fn set_symbolica(&mut self, enabled: bool) {
        self.opt = (self.opt & !SYMBOLICA) | if enabled { SYMBOLICA } else { 0 };
    }

    /// Compact stack frame.
    pub fn set_compact(&mut self, enabled: bool) {
        self.opt = (self.opt & !COMPACT) | if enabled { COMPACT } else { 0 };
    }

    /// Memory-saver mode for very large inputs.
    pub fn set_compress(&mut self, enabled: bool) {
        self.opt = (self.opt & !COMPRESS) | if enabled { COMPRESS } else { 0 };
    }

    /// Direct translation from Symbolica IR to Symjit IR.
    pub fn set_dicect(&mut self, enabled: bool) {
        self.opt = (self.opt & !DIRECT) | if enabled { DIRECT } else { 0 };
    }

    /// Huge paged to reduce TLB pressure.
    pub fn set_huge(&mut self, enabled: bool) {
        self.opt = (self.opt & !HUGE) | if enabled { HUGE } else { 0 };
    }

    /// Merge serial complex multiplications into parallel operation.
    pub fn set_parallel_mul(&mut self, enabled: bool) {
        self.opt = (self.opt & !PARALLEL_MUL) | if enabled { PARALLEL_MUL } else { 0 };
    }

    /// Dump bytecode for debugging
    pub fn set_debug_bytecode(&mut self, enabled: bool) {
        self.opt = (self.opt & !DEBUG_BYTECODE) | if enabled { DEBUG_BYTECODE } else { 0 };
    }

    /// Dump scalar binary for debugging
    pub fn set_debug_scalar(&mut self, enabled: bool) {
        self.opt = (self.opt & !DEBUG_SCALAR) | if enabled { DEBUG_SCALAR } else { 0 };
    }

    /// Dump simd binary for debugging
    pub fn set_debug_simd(&mut self, enabled: bool) {
        self.opt = (self.opt & !DEBUG_SIMD) | if enabled { DEBUG_SIMD } else { 0 };
    }

    /// Print stats for debugging
    pub fn set_debug_stats(&mut self, enabled: bool) {
        self.opt = (self.opt & !DEBUG_STATS) | if enabled { DEBUG_STATS } else { 0 };
    }
}

impl Default for Config {
    fn default() -> Config {
        if std::fs::exists("symjit.toml").unwrap() {
            Self::from_toml("symjit.toml", 0).unwrap()
        } else {
            Config::new(
                CompilerType::Native,
                USE_SIMD
                    | SYMBOLICA
                    | COMPACT
                    | FASTMATH
                    | FAST_COMPLEX
                    | DIRECT
                    | PARALLEL_MUL
                    | (2 << OPT_LEVEL_SHIFT),
            )
            .unwrap()
            // config.to_toml("symjit.toml");
        }
    }
}

// the list of intrinsic unary ops, i.e., operations that can be implemented directly in
// machine code
const UNARY: &[&str] = &[
    "abs",
    "not",
    "neg",
    "root",
    "real_root",
    "square",
    "cube",
    "recip",
    "round",
    "floor",
    "ceiling",
    "trunc",
    "frac",
    "_powi_",
    "_call_",
    "real",
    "imaginary",
    "conjugate",
    "iszero",
    "isnotzero",
];

// the list of intrinsic binary ops, i.e., operations that can be implemented directly in
// machine code
const BINARY: &[&str] = &[
    "plus",
    "minus",
    "times",
    "divide",
    "rem",
    "gt",
    "geq",
    "lt",
    "leq",
    "eq",
    "neq",
    "and",
    "or",
    "xor",
    "_ifelse_",
    "_powi_mod_",
    "_call_",
    "min",
    "max",
    "heaviside",
    "complex",
];

impl Config {
    pub fn is_intrinsic_unary(&self, op: &str) -> bool {
        UNARY.contains(&op)
    }

    pub fn is_intrinsic_binary(&self, op: &str) -> bool {
        BINARY.contains(&op)
    }
}
