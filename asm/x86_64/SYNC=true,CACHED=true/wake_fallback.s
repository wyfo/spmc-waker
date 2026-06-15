spmc_waker::SpmcWaker<_,_>::wake_fallback:
	push r14
	push rbx
	push rax
	mov rax, rsi
	mov rsi, -1
	mov r9, -3
	jmp .LBB1_1
.LBB1_10:
	mov rbx, qword ptr [rdi + 16]
	mov r10, qword ptr [rdi + 24]
	mov rax, -1
	lock cmpxchg	qword ptr [rdi], rdx
	je .LBB1_11
.LBB1_1:
	cmp rax, -1
	je .LBB1_10
	cmp rax, 1
	je .LBB1_6
	test rax, rax
	jne .LBB1_9
	xor eax, eax
	lock cmpxchg	qword ptr [rdi], rdx
	jne .LBB1_1
	jmp .LBB1_5
.LBB1_6:
	mov eax, 1
	lock cmpxchg	qword ptr [rdi], r9
	jne .LBB1_1
	mov rax, qword ptr [rdi + 16]
	mov r10, qword ptr [rdi + 8]
	mov qword ptr [rdi + 8], rax
	mov r11, qword ptr [rdi + 24]
	or r11, 1
	mov rax, -3
	lock cmpxchg	qword ptr [rdi], r11
	je .LBB1_8
	mov qword ptr [rdi + 8], r10
	jmp .LBB1_1
.LBB1_9:
	lock cmpxchg	qword ptr [rdi], rsi
	jne .LBB1_1
	jmp .LBB1_10
.LBB1_11:
	mov rdi, rbx
	mov r14, r10
	call qword ptr [r10 + 16]
	mov rcx, r14
	jmp .LBB1_12
.LBB1_5:
	add rsp, 8
	pop rbx
	pop r14
	ret
.LBB1_8:
	mov rbx, r8
.LBB1_12:
	mov rdi, rbx
	add rsp, 8
	pop rbx
	pop r14
	jmp qword ptr [rcx + 24]
