asm_wake_cold_asm:
	mov rax, qword ptr [rdi]
	mov rsi, rax
	test al, 1
	jne .LBB19_3
	lock or	dword ptr [rsp - 64], 0
	mov rsi, qword ptr [rdi]
	test sil, 1
	jne .LBB19_3
	ret
.LBB19_3:
	not al
	movzx edx, al
	and edx, 1
	jmp qword ptr [rip + spmc_waker::SpmcWaker<S,_>::wake_registered_cold@GOTPCREL]
