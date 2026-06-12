spmc_waker::SpmcWaker<_,_>::overwrite:
	stp x29, x30, [sp, #-48]!
	str x21, [sp, #16]
	stp x20, x19, [sp, #32]
	mov x29, sp
	mov x19, x0
	ldr x0, [x0, #8]
	cmp x2, x0
	b.ne .LBB0_2
	and x8, x3, #0xfffffffffffffffe
	cmp x1, x8
	b.eq .LBB0_8
.LBB0_2:
	tbnz w3, #1, .LBB0_7
	tbz w3, #0, .LBB0_6
	mov x8, x3
	cas x8, xzr, [x19]
	cmp x8, x3
	b.ne .LBB0_7
	ldur x8, [x3, #23]
	mov x20, x2
	mov x21, x1
	blr x8
	mov x1, x21
	mov x2, x20
	ldr x8, [x1]
	mov x0, x2
	blr x8
	mov x8, x0
	mov w0, #1
	str x1, [x19, #8]
	orr x8, x8, #0x1
	stlr x8, [x19]
	ldp x20, x19, [sp, #32]
	ldr x21, [sp, #16]
	ldp x29, x30, [sp], #48
	ret
.LBB0_7:
	mov w0, wzr
	ldp x20, x19, [sp, #32]
	ldr x21, [sp, #16]
	ldp x29, x30, [sp], #48
	ret
.LBB0_8:
	mov w0, #1
	ldp x20, x19, [sp, #32]
	ldr x21, [sp, #16]
	ldp x29, x30, [sp], #48
	ret
