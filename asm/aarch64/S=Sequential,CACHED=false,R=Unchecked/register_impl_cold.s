<spmc_waker::SpmcWaker<spmc_waker::synchronization::Sequential, false, spmc_waker::registration::Unchecked>>::register_impl_cold:
	stp x29, x30, [sp, #-48]!
	stp x22, x21, [sp, #16]
	stp x20, x19, [sp, #32]
	mov x29, sp
	mov x20, x0
	ldr x0, [x0, #8]
	mov x19, x2
	ldr x8, [x20, #16]
	ldp x22, x21, [x1]
	cmp x0, x21
	b.ne .LBB0_3
	cmp x8, x22
	b.ne .LBB0_3
	mov x0, x19
	ldp x20, x19, [sp, #32]
	ldp x22, x21, [sp, #16]
	ldp x29, x30, [sp], #48
	ret
.LBB0_3:
	sub x9, x19, #1
	swp x9, x9, [x20]
	tbz w9, #0, .LBB0_5
	ldr x8, [x8, #24]
	blr x8
.LBB0_5:
	ldr x8, [x22]
	mov x0, x21
	add x19, x19, #8
	blr x8
	str x1, [x20, #8]
	str x0, [x20, #16]
	stlr x19, [x20]
	mov x0, x19
	ldp x20, x19, [sp, #32]
	ldp x22, x21, [sp, #16]
	ldp x29, x30, [sp], #48
	ret
