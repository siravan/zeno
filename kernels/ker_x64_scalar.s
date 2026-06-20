	.intel_syntax noprefix

	.equ LDX, 0x80
	.equ STX, 0x40
	.equ BINOP, 0x20
	.equ LDY, 0x10

    .text
	.global ker_x64_scalar

.macro LOAD reg
    mov eax, [rsi + 4 * r9]
    inc r9
    vmovsd \reg, [rdx + 8 * rax]
.endm

ker_x64_scalar:
    ; code = rdi
    ; words = rsi
    ; mem = rdx
    xor r8, r8  ; ip = r8
    xor r9, r9  ; pos = r9

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
    vmulsd xmm0, xmm0, xmm1
    jmp .Next

.A1:    /* LDY + ADD */
    LOAD xmm1
.B1:    /* ADD */
    vaddsd xmm0, xmm0, xmm1
    jmp .Next

.A2:    /* LDY + SUB */
    LOAD xmm1
.B2:    /* SUB */
    vsubsd xmm0, xmm0, xmm1
    jmp .Next

.A3:    /* LDY + DIV */
    LOAD xmm1
.B3:    /* DIV */
    vdivsd xmm0, xmm0, xmm1
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
    lea	r10, .NEG_ZERO[rip]
    vmovsd xmm3, [r10]
    vxorpd xmm0, xmm3, xmm0
    jmp .Next

.U2:    /* NOT */
    lea	r10, .ALL_ONES[rip]
    vmovsd xmm3, [r10]
    vxorpd xmm0, xmm3, xmm0
    jmp .Next

.U3:    /* RECIP */
    lea	r10, .ONE[rip]
    vmovsd xmm3, [r10]
    vdivsd xmm0, xmm3, xmm0
    jmp .Next

.U4:    /* ABS */
    lea	r10, .NEG_ZERO[rip]
    vmovsd xmm3, [r10]
    vandnpd xmm0, xmm3, xmm0
    jmp .Next

.U5:    /* ROOT */
.U6:    /* ROOT_REAL */
    vsqrtsd xmm0, xmm0, xmm0
    jmp .Next

.U7:    /* ROUND */
    vroundsd xmm0, xmm0, xmm0, 0
    jmp .Next
.U8:    /* FLOOR */
    vroundsd xmm0, xmm0, xmm0, 1
    jmp .Next
.U9:    /* REAL */
    jmp .Next
.U10:   /* IMAGINARY */
    vxorpd xmm0, xmm0, xmm0
    jmp .Next
.U11:   /* CONJUGATE */
    jmp .Next
.U12:   /* ISZERO */
    vxorpd xmm3, xmm3, xmm3
    vcmpeqsd xmm0, xmm3, xmm0
    jmp .Next
.U13:   /* POW */
    mov eax, [rsi + 4 * r9]
    inc r9
    vmovapd xmm4, xmm0
    lea	r10, .ONE[rip]
    vmovsd xmm0, [r10]
    or eax, eax
    jz .Next
    jns .P1
    neg eax
    vdivsd xmm4, xmm0, xmm4
.P1:
    test al, 1
    jz .P2
    vmulsd xmm0, xmm0, xmm4
.P2:
    sar eax, 1
    jz .Next
    vmulsd xmm4, xmm4, xmm4
    jmp .P1

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
    jmp .Next

.U19:   /* GEQ */
    vcmpeqsd xmm0, xmm0, xmm1
    jmp .Next

.U20:   /* LT */
    vcmpltsd xmm0, xmm0, xmm1
    jmp .Next

.U21:   /* LEQ */
    vcmpltsd xmm0, xmm0, xmm1
    jmp .Next

.U22:   /* EQ */
    vcmpeqsd xmm0, xmm0, xmm1
    jmp .Next

.U23:   /* NEQ */
    vcmpeqsd xmm0, xmm0, xmm1
    jmp .U2 /* NOT */
.U24:
.U25:
.U26:
.U27:
.U28:
.U29:
    jmp .Error
.U30:   /* DUP */
    vmodapd xmm1, xmm0
    jmp .Next
.U31:
    xor rax, rax    # return OK
    jmp .End

.Next:
    test cl, STX
    jz .Loop
    mov eax, [rsi + 4 * r9]
    inc r9
    vmovsd [rdx + 8 * rax], xmm0
    jmp .Loop

.Error:
    mov rax, rcx
    or rax, 256
.End:
    vzeroupper
    ret

    .section	.rodata
	.align 8
.NEG_ZERO:
    .quad   0x8000000000000000
.ALL_ONES:
    .quad   0xffffffffffffffff
.ONE:
    .quad   0x3ff0000000000000
.TWO:
    .quad   0x4000000000000000

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
