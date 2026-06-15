asm_wake_cold_asm:
	mov rax, qword ptr [rdi]
	mov rsi, rax
	test al, 1
	jne .LBB15_3
	lock or	dword ptr [rsp - 64], 0
	mov rsi, qword ptr [rdi]
	test sil, 1
	jne .LBB15_3
	ret
.LBB15_3:
	xor edx, edx
	test al, 1
	sete dl
	jmp qword ptr [rip + spmc_waker::SpmcWaker<_,_>::wake_registered_cold@GOTPCREL]
