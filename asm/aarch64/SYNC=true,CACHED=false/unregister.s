su_unregister:
	ldr x8, [x0]
	and x9, x8, #0x3
	cmp x9, #1
	b.ne .LBB0_3
	sub x9, x8, #1
	mov x10, x8
	cas x10, x9, [x0]
	cmp x10, x8
	b.ne .LBB0_3
	stp x29, x30, [sp, #-16]!
	mov x29, sp
	ldr x0, [x0, #8]
	ldur x8, [x8, #23]
	blr x8
	mov w0, #1
	ldp x29, x30, [sp], #16
	ret
.LBB0_3:
	mov w0, wzr
	ret
