spmc_waker::SpmcWaker<S,_>::wake_registered_cold:
	push r15
	push r14
	push r12
	push rbx
	push rax
.LBB4_1:
	test sil, 2
	jne .LBB4_8
	lea r14, [rsi + 2]
	mov rax, rsi
	lock cmpxchg	qword ptr [rdi], r14
	je .LBB4_5
	add rsi, -4
	cmp rsi, -8
	jb .LBB4_8
	mov ecx, eax
	mov rsi, rax
	and ecx, 1
	jne .LBB4_1
	jmp .LBB4_8
.LBB4_5:
	lea rax, [rsi + 4]
	cmp rax, 8
	jb .LBB4_8
	lea rbx, [rsi - 1]
	mov r12, rdi
	mov r15, qword ptr [rdi + 8]
	mov rdi, r15
	call qword ptr [rsi + 15]
	mov rax, r14
	lock cmpxchg	qword ptr [r12], rbx
	jne .LBB4_10
.LBB4_8:
	xor eax, eax
	add rsp, 8
	pop rbx
	pop r12
	pop r14
	pop r15
	ret
.LBB4_10:
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
