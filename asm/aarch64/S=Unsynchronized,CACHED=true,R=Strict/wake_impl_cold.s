<spmc_waker::SpmcWaker<spmc_waker::synchronization::Unsynchronized, true>>::wake_impl_cold:
	stp x29, x30, [sp, #-64]!
	str x23, [sp, #16]
	stp x22, x21, [sp, #32]
	stp x20, x19, [sp, #48]
	mov x29, sp
	sub x23, x1, #1
	mov x8, x1
	ldr x19, [x0, #8]
	ldr x22, [x0, #16]
	casl x8, x23, [x0]
	cmp x8, x1
	b.ne .LBB0_3
	mov x21, x1
	mov x20, x0
	ldr x8, [x22, #16]
	mov x0, x19
	blr x8
	add x8, x21, #1
	mov x9, x23
	casl x9, x8, [x20]
	cmp x9, x23
	b.ne .LBB0_4
.LBB0_3:
	ldp x20, x19, [sp, #48]
	ldr x23, [sp, #16]
	ldp x22, x21, [sp, #32]
	ldp x29, x30, [sp], #64
	ret
.LBB0_4:
	ldr x1, [x22, #24]
	mov x0, x19
	ldp x20, x19, [sp, #48]
	ldr x23, [sp, #16]
	ldp x22, x21, [sp, #32]
	ldp x29, x30, [sp], #64
	br x1
	ldr x8, [x22, #24]
	mov x20, x0
	mov x0, x19
	blr x8
	mov x0, x20
	bl _Unwind_Resume
	bl core::panicking::panic_in_cleanup
