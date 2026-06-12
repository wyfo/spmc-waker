spmc_waker::SpmcWaker<_,_>::wake_unsync_cold:
	stp x29, x30, [sp, #-32]!
	stp x20, x19, [sp, #16]
	mov x29, sp
	mov w8, #2
	mov x9, x1
	casa x9, x8, [x0]
	cmp x9, x1
	b.ne .LBB0_3
	mov x19, x0
	and x20, x1, #0xfffffffffffffffe
	ldr x0, [x0, #8]
	ldr x8, [x20, #16]
	blr x8
	stlr x20, [x19]
.LBB0_3:
	ldp x20, x19, [sp, #16]
	ldp x29, x30, [sp], #32
	ret
	stlr x20, [x19]
	bl _Unwind_Resume
