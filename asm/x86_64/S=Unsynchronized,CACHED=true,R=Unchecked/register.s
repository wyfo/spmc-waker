asm_register_asm:
	mov rdx, qword ptr [rdi]
	test dl, 2
	je <spmc_waker::SpmcWaker<spmc_waker::synchronization::Unsynchronized, true, spmc_waker::registration::Unchecked>>::register_impl_cold
	mov rcx, qword ptr [rdi + 8]
	mov rax, qword ptr [rdi + 16]
	cmp rcx, qword ptr [rsi + 8]
	jne <spmc_waker::SpmcWaker<spmc_waker::synchronization::Unsynchronized, true, spmc_waker::registration::Unchecked>>::register_impl_cold
	cmp rax, qword ptr [rsi]
	jne <spmc_waker::SpmcWaker<spmc_waker::synchronization::Unsynchronized, true, spmc_waker::registration::Unchecked>>::register_impl_cold
	add rdx, 7
	mov qword ptr [rdi], rdx
	ret
