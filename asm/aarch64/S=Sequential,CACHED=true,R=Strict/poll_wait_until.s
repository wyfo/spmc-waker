asm_poll_wait_until_asm:
	stp x29, x30, [sp, #-32]!
	stp x20, x19, [sp, #16]
	mov x29, sp
	ldrb w8, [x2]
	cbnz w8, .LBB2_9
	ldr x1, [x1]
	ldr x9, [x0]
.LBB2_2:
	mov x8, x9
	tbnz w8, #2, .LBB2_12
	and x9, x8, #0xfffffffffffffff8
	orr x10, x9, #0x4
	mov x9, x8
	casa x9, x10, [x0]
	cmp x9, x8
	b.ne .LBB2_2
	tbz w8, #1, .LBB2_10
	ldr x10, [x0, #8]
	ldr x9, [x0, #16]
	ldr x11, [x1, #8]
	cmp x10, x11
	b.ne .LBB2_10
	ldr x10, [x1]
	cmp x9, x10
	b.ne .LBB2_10
	add x8, x8, #7
	stlr x8, [x0]
	ldrb w9, [x2]
	cbz w9, .LBB2_11
.LBB2_8:
	add x9, x8, #1
	cas x8, x9, [x0]
.LBB2_9:
	mov w0, wzr
	ldp x20, x19, [sp, #16]
	ldp x29, x30, [sp], #32
	ret
.LBB2_10:
	mov x20, x2
	mov x2, x8
	mov x19, x0
	bl spmc_waker::SpmcWaker<S,_,R>::register_impl_cold
	mov x8, x0
	mov x0, x19
	ldrb w9, [x20]
	cbnz w9, .LBB2_8
.LBB2_11:
	mov w0, #1
	ldp x20, x19, [sp, #16]
	ldp x29, x30, [sp], #32
	ret
.LBB2_12:
	adrp x0, .Lanon.8d71c8851bc03468f628dced9b2f7f7b.0
	add x0, x0, :lo12:.Lanon.8d71c8851bc03468f628dced9b2f7f7b.0
	adrp x2, .Lanon.8d71c8851bc03468f628dced9b2f7f7b.2
	add x2, x2, :lo12:.Lanon.8d71c8851bc03468f628dced9b2f7f7b.2
	mov w1, #47
	bl core::panicking::panic_fmt
