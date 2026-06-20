	.intel_syntax noprefix


	.equ LDX, 0x80
	.equ STX, 0x40
	.equ BINOP, 0x20
	.equ LDY, 0x10

	.equ MUL, 0
	.equ ADD, 1
	.equ SUB, 2
	.equ DIV, 3
	.equ POWF, 4
	.equ AND, 5
	.equ OR, 6
	.equ XOR, 7
	.equ COMPLEX, 8
	.equ MOVZ, 9


    .equ ASSIGN, 0; // also NOP
    .equ NEG, 1
    .equ NOT, 2
    .equ RECIP, 3
    .equ ABS, 4
    .equ ROOT, 5
    .equ ROOT_REAL, 6
    .equ ROUND, 7
    .equ FLOOR, 8
    .equ REAL, 9
    .equ IMAGINARY, 10
    .equ CONJUGATE, 11
    .equ ISZERO, 12
    .equ POW, 13
    .equ GOTO, 14
    .equ BRANCH_IF, 15
    .equ BRANCH_ELSE, 16
    .equ JOIN, 17
    .equ GT, 18
    .equ GEQ, 19
    .equ LT, 20
    .equ LEQ, 21
    .equ EQ, 22
    .equ NEQ, 23

    .equ DUP, 30
    .equ RET, 31

	.text
	.global ker_x64_scalar

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
    mov eax, [rsi + 4 * r9]
    inc r9
    vmovsd xmm0, [rdx + 8 * rax]
.L1:
    test cl, BINOP
    jz .Uni

    test cl, LDY
    jz .Bi
    mov eax, [rsi + 4 * r9]
    inc r9
    vmovsd xmm1, [rdx + 8 * rax]
.Bi:
    xor rax, rax
    mov al, cl
    and al, 0x0f
    lea	r10, .BT[rip]
    add	r10, QWORD PTR [r10 + 8 * rax]
	notrack jmp	r10

.B0:    /* MUL */
    vmulsd xmm0, xmm0, xmm1
    jmp .Next
.B1:    /* ADD */
    vaddsd xmm0, xmm0, xmm1
    jmp .Next
.B2:    /* SUB */
    vsubsd xmm0, xmm0, xmm1
    jmp .Next
.B3:    /* DIV */
    vdivsd xmm0, xmm0, xmm1
    jmp .Next
.B4:    /* POWF */
    jmp .Next
.B5:    /* AND */
    vandpd xmm0, xmm0, xmm1
    jmp .Next
.B6:    /* OR */
    vorpd xmm0, xmm0, xmm1
    jmp .Next
.B7:    /* XOR */
    vxorpd xmm0, xmm0, xmm1
    jmp .Next
.B8:    /* COMPLEX */
    jmp .Next
.B9:    /* MOVZ */
    vmovapd xmm2, xmm0
    jmp .Next
.B10:
.B11:
.B12:
.B13:
.B14:
.B15:
    jmp .Next

.Uni:
    xor rax, rax
    mov al, cl
    and al, 0x1f
    lea	r10, .UT[rip]
    add	r10, QWORD PTR [r10 + 8 * rax]
	notrack jmp	r10
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
    vxorpd xmm0, xmm3, xmm0
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
.U7:
.U8:
.U9:
.U10:
.U11:
.U12:
.U13:
.U14:
.U15:
.U16:
.U17:
.U18:
.U19:
.U20:
.U21:
.U22:
.U23:
.U24:
.U25:
.U26:
.U27:
.U28:
.U29:
.U30:
    jmp .Next
.U31:
    jmp .End

.Next:
    test cl, STX
    jz .Loop
    mov eax, [rsi + 4 * r9]
    inc r9
    vmovsd [rdx + 8 * rax], xmm0
    jmp .Loop

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

.UT:
	.quad	.U0-.UT
	.quad	.U1-.UT
	.quad	.U2-.UT
	.quad	.U3-.UT
	.quad	.U4-.UT
	.quad	.U5-.UT
	.quad	.U6-.UT
	.quad	.U7-.UT
	.quad	.U8-.UT
	.quad	.U9-.UT
	.quad	.U10-.UT
	.quad	.U11-.UT
	.quad	.U12-.UT
	.quad	.U13-.UT
	.quad	.U14-.UT
	.quad	.U15-.UT
	.quad	.U16-.UT
	.quad	.U17-.UT
	.quad	.U18-.UT
	.quad	.U19-.UT
	.quad	.U20-.UT
	.quad	.U21-.UT
	.quad	.U22-.UT
	.quad	.U23-.UT
	.quad	.U24-.UT
	.quad	.U25-.UT
	.quad	.U26-.UT
	.quad	.U27-.UT
	.quad	.U28-.UT
	.quad	.U29-.UT
	.quad	.U30-.UT
	.quad	.U31-.UT
