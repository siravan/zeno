	.intel_syntax noprefix

	.equ LDX, 0x80
	.equ STX, 0x40
	.equ BINOP, 0x20
	.equ LDY, 0x10

    .text
	.global ker_complex

.macro LOAD reg
    mov eax, [rsi + 4 * r9]
    inc r9
    sal eax, 1
    vmovupd \reg, [rdx + 8 * rax]
.endm

.macro CMUL dst, s1, s2
    /*
        T1 = xmm4
        T2 = xmm5
    */
    vunpcklpd xmm4, \s1, \s1    # duplicate real
    vunpckhpd xmm5, \s1, \s1    # duplicate imag
    vmulpd xmm4, xmm4, \s2
    vmulpd xmm5, xmm5, \s2
    vshufpd xmm5, xmm5, xmm5, 1
    vaddsubpd \dst, xmm4, xmm5
.endm

.macro NEG_ZERO dst
    vbroadcastsd \dst, [r11]
.endm`

.macro ALL_ONES dst
    vbroadcastsd \dst, [r11 + 8]
.endm

.macro ZERO dst
    vxorpd \dst, \dst, \dst
.endm

.macro ONE dst
    vbroadcastsd \dst, [r11 + 16]
.endm

.macro TWO dst
    vbroadcastsd \dst, [r11 + 24]
.endm

ker_complex:
    ; code = rdi
    ; words = rsi
    ; mem = rdx
    xor r8, r8  ; ip = r8
    xor r9, r9  ; pos = r9
    lea	r11, .CONSTANTS[rip]

.Loop:
    mov cl, [rdi + r8]
    inc r8

    test cl, LDX
    jz .L1
    LOAD xmm0
.L1:
    xor rax, rax
    mov al, cl
    and al, 0x3f
    lea	r10, .BT[rip]
    add	r10, QWORD PTR [r10 + 8 * rax]
	notrack jmp	r10

.A0:    /* LDY + MUL */
    LOAD xmm1
.B0:    /* MUL */
    CMUL xmm0, xmm0, xmm1
    jmp .Next

.A1:    /* LDY + ADD */
    LOAD xmm1
.B1:    /* ADD */
    vaddpd xmm0, xmm0, xmm1
    jmp .Next

.A2:    /* LDY + SUB */
    LOAD xmm1
.B2:    /* SUB */
    vsubpd xmm0, xmm0, xmm1
    jmp .Next

.A3:    /* LDY + DIV */
    LOAD xmm1
.B3:    /* DIV */
    vmulpd xmm3, xmm1, xmm1
    vhaddpd xmm3, xmm3, xmm3
    vunpcklpd xmm4, xmm0, xmm0
    vunpckhpd xmm5, xmm0, xmm0
    vmulpd xmm4, xmm4, xmm1
    vmulpd xmm5, xmm5, xmm1
    vshufpd xmm4, xmm4, xmm4, 1
    vaddsubpd xmm0, xmm5, xmm4
    vshufpd xmm0, xmm0, xmm0, 1
    vdivpd xmm0, xmm0, xmm3
    jmp .Next

.A4:    /* LDY + POWF */
    LOAD xmm1
.B4:    /* POWF */
    jmp .Error

.A5:    /* LDY + AND */
    LOAD xmm1
.B5:    /* AND */
    vandpd xmm0, xmm0, xmm1
    jmp .Next

.A6:    /* LDY + OR */
    LOAD xmm1
.B6:    /* OR */
    vorpd xmm0, xmm0, xmm1
    jmp .Next

.A7:    /* LDY + XOR */
    LOAD xmm1
.B7:    /* XOR */
    vxorpd xmm0, xmm0, xmm1
    jmp .Next

.A8:    /* LDY + COMPLEX */
    LOAD xmm1
.B8:    /* COMPLEX */
    jmp .Next

.A9:    /* LDY + MOVZ */
    LOAD xmm1
.B9:    /* MOVZ */
    vmovapd xmm2, xmm0
    jmp .Next

.A10:
.B10:

.A11:
.B11:

.A12:
.B12:

.A13:
.B13:

.A14:
.B14:

.A15:
.B15:
    jmp .Error

.U0:    /* ASSIGN */
    jmp .Next

.U1:    /* NEG */
    NEG_ZERO ymm3
    vxorpd xmm0, xmm3, xmm0
    jmp .Next

.U2:    /* NOT */
    ALL_ONES ymm3
    vxorpd xmm0, xmm3, xmm0
    jmp .Next

.U3:    /* RECIP */
    vshufpd xmm4, xmm0, xmm0, 1
    vxorpd xmm5, xmm5, xmm5
    vaddsubpd xmm4, xmm5, xmm4
    vshufpd xmm5, xmm4, xmm4, 1
    vmulpd xmm4, xmm0, xmm0
    vhaddpd xmm4, xmm4, xmm4
    vdivpd xmm0, xmm5, xmm4
    jmp .Next

.U4:    /* ABS */
    vmulpd xmm4, xmm0, xmm0
    vhaddpd xmm4, xmm4, xmm4
    vsqrtsd xmm5, xmm4, xmm4
    vxorpd xmm4, xmm4, xmm4
    vunpcklpd xmm0, xmm5, xmm4
    jmp .Next

.U5:    /* ROOT */
    vmovq rax, xmm0
    vmulpd xmm4, xmm0, xmm0
    vhaddpd xmm4, xmm4, xmm4
    vsqrtsd xmm4, xmm4, xmm4
    vmovsd xmm3, [r11]  # NEG_ZERO
    vandnpd xmm5, xmm3, xmm0
    vaddsd xmm4, xmm4, xmm5
    vmovsd xmm3, [r11 + 24]  # TWO
    vdivsd xmm4, xmm4, xmm3
    vsqrtsd xmm4, xmm4, xmm4

    vunpckhpd xmm5, xmm0, xmm0
    vdivsd xmm5, xmm5, xmm4
    vdivsd xmm5, xmm5, xmm3

    vcmpeqsd xmm3, xmm5, xmm5
    vandpd xmm5, xmm5, xmm3

    vunpcklpd xmm0, xmm5, xmm4
    or rax, rax
    js .Next
    vshufpd xmm0, xmm0, xmm0, 1
    jmp .Next

.U6:    /* ROOT_REAL */
    vxorpd  xmm3, xmm3, xmm3
    vsqrtsd xmm0, xmm0, xmm0
    vunpcklpd xmm0, xmm0, xmm3
    jmp .Next

.U7:    /* ROUND */
    vroundpd xmm0, xmm0, 0
    jmp .Next

.U8:    /* FLOOR */
    vroundpd xmm0, xmm0, 1
    jmp .Next

.U9:    /* REAL */
    vxorpd xmm3, xmm3, xmm3
    vunpcklpd xmm0, xmm0, xmm3
    jmp .Next

.U10:   /* IMAGINARY */
    vxorpd xmm3, xmm3, xmm3
    vunpckhpd xmm0, xmm0, xmm3
    jmp .Next

.U11:   /* CONJUGATE */
    vxorpd xmm3, xmm3, xmm3
    vshufpd xmm0, xmm0, xmm0, 1
    vaddsubpd xmm0, xmm3, xmm0
    vshufpd xmm0, xmm0, xmm0, 1
    jmp .Next

.U12:   /* ISZERO */
    vxorpd xmm3, xmm3, xmm3
    vcmpeqsd xmm0, xmm3, xmm0
    vunpcklpd xmm0, xmm0, xmm0
    jmp .Next

.U13:   /* POW */
    mov eax, [rsi + 4 * r9]
    inc r9
    vmovapd xmm3, xmm0
    vxorpd xmm0, xmm0, xmm0
    vmovsd xmm0, [r11 + 16]
    xor r10b, r10b
    or eax, eax
    jz .Next
    jns .P1
    inc r10b
    neg eax
.P1:
    test al, 1
    jz .P2
    CMUL xmm0, xmm0, xmm3
.P2:
    sar eax, 1
    jz .P3
    CMUL xmm3, xmm3, xmm3
    jmp .P1
.P3:
    test r10b, 1
    jnz .U3
    jmp .Next

.U14:   /* GOTO */
    mov r8d, [rsi + 4 * r9]
    mov r9d, [rsi + 4 * r9 + 4]
    jmp .Loop

.U15:   /* BRANCH_IF */
    vxorpd xmm3, xmm3, xmm3
    vcmpeqsd xmm3, xmm3, xmm0
    vmovmskpd rax, xmm3
    test al, 1
    jz .U14
    add r9, 2
    jmp .Loop

.U16:   /* BRANCH_ELSE */
    vxorpd xmm3, xmm3, xmm3
    vcmpeqsd xmm3, xmm3, xmm0
    vmovmskpd rax, xmm3
    test al, 1
    jnz .U14
    add r9, 2
    jmp .Loop

.U17:   /* JOIN */
    vxorpd xmm3, xmm3, xmm3
    vcmpeqsd xmm3, xmm3, xmm0
    vandpd xmm0, xmm3, xmm1
    vandnpd xmm3, xmm3, xmm2
    vorpd xmm0, xmm0, xmm3
    jmp .Next

.U18:   /* GT */
    vcmpgtsd xmm0, xmm0, xmm1
    vunpcklpd xmm0, xmm0, xmm0
    jmp .Next

.U19:   /* GEQ */
    vcmpeqsd xmm0, xmm0, xmm1
    vunpcklpd xmm0, xmm0, xmm0
    jmp .Next

.U20:   /* LT */
    vcmpltsd xmm0, xmm0, xmm1
    vunpcklpd xmm0, xmm0, xmm0
    jmp .Next

.U21:   /* LEQ */
    vcmpltsd xmm0, xmm0, xmm1
    vunpcklpd xmm0, xmm0, xmm0
    jmp .Next

.U22:   /* EQ */
    vcmpeqsd xmm0, xmm0, xmm1
    vunpcklpd xmm0, xmm0, xmm0
    jmp .Next

.U23:   /* NEQ */
    vcmpeqsd xmm0, xmm0, xmm1
    vunpcklpd xmm0, xmm0, xmm0
    jmp .U2 /* NOT */
.U24:
.U25:
.U26:
.U27:
.U28:
.U29:
    jmp .Error
.U30:   /* DUP */
    vmovapd xmm1, xmm0
    jmp .Next
.U31:
    xor rax, rax    # return OK
    jmp .End

.Next:
    test cl, STX
    jz .Loop
    mov eax, [rsi + 4 * r9]
    inc r9
    sal eax, 1
    vmovupd [rdx + 8 * rax], xmm0
    jmp .Loop

.Error:
    mov rax, 256
    mov al, cl
.End:
    vzeroupper
    ret

    .section	.rodata
	.align 8
.CONSTANTS:
    .quad   0x8000000000000000  # -0.0
    .quad   0xffffffffffffffff  # NaN
    .quad   0x3ff0000000000000  # 1.0
    .quad   0x4000000000000000  # 2.0

.BT:
	.quad	.U0-.BT
	.quad	.U1-.BT
	.quad	.U2-.BT
	.quad	.U3-.BT
	.quad	.U4-.BT
	.quad	.U5-.BT
	.quad	.U6-.BT
	.quad	.U7-.BT
	.quad	.U8-.BT
	.quad	.U9-.BT
	.quad	.U10-.BT
	.quad	.U11-.BT
	.quad	.U12-.BT
	.quad	.U13-.BT
	.quad	.U14-.BT
	.quad	.U15-.BT
	.quad	.U16-.BT
	.quad	.U17-.BT
	.quad	.U18-.BT
	.quad	.U19-.BT
	.quad	.U20-.BT
	.quad	.U21-.BT
	.quad	.U22-.BT
	.quad	.U23-.BT
	.quad	.U24-.BT
	.quad	.U25-.BT
	.quad	.U26-.BT
	.quad	.U27-.BT
	.quad	.U28-.BT
	.quad	.U29-.BT
	.quad	.U30-.BT
	.quad	.U31-.BT
	.quad	.B0-.BT
	.quad	.B1-.BT
	.quad	.B2-.BT
	.quad	.B3-.BT
	.quad	.B4-.BT
	.quad	.B5-.BT
	.quad	.B6-.BT
	.quad	.B7-.BT
	.quad	.B8-.BT
	.quad	.B9-.BT
	.quad	.B10-.BT
	.quad	.B11-.BT
	.quad	.B12-.BT
	.quad	.B13-.BT
	.quad	.B14-.BT
	.quad	.B15-.BT
	.quad	.A0-.BT
	.quad	.A1-.BT
	.quad	.A2-.BT
	.quad	.A3-.BT
	.quad	.A4-.BT
	.quad	.A5-.BT
	.quad	.A6-.BT
	.quad	.A7-.BT
	.quad	.A8-.BT
	.quad	.A9-.BT
	.quad	.A10-.BT
	.quad	.A11-.BT
	.quad	.A12-.BT
	.quad	.A13-.BT
	.quad	.A14-.BT
	.quad	.A15-.BT
