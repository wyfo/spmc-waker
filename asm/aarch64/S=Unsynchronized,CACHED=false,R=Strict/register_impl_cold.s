spmc_waker::SpmcWaker<S,_,R>::register_impl_cold:
	stp x29, x30, [sp, #-48]!
	stp x22, x21, [sp, #16]
	stp x20, x19, [sp, #32]
	mov x29, sp
	mov x20, x0
	ldr x21, [x0, #8]
	ldr x22, [x0, #16]
	ldp x8, x0, [x1]
	mov x19, x2
	cmp x21, x0
	b.ne .LBB0_3
	cmp x22, x8
	b.ne .LBB0_3
	stlr x19, [x20]
	mov x0, x19
	ldp x20, x19, [sp, #32]
	ldp x22, x21, [sp, #16]
	ldp x29, x30, [sp], #48
	ret
.LBB0_3:
	ldr x8, [x8]
	blr x8
	add x19, x19, #8
	str x1, [x20, #8]
	str x0, [x20, #16]
	mov x0, x21
	stlr x19, [x20]
	ldr x8, [x22, #24]
	blr x8
	mov x0, x19
	ldp x20, x19, [sp, #32]
	ldp x22, x21, [sp, #16]
	ldp x29, x30, [sp], #48
	ret
	swpal x19, x8, [x20]
	bl _Unwind_Resume
