asm_unregister_asm:
	ldr x8, [x0]
	and x9, x8, #0x3
	sub x10, x8, #4
	cmp x9, #1
	ccmn x10, #9, #2, eq
	b.hi .LBB17_3
	sub x9, x8, #1
	mov x10, x8
	casa x10, x9, [x0]
	cmp x10, x8
	b.ne .LBB17_3
	stp x29, x30, [sp, #-16]!
	mov x29, sp
	ldr x0, [x0, #8]
	ldur x8, [x8, #23]
	blr x8
	mov w0, #1
	ldp x29, x30, [sp], #16
	ret
.LBB17_3:
	mov w0, wzr
	ret
