spmc_waker::SpmcWaker<_,_>::wake_unsync_cold:
	push r14
	push rbx
	push rax
	mov ecx, 2
	mov rax, rsi
	lock cmpxchg	qword ptr [rdi], rcx
	jne .LBB0_3
	mov rbx, rsi
	mov r14, rdi
	and rbx, -2
	mov rdi, qword ptr [rdi + 8]
	call qword ptr [rbx + 16]
	mov qword ptr [r14], rbx
.LBB0_3:
	add rsp, 8
	pop rbx
	pop r14
	ret
	mov qword ptr [r14], rbx
	mov rdi, rax
	call _Unwind_Resume@PLT
