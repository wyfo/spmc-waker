spmc_waker::SpmcWaker<_,_>::wake_unsync_cold:
	mov w8, #2
	mov x9, x1
	casa x9, x8, [x0]
	cmp x9, x1
	b.ne .LBB0_2
	and x9, x1, #0xfffffffffffffffe
	ldr x8, [x0, #8]
	stlr x9, [x0]
	ldr x1, [x9, #8]
	mov x0, x8
	br x1
.LBB0_2:
	ret
