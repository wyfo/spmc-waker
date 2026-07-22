asm_unregister_asm:
	mov x8, x0
	sub x10, x1, #1
	mov x11, x1
	ldr x0, [x0, #8]
	ldr x9, [x8, #16]
	cas x11, x10, [x8]
	cmp x11, x1
	b.ne .LBB5_2
	ldr x1, [x9, #24]
	br x1
.LBB5_2:
	ret
