uc_wake:
	stp x29, x30, [sp, #-32]!
	stp x20, x19, [sp, #16]
	mov x29, sp
	ldar x8, [x0]
	tbz w8, #0, .LBB1_4
	mov w9, #2
	mov x10, x8
	casa x10, x9, [x0]
	cmp x10, x8
	b.ne .LBB1_4
	mov x19, x0
	ldr x0, [x0, #8]
	ldur x9, [x8, #15]
	sub x20, x8, #1
	blr x9
	stlr x20, [x19]
.LBB1_4:
	ldp x20, x19, [sp, #16]
	ldp x29, x30, [sp], #32
	ret
	stlr x20, [x19]
	bl _Unwind_Resume
