asm_poll_wait_until_asm:
	stp x29, x30, [sp, #-32]!
	stp x20, x19, [sp, #16]
	mov x29, sp
	ldrb w8, [x2]
	cbz w8, .LBB10_2
.LBB10_1:
	mov w0, wzr
	ldp x20, x19, [sp, #16]
	ldp x29, x30, [sp], #32
	ret
	mov x19, x2
	ldr x1, [x1]
	ldar x2, [x0]
	tst x2, #0x3
	b.ne .LBB10_7
	ldp x9, x8, [x1]
	mov x20, x0
	ldr x9, [x9]
	mov x0, x8
	blr x9
	str x1, [x20, #8]
	add x9, x0, #1
	swpal x9, x10, [x20]
	ldrb w10, [x19]
	cbz w10, .LBB10_6
	and x10, x9, #0xfffffffffffffffe
	mov x11, x9
	mov x8, x20
	casa x11, x10, [x20]
	cmp x11, x9
	b.ne .LBB10_1
	ldr x0, [x8, #8]
	ldr x8, [x10, #24]
	blr x8
	mov w0, wzr
	ldp x20, x19, [sp, #16]
	ldp x29, x30, [sp], #32
	ret
	mov w0, #1
	ldp x20, x19, [sp, #16]
	ldp x29, x30, [sp], #32
	ret
.LBB10_7:
	mov x3, x19
	ldp x20, x19, [sp, #16]
	ldp x29, x30, [sp], #32
	b spmc_waker::SpmcWaker<S,_>::poll_wait_until_cold
