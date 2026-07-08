asm_take_cold_asm:
	ldr x8, [x0]
	tbnz w8, #0, .LBB15_3
	ldsetl xzr, x1, [x0]
	tbnz w1, #0, .LBB15_4
	mov x0, xzr
	ret
.LBB15_3:
	mov x1, x8
.LBB15_4:
	stp x29, x30, [sp, #-16]!
	mov x29, sp
	tst x8, #0x1
	cset w2, eq
	bl spmc_waker::SpmcWaker<S,_>::wake_registered_cold
	ldp x29, x30, [sp], #16
	ret
