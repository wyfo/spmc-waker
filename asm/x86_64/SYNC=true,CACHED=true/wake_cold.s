sc_wake_cold:
	mov rax, qword ptr [rdi]
	test al, 1
	jne .LBB1_3
	lock or	dword ptr [rsp - 64], 0
	mov rax, qword ptr [rdi]
	test al, 1
	jne .LBB1_3
	ret
.LBB1_3:
	jmp qword ptr [rip + spmc_waker::SpmcWaker<_,_>::wake_sync_cold@GOTPCREL]
