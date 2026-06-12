spmc_waker::SpmcWaker<_,_>::wake_unsync_cold:
	mov ecx, 2
	mov rax, rsi
	lock cmpxchg	qword ptr [rdi], rcx
	jne .LBB0_1
	and rsi, -2
	mov rax, qword ptr [rdi + 8]
	mov qword ptr [rdi], rsi
	mov rdi, rax
	jmp qword ptr [rsi + 8]
.LBB0_1:
	ret
