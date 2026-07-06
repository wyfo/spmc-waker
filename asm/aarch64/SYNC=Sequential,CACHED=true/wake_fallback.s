spmc_waker::SpmcWaker<S,_>::wake_fallback:
	stp x29, x30, [sp, #-16]!
	mov x29, sp
	mov x9, #-1
	mov x10, #-3
	b .LBB1_2
.LBB1_1:
	mov x1, #-1
	ldr x8, [x0, #16]
	ldr x11, [x0, #24]
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
	b .LBB1_11
.LBB1_6:
	casa x1, x10, [x0]
	cmp x1, #1
	b.ne .LBB1_2
	ldr x11, [x0, #16]
	ldr x8, [x0, #8]
	mov x1, #-3
	str x11, [x0, #8]
	ldr x11, [x0, #24]
	orr x11, x11, #0x1
	casl x1, x11, [x0]
	cmn x1, #3
	b.eq .LBB1_12
	str x8, [x0, #8]
	b .LBB1_2
.LBB1_9:
	mov x8, x1
	casa x8, x9, [x0]
	cmp x8, x1
	mov x1, x8
	b.ne .LBB1_2
	b .LBB1_1
.LBB1_10:
	ldr x9, [x11, #8]
	mov x0, x8
	blr x9
	mov w0, #1
	ldp x29, x30, [sp], #16
	ret
.LBB1_11:
	mov w0, wzr
	ldp x29, x30, [sp], #16
	ret
.LBB1_12:
	ldr x8, [x3, #24]
	mov x0, x4
	blr x8
	mov w0, wzr
	ldp x29, x30, [sp], #16
	ret
