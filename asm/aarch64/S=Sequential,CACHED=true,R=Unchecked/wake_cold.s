asm_wake_cold_asm:
	ldar x1, [x0]
	tbnz w1, #0, .LBB7_2
	ret
.LBB7_2:
	b <spmc_waker::SpmcWaker<spmc_waker::synchronization::Sequential, true, spmc_waker::registration::Unchecked>>::wake_impl_cold
