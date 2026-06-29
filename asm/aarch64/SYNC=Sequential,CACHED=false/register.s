asm_register_asm:
	stp x29, x30, [sp, #-32]!
	str x19, [sp, #16]
	mov x29, sp
	ldar x2, [x0]
	mov x19, x0
	tst x2, #0x3
	b.ne .LBB11_2
	ldp x8, x0, [x1]
	ldr x8, [x8]
	blr x8
	add x8, x0, #1
	str x1, [x19, #8]
	stlr x8, [x19]
	ldr x19, [sp, #16]
	ldp x29, x30, [sp], #32
	ret
.LBB11_2:
	mov x0, x19
	mov w3, #1
	mov w4, #1
	ldr x19, [sp, #16]
	ldp x29, x30, [sp], #32
	b spmc_waker::SpmcWaker<S,_>::register_cold
