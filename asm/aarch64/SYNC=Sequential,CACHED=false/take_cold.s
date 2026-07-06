asm_take_cold_asm:
	ldar x1, [x0]
	tbnz w1, #0, .LBB15_2
	mov x0, xzr
	ret
.LBB15_2:
	stp x29, x30, [sp, #-16]!
	mov x29, sp
	bl spmc_waker::SpmcWaker<S,_>::wake_registered_cold
	ldp x29, x30, [sp], #16
	ret
