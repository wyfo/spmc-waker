spmc_waker::SpmcWaker<_,_>::wake_fallback:
	stp x29, x30, [sp, #-32]!
	stp x20, x19, [sp, #16]
	mov x29, sp
	mov x8, #-1
	mov x9, #-3
	b .LBB1_2
.LBB1_1:
	mov x1, #-1
	ldr x19, [x0, #16]
	ldr x20, [x0, #24]
	casl x1, x2, [x0]
	cmn x1, #1
	b.eq .LBB1_10
.LBB1_2:
	cmn x1, #1
	b.eq .LBB1_1
	cmp x1, #1
	b.eq .LBB1_6
	cbnz x1, .LBB1_9
	casl x1, x2, [x0]
	cmp x1, #0
	b.ne .LBB1_2
	b .LBB1_12
.LBB1_6:
	casa x1, x9, [x0]
	cmp x1, #1
	b.ne .LBB1_2
	ldr x11, [x0, #16]
	ldr x10, [x0, #8]
	mov x1, #-3
	str x11, [x0, #8]
	ldr x11, [x0, #24]
	orr x11, x11, #0x1
	casl x1, x11, [x0]
	cmn x1, #3
	b.eq .LBB1_11
	str x10, [x0, #8]
	b .LBB1_2
	mov x10, x1
	casa x10, x8, [x0]
	cmp x10, x1
	mov x1, x10
	b.ne .LBB1_2
	b .LBB1_1
.LBB1_10:
	ldr x8, [x20, #16]
	mov x0, x19
	blr x8
	mov x3, x20
	mov x4, x19
.LBB1_11:
	ldr x1, [x3, #24]
	mov x0, x4
	ldp x20, x19, [sp, #16]
	ldp x29, x30, [sp], #32
	br x1
.LBB1_12:
	ldp x20, x19, [sp, #16]
	ldp x29, x30, [sp], #32
	ret
