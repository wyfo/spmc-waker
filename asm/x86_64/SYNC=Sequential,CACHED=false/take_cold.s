asm_take_cold_asm:
	mov rsi, qword ptr [rdi]
	test sil, 1
	jne .LBB15_2
	xor eax, eax
	ret
.LBB15_2:
	push rax
	call qword ptr [rip + spmc_waker::SpmcWaker<S,_>::wake_registered_cold@GOTPCREL]
	add rsp, 8
	ret
