asm_take_asm:
	ldar x8, [x0]
	tbz w8, #0, .LBB5_2
	sub x9, x8, #1
	mov x10, x8
	ldr x1, [x0, #8]
	ldr x11, [x0, #16]
	casl x10, x9, [x0]
	cmp x10, x8
	csel x0, x11, xzr, eq
	ret
.LBB5_2:
	mov x0, xzr
	ret
