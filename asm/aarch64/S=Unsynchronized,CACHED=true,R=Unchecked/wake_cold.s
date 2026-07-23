asm_wake_cold_asm:
	ldr x1, [x0]
	tbnz w1, #0, .LBB7_2
	ret
.LBB7_2:
	dmb ishld
	b <spmc_waker::SpmcWaker<spmc_waker::synchronization::Unsynchronized, true, spmc_waker::registration::Unchecked>>::wake_impl_cold
