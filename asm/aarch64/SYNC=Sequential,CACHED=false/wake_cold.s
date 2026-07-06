asm_wake_cold_asm:
	ldar x1, [x0]
	tbnz w1, #0, .LBB19_2
	ret
.LBB19_2:
	b spmc_waker::SpmcWaker<S,_>::wake_registered_cold
