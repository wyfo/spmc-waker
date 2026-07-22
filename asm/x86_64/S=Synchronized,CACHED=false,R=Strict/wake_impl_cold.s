spmc_waker::SpmcWaker<S,_,R>::wake_impl_cold:
	mov rax, rsi
	mov rcx, qword ptr [rdi + 8]
	mov rsi, qword ptr [rdi + 16]
	lea r8, [rax - 1]
	lock cmpxchg	qword ptr [rdi], r8
	jne .LBB0_1
.LBB0_5:
	mov rdi, rcx
	jmp qword ptr [rsi + 8]
.LBB0_1:
	test dl, dl
	jne .LBB0_4
	lock or	dword ptr [rsp - 64], 0
	mov rax, qword ptr [rdi]
	test al, 1
	je .LBB0_4
	#MEMBARRIER
	mov rcx, qword ptr [rdi + 8]
	mov rsi, qword ptr [rdi + 16]
	lea rdx, [rax - 1]
	lock cmpxchg	qword ptr [rdi], rdx
	je .LBB0_5
.LBB0_4:
	ret
