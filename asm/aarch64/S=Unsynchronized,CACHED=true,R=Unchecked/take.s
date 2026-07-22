asm_take_asm:
	ldr x8, [x0]
	tbnz w8, #0, .LBB5_2
	mov x0, xzr
	ret
.LBB5_2:
	dmb ishld
	sub x9, x8, #1
	mov x10, x8
	ldr x1, [x0, #8]
	ldr x11, [x0, #16]
	casl x10, x9, [x0]
	cmp x10, x8
	csel x0, x11, xzr, eq
	ret
