asm_wake_asm:
	ldar x9, [x0]
	tbz w9, #0, .LBB5_3
	sub x11, x9, #1
	mov x12, x9
	ldr x8, [x0, #8]
	ldr x10, [x0, #16]
	casl x12, x11, [x0]
	cmp x12, x9
	b.ne .LBB5_3
	ldr x1, [x10, #8]
	mov x0, x8
	br x1
.LBB5_3:
	ret
