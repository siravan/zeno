	.intel_syntax noprefix

	.equ LDX, 0x80
	.equ STX, 0x40
	.equ BINOP, 0x20
	.equ LDY, 0x10

    .text
	.global ker_f64x4_complex

.macro LOAD re, im
    mov eax, [rsi + 4 * r9]
    inc r9
    sal eax, 3
    vmovupd \re, [rdx + 8 * rax]
    vmovupd \im, [rdx + 8 * rax + 32]
.endm

.macro CMUL xd, yd, x1, y1, x2, y2
    /*
        T1 = ymm4
        T2 = ymm5
    */
    vmulpd ymm4, \y1, \y2
    vmulpd ymm5, \x1, \y2
    vfmsub231pd ymm4, \x1, \x2
    vfmadd231pd ymm5, \x2, \y1
    vmovapd \xd, ymm4
    vmovapd \yd, ymm5
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

ker_f64x4_complex:
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
    LOAD ymm0, ymm1
.L1:
    xor rax, rax
    mov al, cl
    and al, 0x3f
    lea	r10, .BT[rip]
    add	r10, QWORD PTR [r10 + 8 * rax]
	notrack jmp	r10

.A0:    /* LDY + MUL */
    LOAD ymm2, ymm3
.B0:    /* MUL */
    CMUL ymm0, ymm1, ymm0, ymm1, ymm2, ymm3
    jmp .Next

.A1:    /* LDY + ADD */
    LOAD ymm2, ymm3
.B1:    /* ADD */
    vaddpd ymm0, ymm0, ymm2
    vaddpd ymm1, ymm1, ymm3
    jmp .Next

.A2:    /* LDY + SUB */
    LOAD ymm2, ymm3
.B2:    /* SUB */
    vsubpd ymm0, ymm0, ymm2
    vsubpd ymm1, ymm1, ymm3
    jmp .Next

.A3:    /* LDY + DIV */
    LOAD ymm2, ymm3
.B3:    /* DIV */
    /*
        T0: ymm1
        T1: ymm4
        T2: ymm5
    */
    vmulpd ymm4, ymm1, ymm3
    vfmadd231pd ymm4, ymm0, ymm2
    vmulpd ymm5, ymm0, ymm3
    vfmsub231pd ymm5, ymm2, ymm1
    vmulpd ymm1, ymm2, ymm2
    vfmadd231pd ymm1, ymm3, ymm3
    vdivpd ymm0, ymm4, ymm1
    vdivpd ymm1, ymm5, ymm1
    jmp .Next

.A4:    /* LDY + POWF */
    LOAD ymm2, ymm3
.B4:    /* POWF */
    jmp .Error

.A5:    /* LDY + AND */
    LOAD ymm2, ymm3
.B5:    /* AND */
    vandpd xmm0, xmm0, xmm2
    vandpd xmm1, xmm1, xmm3
    jmp .Next

.A6:    /* LDY + OR */
    LOAD ymm2, ymm3
.B6:    /* OR */
    vorpd xmm0, xmm0, xmm2
    vorpd xmm1, xmm1, xmm3
    jmp .Next

.A7:    /* LDY + XOR */
    LOAD ymm2, ymm3
.B7:    /* XOR */
    vxorpd xmm0, xmm0, xmm2
    vxorpd xmm1, xmm1, xmm3
    jmp .Next

.A8:    /* LDY + COMPLEX */
    LOAD ymm2, ymm3
.B8:    /* COMPLEX */
    jmp .Next

.A9:    /* LDY + MOVZ */
    LOAD ymm2, ymm3
.B9:    /* MOVZ */
    vmovapd xmm6, xmm0
    vmovapd xmm7, xmm1
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
    NEG_ZERO ymm4
    vxorpd ymm0, ymm4, ymm0
    vxorpd ymm1, ymm4, ymm1
    jmp .Next

.U2:    /* NOT */
    ALL_ONES ymm4
    vxorpd ymm0, ymm4, ymm0
    vxorpd ymm1, ymm4, ymm1
    jmp .Next

.U3:    /* RECIP */
    vmulpd ymm4, ymm0, ymm0
    vfmadd231pd ymm4, ymm1, ymm1
    vdivpd ymm0, ymm0, ymm4
    vdivpd ymm1, ymm1, ymm4
    NEG_ZERO ymm4
    vxorpd xmm1, xmm4, xmm1
    jmp .Next

.U4:    /* ABS */
    NEG_ZERO ymm4
    vandnpd ymm0, ymm4, ymm0
    vandnpd ymm1, ymm4, ymm1
    jmp .Next

.U5:    /* ROOT */
    vxorpd ymm4, ymm4, ymm4
    vcmpltpd ymm8, ymm4, ymm0

    vmulpd ymm4, ymm0, ymm0
    vfmadd231pd ymm4, ymm1, ymm1
    vsqrtpd ymm4, ymm4

    NEG_ZERO ymm9
    vandnpd ymm9, ymm9, ymm0
    vaddpd ymm4, ymm4, ymm9

    TWO ymm9
    vdivpd ymm4, ymm4, ymm9
    vsqrtpd ymm4, ymm4
    vdivpd ymm5, ymm1, ymm4
    vdivpd ymm5, ymm5, ymm9
    vcmpeqpd ymm1, ymm5, ymm5
    vandpd ymm5, ymm5, ymm1

    vmovapd ymm9, ymm8
    vandpd ymm0, ymm8, ymm4
    vandnpd ymm8, ymm8, ymm5
    vorpd ymm0, ymm0, ymm8

    vandpd ymm1, ymm9, ymm5
    vandnpd ymm9, ymm9, ymm4
    vorpd ymm1, ymm1, ymm9

    jmp .Next

.U6:    /* ROOT_REAL */
    vsqrtpd ymm0, ymm0
    vxorpd ymm1, ymm1, ymm1
    jmp .Next

.U7:    /* ROUND */
    vroundpd ymm0, ymm0, 0
    vroundpd ymm1, ymm1, 0
    jmp .Next

.U8:    /* FLOOR */
    vroundpd ymm0, ymm0, 1
    vroundpd ymm1, ymm1, 1
    jmp .Next

.U9:    /* REAL */
    vroundpd ymm1, ymm1, 1
    jmp .Next

.U10:   /* IMAGINARY */
    vmovapd ymm0, ymm1
    vxorpd ymm1, ymm1, ymm1
    jmp .Next

.U11:   /* CONJUGATE */
    NEG_ZERO ymm4
    vxorpd ymm1, ymm4, ymm1
    jmp .Next

.U12:   /* ISZERO */
    vxorpd ymm4, ymm4, ymm4
    vcmpeqpd ymm0, ymm4, ymm0
    vmovapd ymm1, ymm0
    jmp .Next

.U13:   /* POW */
    mov eax, [rsi + 4 * r9]
    inc r9
    vmovapd ymm8, ymm0
    vmovapd ymm9, ymm1
    vbroadcastsd ymm0, [r11 + 16]
    vxorpd ymm1, ymm1, ymm1
    xor r10b, r10b
    or eax, eax
    jz .Next
    jns .P1
    inc r10b
    neg eax
.P1:
    test al, 1
    jz .P2
    CMUL ymm0, ymm1, ymm0, ymm1, ymm8, ymm9
.P2:
    sar eax, 1
    jz .P3
    CMUL ymm8, ymm9, ymm8, ymm9, ymm8, ymm9
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
    vxorpd ymm4, ymm4, ymm4
    vcmpeqsd xmm4, xmm4, xmm0
    vmovmskpd rax, ymm4
    cmp al, 0
    jz .U14
    add r9, 2
    jmp .Loop

.U16:   /* BRANCH_ELSE */
    vxorpd ymm4, ymm4, ymm4
    vcmpeqpd ymm4, ymm4, ymm0
    vmovmskpd rax, ymm4
    cmp al, 15
    jz .U14
    add r9, 2
    jmp .Loop

.U17:   /* JOIN */
    vxorpd ymm4, ymm4, ymm4
    vcmpeqpd ymm4, ymm4, ymm0
    vandpd ymm0, ymm4, ymm2
    vandnpd ymm4, ymm4, ymm6
    vorpd ymm0, ymm0, ymm4

    vxorpd ymm5, ymm5, ymm5
    vcmpeqpd ymm5, ymm5, ymm1
    vandpd ymm1, ymm5, ymm3
    vandnpd ymm5, ymm5, ymm7
    vorpd ymm1, ymm1, ymm5

    jmp .Next

.U18:   /* GT */
    vcmpgtpd ymm0, ymm0, ymm2
    vmovapd ymm1, ymm0
    jmp .Next

.U19:   /* GEQ */
    vcmpeqpd ymm0, ymm0, ymm2
    vmovapd ymm1, ymm0
    jmp .Next

.U20:   /* LT */
    vcmpltpd ymm0, ymm0, ymm2
    vmovapd ymm1, ymm0
    jmp .Next

.U21:   /* LEQ */
    vcmpltpd ymm0, ymm0, ymm2
    vmovapd ymm1, ymm0
    jmp .Next

.U22:   /* EQ */
    vcmpeqpd ymm0, ymm0, ymm2
    vmovapd ymm1, ymm0
    jmp .Next

.U23:   /* NEQ */
    vcmpeqpd ymm0, ymm0, ymm2
    vmovapd ymm1, ymm0
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
    sal eax, 3
    vmovupd [rdx + 8 * rax], ymm0
    vmovupd [rdx + 8 * rax + 32], ymm1
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
