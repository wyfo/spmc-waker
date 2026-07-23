asm_wake_cold_asm:
	mov rsi, qword ptr [rdi]
	test sil, 1
	jne .LBB7_2
	ret
.LBB7_2:
	#MEMBARRIER
	jmp <spmc_waker::SpmcWaker<spmc_waker::synchronization::Unsynchronized, true>>::wake_impl_cold
