.global main

.section .data
hex_format: .asciz "%#x"
float_format: .asciz "%.2f"
long_float_format: .asciz "%.2Lf"

.section .text

.macro trap
	movq $62, %rax
	movq %r12, %rdi
	movq $5, %rsi
	syscall
.endm

.macro print_rsi
	movq $0, %rax
	call printf@plt
	movq $0, %rdi
	call fflush@plt
.endm

main:
	push %rbp
	movq %rsp, %rbp

	# Get this program's PID, store it in r12, and then return control to the test
	movq $39, %rax
	syscall
	movq %rax, %r12
	trap

	# print the contents of %rsi (the second argument to printf); conveniently
	# this is the register we chose to write to in our unit test
	leaq hex_format(%rip), %rdi
	print_rsi
	trap

	# test an MMX register
	leaq hex_format(%rip), %rdi
	movq %mm0, %rsi
	print_rsi
	trap

	# test an XMMX register
	leaq float_format(%rip), %rdi
	movq $1, %rax
	call printf@plt
	movq $0, %rdi
	call fflush@plt
	trap

	# test an x87 (long double) register (currently unsupported)
	// subq $16, %rsp
	// fstpt (%rsp)
	// leaq long_float_format(%rip), %rdi
	// print_rsi
	// addq $16, %rsp
	// trap

	popq %rbp
	movq $0, %rax
	ret
