asm_wake_cold_asm:
	ldr x8, [x0]
	tbnz w8, #0, .LBB19_3
	ldsetl xzr, x1, [x0]
	tbnz w1, #0, .LBB19_4
	ret
.LBB19_3:
	mov x1, x8
.LBB19_4:
	tst x8, #0x1
	cset w2, eq
	b spmc_waker::SpmcWaker<S,_>::wake_registered_cold
