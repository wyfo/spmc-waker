asm_wake_cold_asm:
	ldr x1, [x0]
	tbnz w1, #0, .LBB8_2
	ret
.LBB8_2:
	dmb ishld
	b spmc_waker::SpmcWaker<S,_,R>::wake_impl_cold
