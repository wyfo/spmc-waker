asm_wake_asm:
	ldr x8, [x0]
	tst x8, #0x1
	cset w9, eq
	tbnz w8, #0, .LBB14_2
	ldsetl xzr, x8, [x0]
	tbz w8, #0, .LBB14_9
	tbnz w8, #1, .LBB14_6
	add x10, x8, #2
	mov x11, x8
	casal x11, x10, [x0]
	cmp x11, x8
	b.eq .LBB14_8
	sub x8, x8, #4
	cmn x8, #8
	b.lo .LBB14_6
	mov x8, x11
	tbnz w11, #0, .LBB14_2
.LBB14_6:
	tbnz w9, #0, .LBB14_9
	ldsetl xzr, x8, [x0]
	mov w9, #1
	tbnz w8, #0, .LBB14_2
	b .LBB14_9
.LBB14_8:
	add x9, x8, #4
	cmp x9, #8
	b.hs .LBB14_10
.LBB14_9:
	ret
.LBB14_10:
	sub x2, x8, #1
	add x1, x8, #2
	ldr x4, [x0, #8]
	casl x1, x2, [x0]
	cmp x1, x10
	b.ne .LBB14_12
	ldur x1, [x8, #7]
	mov x0, x4
	br x1
.LBB14_12:
	mov x3, x2
	b spmc_waker::SpmcWaker<_,_>::wake_fallback
