spmc_waker::SpmcWaker<S,_>::wake_registered_cold:
	tbnz w1, #1, .LBB4_5
	add x9, x1, #2
	mov x8, x1
	casal x8, x9, [x0]
	cmp x8, x1
	b.eq .LBB4_7
	sub x9, x1, #4
	cmn x9, #8
	b.lo .LBB4_5
	mov x1, x8
	tbnz w8, #0, .LBB4_1
.LBB4_5:
	tbnz w2, #0, .LBB4_8
	ldsetl xzr, x1, [x0]
	mov w2, #1
	tbnz w1, #0, .LBB4_1
	b .LBB4_8
.LBB4_7:
	add x8, x1, #4
	cmp x8, #8
	b.hs .LBB4_9
.LBB4_8:
	ret
.LBB4_9:
	sub x2, x1, #1
	add x8, x1, #2
	ldr x4, [x0, #8]
	casl x8, x2, [x0]
	cmp x8, x9
	b.ne .LBB4_11
	ldur x1, [x1, #7]
	mov x0, x4
	br x1
.LBB4_11:
	mov x1, x8
	mov x3, x2
	b spmc_waker::SpmcWaker<S,_>::wake_fallback
