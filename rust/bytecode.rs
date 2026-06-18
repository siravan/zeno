pub const LDX: u8 = 0x80;
pub const STX: u8 = 0x40;
pub const BINOP: u8 = 0x20;
pub const LDY: u8 = 0x10;

// BinOp operations
pub const MUL: u8 = BINOP;
pub const ADD: u8 = 1 | BINOP;
pub const SUB: u8 = 2 | BINOP;
pub const DIV: u8 = 3 | BINOP;
pub const POWF: u8 = 4 | BINOP;
pub const AND: u8 = 5 | BINOP;
pub const OR: u8 = 6 | BINOP;
pub const XOR: u8 = 7 | BINOP;
pub const COMPLEX: u8 = 8 | BINOP;
pub const MOVZ: u8 = 9 | BINOP;

// UniOp operations
pub const ASSIGN: u8 = 0; // also NOP
pub const NEG: u8 = 1;
pub const NOT: u8 = 2;
pub const RECIP: u8 = 3;
pub const ABS: u8 = 4;
pub const ROOT: u8 = 5;
pub const ROOT_REAL: u8 = 6;
pub const ROUND: u8 = 7;
pub const FLOOR: u8 = 8;
pub const REAL: u8 = 9;
pub const IMAGINARY: u8 = 10;
pub const CONJUGATE: u8 = 11;
pub const ISZERO: u8 = 12;
pub const POW: u8 = 13;
pub const GOTO: u8 = 14;
pub const BRANCH_IF: u8 = 15;
pub const BRANCH_ELSE: u8 = 16;
pub const JOIN: u8 = 17;
pub const GT: u8 = 18;
pub const GEQ: u8 = 19;
pub const LT: u8 = 20;
pub const LEQ: u8 = 21;
pub const EQ: u8 = 22;
pub const NEQ: u8 = 23;

pub const DUP: u8 = 30;
pub const RET: u8 = 31;
