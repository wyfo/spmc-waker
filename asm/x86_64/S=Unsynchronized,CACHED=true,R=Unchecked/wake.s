asm_wake_asm:
	push r15
	push r14
	push r13
	push r12
	push rbx
	mov r15, qword ptr [rdi]
	test r15b, 1
	je .LBB7_8
	mov r14, rdi
	#MEMBARRIER
	mov rbx, qword ptr [rdi + 8]
	mov rcx, qword ptr [rdi + 16]
	lea r12, [r15 - 1]
	mov rax, r15
	lock cmpxchg	qword ptr [rdi], r12
	jne .LBB7_8
	mov rdi, rbx
	mov r13, rcx
	call qword ptr [rcx + 16]
	inc r15
	mov rax, r12
	lock cmpxchg	qword ptr [r14], r15
	jne .LBB7_4
.LBB7_8:
	pop rbx
	pop r12
	pop r13
	pop r14
	pop r15
	ret
.LBB7_4:
	mov rax, r13
	mov rdi, rbx
	pop rbx
	pop r12
	pop r13
	pop r14
	pop r15
	jmp qword ptr [rax + 24]
	mov r14, rax
	mov rdi, rbx
	call qword ptr [r13 + 24]
	mov rdi, r14
	call _Unwind_Resume@PLT
	call qword ptr [rip + core::panicking::panic_in_cleanup@GOTPCREL]
