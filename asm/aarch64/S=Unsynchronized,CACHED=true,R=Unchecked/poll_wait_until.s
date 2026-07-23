asm_poll_wait_until_asm:
	ldrb w8, [x2]
	cbz w8, .LBB2_2
	mov w0, wzr
	ret
.LBB2_2:
	stp x29, x30, [sp, #-32]!
	stp x20, x19, [sp, #16]
	mov x29, sp
	ldr x1, [x1]
	ldar x8, [x0]
	tbz w8, #1, .LBB2_7
	ldr x10, [x0, #8]
	ldr x9, [x0, #16]
	ldr x11, [x1, #8]
	cmp x10, x11
	b.ne .LBB2_7
	ldr x10, [x1]
	cmp x9, x10
	b.ne .LBB2_7
	add x8, x8, #7
	stlr x8, [x0]
	ldrb w9, [x2]
	cbz w9, .LBB2_8
.LBB2_6:
	add x9, x8, #1
	cas x8, x9, [x0]
	mov w0, wzr
	ldp x20, x19, [sp, #16]
	ldp x29, x30, [sp], #32
	ret
.LBB2_7:
	mov x20, x2
	mov x2, x8
	mov x19, x0
	bl <spmc_waker::SpmcWaker<spmc_waker::synchronization::Unsynchronized, true, spmc_waker::registration::Unchecked>>::register_impl_cold
	mov x8, x0
	mov x0, x19
	ldrb w9, [x20]
	cbnz w9, .LBB2_6
.LBB2_8:
	mov w0, #1
	ldp x20, x19, [sp, #16]
	ldp x29, x30, [sp], #32
	ret
