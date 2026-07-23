asm_poll_wait_until_asm:
	ldrb w8, [x2]
	cbz w8, .LBB1_2
	mov w0, wzr
	ret
.LBB1_2:
	stp x29, x30, [sp, #-48]!
	str x21, [sp, #16]
	stp x20, x19, [sp, #32]
	mov x29, sp
	mov x20, x2
	ldr x1, [x1]
	ldr x2, [x0]
	mov x19, x0
	tbnz w2, #0, .LBB1_7
	ldp x8, x0, [x1]
	add x21, x2, #9
	ldr x8, [x8]
	blr x8
	str x1, [x19, #8]
	str x0, [x19, #16]
	stlr x21, [x19]
	ldrb w8, [x20]
	cbz w8, .LBB1_8
.LBB1_4:
	sub x9, x21, #1
	mov x10, x21
	ldr x0, [x19, #8]
	ldr x8, [x19, #16]
	cas x10, x9, [x19]
	cmp x10, x21
	b.ne .LBB1_6
	ldr x8, [x8, #24]
	blr x8
.LBB1_6:
	mov w0, wzr
	ldp x20, x19, [sp, #32]
	ldr x21, [sp, #16]
	ldp x29, x30, [sp], #48
	ret
.LBB1_7:
	mov x0, x19
	bl <spmc_waker::SpmcWaker<spmc_waker::synchronization::Sequential, false, spmc_waker::registration::Unchecked>>::register_impl_cold
	mov x21, x0
	ldrb w8, [x20]
	cbnz w8, .LBB1_4
.LBB1_8:
	mov w0, #1
	ldp x20, x19, [sp, #32]
	ldr x21, [sp, #16]
	ldp x29, x30, [sp], #48
	ret
