uu_register:
	stp x29, x30, [sp, #-32]!
	str x19, [sp, #16]
	mov x29, sp
	ldar x3, [x0]
	ldr x8, [x1]
	mov x19, x0
	tst x3, #0x3
	b.ne .LBB1_2
	ldr x8, [x8]
	ldr x0, [x1, #8]
	blr x8
	orr x8, x0, #0x1
	str x1, [x19, #8]
	mov w0, #1
	stlr x8, [x19]
	ldr x19, [sp, #16]
	ldp x29, x30, [sp], #32
	ret
.LBB1_2:
	ldr x2, [x1, #8]
	mov x0, x19
	mov x1, x8
	ldr x19, [sp, #16]
	ldp x29, x30, [sp], #32
	b spmc_waker::SpmcWaker<_,_>::overwrite
