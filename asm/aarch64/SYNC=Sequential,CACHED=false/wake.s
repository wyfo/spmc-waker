asm_wake_asm:
	ldar x8, [x0]
	tbnz w8, #0, .LBB14_2
.LBB14_1:
	ret
	tbnz w8, #1, .LBB14_1
	add x9, x8, #2
	mov x10, x8
	casa x10, x9, [x0]
	cmp x10, x8
	b.eq .LBB14_6
	sub x8, x8, #4
	cmn x8, #8
	b.lo .LBB14_1
	mov x8, x10
	tbnz w10, #0, .LBB14_2
	b .LBB14_1
.LBB14_6:
	add x10, x8, #4
	cmp x10, #8
	b.lo .LBB14_1
	sub x2, x8, #1
	add x1, x8, #2
	ldr x4, [x0, #8]
	casl x1, x2, [x0]
	cmp x1, x9
	b.ne .LBB14_9
	ldur x1, [x8, #7]
	mov x0, x4
	br x1
.LBB14_9:
	mov x3, x2
	b spmc_waker::SpmcWaker<S,_>::wake_fallback
