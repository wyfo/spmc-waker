asm_wake_asm:
	push r15
	push r14
	push r12
	push rbx
	push rax
	mov rcx, qword ptr [rdi]
	test cl, 1
	jne .LBB14_1
.LBB14_8:
	add rsp, 8
	pop rbx
	pop r12
	pop r14
	pop r15
	ret
.LBB14_1:
	test cl, 2
	jne .LBB14_8
	lea r14, [rcx + 2]
	mov rax, rcx
	lock cmpxchg	qword ptr [rdi], r14
	je .LBB14_5
	add rcx, -4
	cmp rcx, -8
	jb .LBB14_8
	mov edx, eax
	mov rcx, rax
	and edx, 1
	jne .LBB14_1
	jmp .LBB14_8
.LBB14_5:
	lea rax, [rcx + 4]
	cmp rax, 8
	jb .LBB14_8
	lea rbx, [rcx - 1]
	mov r12, rdi
	mov r15, qword ptr [rdi + 8]
	mov rdi, r15
	call qword ptr [rcx + 15]
	mov rax, r14
	lock cmpxchg	qword ptr [r12], rbx
	je .LBB14_8
	mov rdi, r12
	mov rsi, rax
	mov rdx, rbx
	mov rcx, rbx
	mov r8, r15
	add rsp, 8
	pop rbx
	pop r12
	pop r14
	pop r15
	jmp spmc_waker::SpmcWaker<S,_>::wake_fallback
	xchg qword ptr [r12], rbx
	mov rdi, rax
	call _Unwind_Resume@PLT
