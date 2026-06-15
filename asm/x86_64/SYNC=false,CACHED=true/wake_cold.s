asm_wake_cold_asm:
	mov rsi, qword ptr [rdi]
	test sil, 1
	jne .LBB15_2
	ret
.LBB15_2:
	jmp qword ptr [rip + spmc_waker::SpmcWaker<_,_>::wake_registered_cold@GOTPCREL]
