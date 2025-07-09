.global main

.section .data
test_double: .double 64.125

.section .text

.macro trap
	movq $62, %rax
	movq %r12, %rdi
	movq $5, %rsi
	syscall
.endm

main:
	push %rbp
	movq %rsp, %rbp

	# Get this program's PID, store it in r12, and then return control to the test
	movq $39, %rax
	syscall
	movq %rax, %r12

	movq $0xcafecafe, %r13
	trap

	movb $42, %r13b
	trap

	movq $0xba5eba11ba5eba11, %r13
	movq %r13, %mm0
	trap

	movsd test_double(%rip), %xmm0
	trap

	# test an x87 (long double) register (currently unsupported)
	// emms
	// fldl test_double(%rip)
	// trap

	popq %rbp
	movq $0, %rax
	ret
