asm_wake_cold_asm:
	mov rax, qword ptr [rdi]
	mov rsi, rax
	test al, 1
	jne .LBB8_3
	lock or	dword ptr [rsp - 64], 0
	mov rsi, qword ptr [rdi]
	test sil, 1
	jne .LBB8_3
	ret
.LBB8_3:
	#MEMBARRIER
	not al
	movzx edx, al
	and edx, 1
	jmp spmc_waker::SpmcWaker<S,_,R>::wake_impl_cold
