spmc_waker::SpmcWaker<_,_>::wake_sync_cold:
	mov rax, qword ptr [rdi]
.LBB0_1:
	mov rcx, rax
	or rcx, 2
	lock cmpxchg	qword ptr [rdi], rcx
	jne .LBB0_1
	test al, 2
	jne .LBB0_5
	test al, 1
	jne .LBB0_6
	mov rcx, rax
	add rcx, 2
	mov rdx, rax
	mov rax, rcx
	lock cmpxchg	qword ptr [rdi], rdx
.LBB0_5:
	ret
.LBB0_6:
	mov rdx, rax
	dec rdx
	mov rcx, qword ptr [rdi + 8]
	xchg qword ptr [rdi], rdx
	mov rdi, rcx
	jmp qword ptr [rax + 7]
