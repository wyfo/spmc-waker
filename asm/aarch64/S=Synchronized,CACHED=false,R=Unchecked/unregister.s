asm_unregister_asm:
	ldp x10, x9, [x0]
	sub x11, x9, #1
	mov x12, x9
	ldr x0, [x10, #8]
	ldr x8, [x10, #16]
	cas x12, x11, [x10]
	cmp x12, x9
	b.ne .LBB6_2
	ldr x1, [x8, #24]
	br x1
.LBB6_2:
	ret
