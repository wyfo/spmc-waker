spmc_waker::SpmcWaker<S,_>::register_fallback:
	sub sp, sp, #64
	stp x29, x30, [sp, #16]
	str x21, [sp, #32]
	stp x20, x19, [sp, #48]
	add x29, sp, #16
	mov x20, x1
	mov x19, x0
	cbz w3, .LBB2_2
	ldp x8, x0, [x2]
	ldr x8, [x8]
	blr x8
	mov x2, sp
	stp x0, x1, [sp]
	add x8, x20, #4
	cmp x8, #7
	b.hi .LBB2_6
.LBB2_3:
	mov x8, x20
	casa x20, xzr, [x19]
	cmp x20, x8
	b.eq .LBB2_8
	add x8, x20, #4
	cmp x8, #8
	b.lo .LBB2_3
	mov x0, x19
	mov x1, x2
	mov x2, x20
	mov w3, #1
	mov w4, wzr
	bl spmc_waker::SpmcWaker<S,_>::register_cold
	ldp x20, x19, [sp, #48]
	ldr x21, [sp, #32]
	ldp x29, x30, [sp, #16]
	add sp, sp, #64
	ret
.LBB2_6:
	ldp x8, x9, [x2]
	mov w0, #1
	str x9, [x19, #16]
	str x8, [x19, #24]
	mov x8, x20
	casal x8, x0, [x19]
	cmp x8, x20
	b.ne .LBB2_12
	ldp x20, x19, [sp, #48]
	ldr x21, [sp, #32]
	ldp x29, x30, [sp, #16]
	add sp, sp, #64
	ret
.LBB2_8:
	ldr x20, [x19, #16]
	ldr x21, [x19, #24]
	mov x8, xzr
	ldp x9, x10, [x2]
	str x10, [x19, #16]
	str x9, [x19, #24]
	mov w9, #1
	casal x8, x9, [x19]
	cmp x8, #0
	b.ne .LBB2_13
	cbz x21, .LBB2_11
	ldr x8, [x21, #24]
	mov x0, x20
	blr x8
	mov w0, #1
	ldp x20, x19, [sp, #48]
	ldr x21, [sp, #32]
	ldp x29, x30, [sp, #16]
	add sp, sp, #64
	ret
.LBB2_12:
	mov x21, xzr
.LBB2_13:
	mov x0, x19
	mov x1, x2
	mov x2, x8
	mov w3, #1
	mov w4, wzr
	bl spmc_waker::SpmcWaker<S,_>::register_cold
	cbz x21, .LBB2_7
	ldr x8, [x21, #24]
	mov x19, x0
	mov x0, x20
	blr x8
	mov x0, x19
	ldp x20, x19, [sp, #48]
	ldr x21, [sp, #32]
	ldp x29, x30, [sp, #16]
	add sp, sp, #64
	ret
	mov x19, x0
	cbz x21, .LBB2_18
	ldr x8, [x21, #24]
	mov x0, x20
	blr x8
	mov x0, x19
	bl _Unwind_Resume
	bl core::panicking::panic_in_cleanup
