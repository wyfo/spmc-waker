asm_wake_cold_asm:
	mov rsi, qword ptr [rdi]
	test sil, 1
	jne spmc_waker::SpmcWaker<S,_,R>::wake_impl_cold
	ret
