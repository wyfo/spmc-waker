spmc_waker::SpmcWaker<_,_>::overwrite:
	push r15
	push r14
	push rbx
	cmp rdx, qword ptr [rdi + 8]
	jne .LBB0_2
	mov rax, rcx
	and rax, -2
	cmp rsi, rax
	je .LBB0_8
.LBB0_2:
	test cl, 2
	jne .LBB0_10
	#MEMBARRIER
	mov r8d, 24
	test cl, 1
	je .LBB0_5
	xor r8d, r8d
	mov rax, rcx
	lock cmpxchg	qword ptr [rdi], r8
	mov eax, 0
	mov r8d, 23
	jne .LBB0_9
.LBB0_5:
	mov rbx, rdx
	mov r15, rsi
	mov r14, rdi
	mov rdi, qword ptr [rdi + 8]
	call qword ptr [rcx + r8]
	mov rdi, rbx
	call qword ptr [r15]
	mov qword ptr [r14 + 8], rdx
	or rax, 1
	xchg qword ptr [r14], rax
.LBB0_8:
	mov al, 1
.LBB0_9:
	pop rbx
	pop r14
	pop r15
	ret
.LBB0_10:
	xor eax, eax
	pop rbx
	pop r14
	pop r15
	ret
	mov qword ptr [r14 + 8], 0
	mov rcx, qword ptr [rip + spmc_waker::NOOP_VTABLE@GOTPCREL]
	mov rcx, qword ptr [rcx]
	xchg qword ptr [r14], rcx
	mov rdi, rax
	call _Unwind_Resume@PLT
