asm_wake_cold_asm:
	ldar x1, [x0]
	tbnz w1, #0, .LBB8_2
	ret
.LBB8_2:
	b spmc_waker::SpmcWaker<S,_,R>::wake_impl_cold
