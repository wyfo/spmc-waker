asm_register_asm:
	stp x29, x30, [sp, #-32]!
	stp x20, x19, [sp, #16]
	mov x29, sp
	ldr x2, [x0]
	mov x19, x0
	tbnz w2, #0, .LBB3_2
	ldp x8, x0, [x1]
	add x20, x2, #9
	ldr x8, [x8]
	blr x8
	str x1, [x19, #8]
	str x0, [x19, #16]
	swpal x20, x8, [x19]
	ldp x20, x19, [sp, #16]
	ldp x29, x30, [sp], #32
	ret
.LBB3_2:
	mov x0, x19
	ldp x20, x19, [sp, #16]
	ldp x29, x30, [sp], #32
	b <spmc_waker::SpmcWaker<spmc_waker::synchronization::Synchronized, false, spmc_waker::registration::Unchecked>>::register_impl_cold
