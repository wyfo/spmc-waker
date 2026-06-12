spmc_waker::SpmcWaker<_,_>::wake_unsync_cold:
	stp x29, x30, [sp, #-32]!
	stp x20, x19, [sp, #16]
	mov x29, sp
	tbnz w1, #1, .LBB0_4
	add x8, x1, #2
	mov x9, x1
	casa x9, x8, [x0]
	cmp x9, x1
	b.ne .LBB0_4
	mov x19, x0
	ldr x0, [x0, #8]
	ldur x8, [x1, #15]
	sub x20, x1, #1
	blr x8
	stlr x20, [x19]
.LBB0_4:
	ldp x20, x19, [sp, #16]
	ldp x29, x30, [sp], #32
	ret
	stlr x20, [x19]
	bl _Unwind_Resume
