spmc_waker::SpmcWaker<_,_>::wake_unsync_cold:
	tbnz w1, #1, .LBB0_3
	add x8, x1, #2
	mov x9, x1
	casa x9, x8, [x0]
	cmp x9, x1
	b.ne .LBB0_3
	sub x9, x1, #1
	ldr x8, [x0, #8]
	stlr x9, [x0]
	ldur x1, [x1, #7]
	mov x0, x8
	br x1
.LBB0_3:
	ret
