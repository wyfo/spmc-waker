spmc_waker::SpmcWaker<S,_>::register_cold:
	push r14
	push rbx
	push rax
	mov r14, rdx
	mov rdx, rsi
	mov rbx, rdi
	mov rsi, qword ptr [rsi]
	lea rax, [rsi + 1]
	cmp rax, r14
	sete al
	test r8b, al
	je .LBB0_2
	mov rax, qword ptr [rdx + 8]
	cmp rax, qword ptr [rbx + 8]
	je .LBB0_9
.LBB0_2:
	test r14b, 2
	jne .LBB0_11
	test r14b, 1
	jne .LBB0_13
.LBB0_4:
	mov rdi, qword ptr [rdx + 8]
	test r8b, r8b
	je .LBB0_7
	call qword ptr [rsi]
	mov rsi, rax
	mov rdi, rdx
.LBB0_7:
	inc rsi
	mov rcx, qword ptr [rbx + 8]
	mov qword ptr [rbx + 8], rdi
	xchg qword ptr [rbx], rsi
	mov al, 1
	and r14, -2
	je .LBB0_10
	mov rdi, rcx
	call qword ptr [r14 + 24]
.LBB0_9:
	mov al, 1
.LBB0_10:
	add rsp, 8
	pop rbx
	pop r14
	ret
.LBB0_11:
	test cl, cl
	jne .LBB0_14
.LBB0_12:
	xor eax, eax
	add rsp, 8
	pop rbx
	pop r14
	ret
.LBB0_13:
	lea rax, [r14 + 4]
	cmp rax, 8
	jae .LBB0_15
.LBB0_14:
	movzx ecx, r8b
	mov rdi, rbx
	mov rsi, r14
	add rsp, 8
	pop rbx
	pop r14
	jmp spmc_waker::SpmcWaker<S,_>::register_fallback
.LBB0_15:
	lea rdi, [rip + .Lanon.8a4a1e93038d20d21177fc6b1c36bd50.0]
	mov rax, r14
	lock cmpxchg	qword ptr [rbx], rdi
	je .LBB0_4
	test cl, cl
	je .LBB0_12
	test al, 2
	jne .LBB0_19
	mov r14, rax
	jmp .LBB0_4
.LBB0_19:
	movzx ecx, r8b
	mov rdi, rbx
	mov rsi, rax
	add rsp, 8
	pop rbx
	pop r14
	jmp spmc_waker::SpmcWaker<S,_>::register_fallback
	test r14b, 1
	je .LBB0_22
	xchg qword ptr [rbx], r14
.LBB0_22:
	mov rdi, rax
	call _Unwind_Resume@PLT
