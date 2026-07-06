asm_take_cold_asm:
	push rax
	mov rax, qword ptr [rdi]
	mov rsi, rax
	test al, 1
	jne .LBB15_3
	lock or	dword ptr [rsp - 64], 0
	mov rsi, qword ptr [rdi]
	test sil, 1
	jne .LBB15_3
	xor eax, eax
	pop rcx
	ret
.LBB15_3:
	not al
	movzx edx, al
	and edx, 1
	call qword ptr [rip + spmc_waker::SpmcWaker<S,_>::wake_registered_cold@GOTPCREL]
	pop rcx
	ret
