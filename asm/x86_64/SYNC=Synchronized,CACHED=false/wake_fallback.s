spmc_waker::SpmcWaker<S,_>::wake_fallback:
	push r14
	push rbx
	push rax
	mov rbx, r8
	mov rax, rsi
	mov rsi, -1
	mov r8, -3
	jmp .LBB1_1
.LBB1_9:
	mov r10, qword ptr [rdi + 16]
	mov r9, qword ptr [rdi + 24]
	mov rax, -1
	lock cmpxchg	qword ptr [rdi], rdx
	je .LBB1_10
.LBB1_1:
	cmp rax, -1
	je .LBB1_9
	cmp rax, 1
	je .LBB1_5
	test rax, rax
	jne .LBB1_8
	xor eax, eax
	lock cmpxchg	qword ptr [rdi], rdx
	jne .LBB1_1
	jmp .LBB1_11
.LBB1_5:
	mov eax, 1
	lock cmpxchg	qword ptr [rdi], r8
	jne .LBB1_1
	mov rax, qword ptr [rdi + 16]
	mov r10, qword ptr [rdi + 8]
	mov qword ptr [rdi + 8], rax
	mov r9, qword ptr [rdi + 24]
	or r9, 1
	mov rax, -3
	lock cmpxchg	qword ptr [rdi], r9
	je .LBB1_11
	mov qword ptr [rdi + 8], r10
	jmp .LBB1_1
.LBB1_8:
	lock cmpxchg	qword ptr [rdi], rsi
	jne .LBB1_1
	jmp .LBB1_9
.LBB1_10:
	mov rdi, r10
	mov r14, rcx
	call qword ptr [r9 + 8]
	mov rcx, r14
.LBB1_11:
	mov rdi, rbx
	add rsp, 8
	pop rbx
	pop r14
	jmp qword ptr [rcx + 24]
