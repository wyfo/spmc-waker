asm_wake_cold_asm:
	mov rsi, qword ptr [rdi]
	test sil, 1
	jne <spmc_waker::SpmcWaker<spmc_waker::synchronization::Sequential, true>>::wake_impl_cold
	ret
