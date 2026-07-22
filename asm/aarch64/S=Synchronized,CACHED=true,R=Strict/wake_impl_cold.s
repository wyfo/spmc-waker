spmc_waker::SpmcWaker<S,_,R>::wake_impl_cold:
	stp x29, x30, [sp, #-64]!
	str x23, [sp, #16]
	stp x22, x21, [sp, #32]
	stp x20, x19, [sp, #48]
	mov x29, sp
	sub x23, x1, #1
	mov x8, x1
	ldr x21, [x0, #8]
	ldr x22, [x0, #16]
	casl x8, x23, [x0]
	mov x19, x0
	cmp x8, x1
	b.ne .LBB0_5
	mov x20, x1
.LBB0_2:
	ldr x8, [x22, #16]
	mov x0, x21
	blr x8
	add x8, x20, #1
	mov x9, x23
	casl x9, x8, [x19]
	cmp x9, x23
	b.eq .LBB0_8
	ldr x1, [x22, #24]
	mov x0, x21
	ldp x20, x19, [sp, #48]
	ldr x23, [sp, #16]
	ldp x22, x21, [sp, #32]
	ldp x29, x30, [sp], #64
	br x1
.LBB0_5:
	tbnz w2, #0, .LBB0_8
	ldsetl xzr, x20, [x19]
	tbz w20, #0, .LBB0_8
	dmb ishld
	sub x23, x20, #1
	mov x8, x20
	ldr x21, [x19, #8]
	ldr x22, [x19, #16]
	casl x8, x23, [x19]
	cmp x8, x20
	b.eq .LBB0_2
.LBB0_8:
	ldp x20, x19, [sp, #48]
	ldr x23, [sp, #16]
	ldp x22, x21, [sp, #32]
	ldp x29, x30, [sp], #64
	ret
	ldr x8, [x22, #24]
	mov x19, x0
	mov x0, x21
	blr x8
	mov x0, x19
	bl _Unwind_Resume
	bl core::panicking::panic_in_cleanup
