mov rcx, rsi
add rcx, rdi
lea	rax, .L4[rip]
xor rdx, rdx
inc rdx
	add	rax, QWORD PTR [rax + 8 * rdx]
	notrack jmp	rax
	add rcx, rdi
.S1:
add rcx, rdi
.S2:
mov rax, rcx
ret

	.section	.rodata
	.align 8
.L4:
	.quad	.S1-.L4
	.quad	.S2-.L4
