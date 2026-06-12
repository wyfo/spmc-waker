spmc_waker::SpmcWaker<_,_>::wake_unsync_cold:
	test sil, 2
	jne .LBB0_2
	lea rcx, [rsi + 2]
	mov rax, rsi
	lock cmpxchg	qword ptr [rdi], rcx
	jne .LBB0_2
	lea rcx, [rsi - 1]
	mov rax, qword ptr [rdi + 8]
	mov qword ptr [rdi], rcx
	mov rdi, rax
	jmp qword ptr [rsi + 7]
.LBB0_2:
	ret
