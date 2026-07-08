asm_wake_cold_asm:
	ldr x1, [x0]
	tbnz w1, #0, .LBB15_2
	ret
.LBB15_2:
	b spmc_waker::SpmcWaker<S,_>::wake_registered_cold
