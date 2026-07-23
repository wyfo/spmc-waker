asm_wake_cold_asm:
	mov rax, qword ptr [rdi]
	mov rsi, rax
	test al, 1
	jne .LBB7_3
	lock or	dword ptr [rsp - 64], 0
	mov rsi, qword ptr [rdi]
	test sil, 1
	jne .LBB7_3
	ret
.LBB7_3:
	#MEMBARRIER
	not al
	movzx edx, al
	and edx, 1
	jmp <spmc_waker::SpmcWaker<spmc_waker::synchronization::Synchronized, true, spmc_waker::registration::Unchecked>>::wake_impl_cold
