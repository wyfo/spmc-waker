spmc_waker::SpmcWaker<_,_>::wake_unsync_cold:
	push r14
	push rbx
	push rax
	test sil, 2
	jne .LBB0_4
	lea rcx, [rsi + 2]
	mov rax, rsi
	lock cmpxchg	qword ptr [rdi], rcx
	jne .LBB0_4
	lea rbx, [rsi - 1]
	mov r14, rdi
	mov rdi, qword ptr [rdi + 8]
	call qword ptr [rsi + 15]
	mov qword ptr [r14], rbx
.LBB0_4:
	add rsp, 8
	pop rbx
	pop r14
	ret
	mov qword ptr [r14], rbx
	mov rdi, rax
	call _Unwind_Resume@PLT
