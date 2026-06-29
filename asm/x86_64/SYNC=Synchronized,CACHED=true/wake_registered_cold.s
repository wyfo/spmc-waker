spmc_waker::SpmcWaker<S,_>::wake_registered_cold:
	push r15
	push r14
	push r12
	push rbx
	push rax
	mov rbx, rdi
.LBB4_1:
	test sil, 2
	jne .LBB4_5
	lea r15, [rsi + 2]
	mov rax, rsi
	lock cmpxchg	qword ptr [rbx], r15
	je .LBB4_8
	add rsi, -4
	cmp rsi, -8
	jb .LBB4_5
	mov ecx, eax
	mov rsi, rax
	and ecx, 1
	jne .LBB4_1
.LBB4_5:
	test dl, 1
	jne .LBB4_7
	lock or	dword ptr [rsp - 64], 0
	mov rsi, qword ptr [rbx]
	mov dl, 1
	test sil, 1
	jne .LBB4_1
	jmp .LBB4_7
.LBB4_8:
	lea rax, [rsi + 4]
	cmp rax, 8
	jb .LBB4_7
	lea r14, [rsi - 1]
	mov r12, qword ptr [rbx + 8]
	mov rdi, r12
	call qword ptr [rsi + 15]
	mov rax, r15
	lock cmpxchg	qword ptr [rbx], r14
	jne .LBB4_11
.LBB4_7:
	add rsp, 8
	pop rbx
	pop r12
	pop r14
	pop r15
	ret
.LBB4_11:
	mov rdi, rbx
	mov rsi, rax
	mov rdx, r14
	mov rcx, r14
	mov r8, r12
	add rsp, 8
	pop rbx
	pop r12
	pop r14
	pop r15
	jmp spmc_waker::SpmcWaker<S,_>::wake_fallback
	xchg qword ptr [rbx], r14
	mov rdi, rax
	call _Unwind_Resume@PLT
