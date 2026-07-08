asm_wake_cold_asm:
	mov rsi, qword ptr [rdi]
	test sil, 1
	jne .LBB19_2
	ret
.LBB19_2:
	jmp qword ptr [rip + spmc_waker::SpmcWaker<S,_>::wake_registered_cold@GOTPCREL]
