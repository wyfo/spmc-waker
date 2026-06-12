spmc_waker::SpmcWaker<_,_>::overwrite:
	stp x29, x30, [sp, #-48]!
	str x21, [sp, #16]
	stp x20, x19, [sp, #32]
	mov x29, sp
	ldr x8, [x0, #8]
	cmp x2, x8
	b.ne .LBB0_2
	and x8, x3, #0xfffffffffffffffe
	cmp x1, x8
	b.eq .LBB0_11
.LBB0_2:
	tbnz w3, #1, .LBB0_10
	dmb ishld
	tbnz w3, #0, .LBB0_5
	mov x21, x1
	mov x19, x2
	mov w8, #24
	b .LBB0_7
	mov x8, x3
	cas x8, xzr, [x0]
	cmp x8, x3
	b.ne .LBB0_10
	mov x21, x1
	mov x19, x2
	mov w8, #23
.LBB0_7:
	mov x20, x0
	ldr x0, [x0, #8]
	ldr x8, [x3, x8]
	blr x8
	ldr x8, [x21]
	mov x0, x19
	blr x8
	orr x8, x0, #0x1
	str x1, [x20, #8]
	mov w0, #1
	stlr x8, [x20]
	ldp x20, x19, [sp, #32]
	ldr x21, [sp, #16]
	ldp x29, x30, [sp], #48
	ret
.LBB0_10:
	mov w0, wzr
	ldp x20, x19, [sp, #32]
	ldr x21, [sp, #16]
	ldp x29, x30, [sp], #48
	ret
.LBB0_11:
	mov w0, #1
	ldp x20, x19, [sp, #32]
	ldr x21, [sp, #16]
	ldp x29, x30, [sp], #48
	ret
	adrp x8, :got:spmc_waker::NOOP_VTABLE
	ldr x8, [x8, :got_lo12:spmc_waker::NOOP_VTABLE]
	str xzr, [x20, #8]
	ldr x8, [x8]
	swpal x8, x8, [x20]
	bl _Unwind_Resume
