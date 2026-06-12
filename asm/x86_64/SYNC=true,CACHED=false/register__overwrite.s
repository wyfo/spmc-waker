spmc_waker::SpmcWaker<_,_>::overwrite:
	push r15
	push r14
	push rbx
	mov rbx, rdi
	mov rdi, qword ptr [rdi + 8]
	cmp rdx, rdi
	jne .LBB0_2
	mov rax, rcx
	and rax, -2
	cmp rsi, rax
	je .LBB0_7
.LBB0_2:
	test cl, 2
	jne .LBB0_10
	test cl, 1
	je .LBB0_6
	xor r8d, r8d
	mov rax, rcx
	lock cmpxchg	qword ptr [rbx], r8
	jne .LBB0_9
	mov r14, rsi
	mov r15, rdx
	call qword ptr [rcx + 23]
	mov rdx, r15
	mov rsi, r14
.LBB0_6:
	mov rdi, rdx
	call qword ptr [rsi]
	mov qword ptr [rbx + 8], rdx
	or rax, 1
	xchg qword ptr [rbx], rax
.LBB0_7:
	mov al, 1
	pop rbx
	pop r14
	pop r15
	ret
.LBB0_9:
	#MEMBARRIER
.LBB0_10:
	xor eax, eax
	pop rbx
	pop r14
	pop r15
	ret
