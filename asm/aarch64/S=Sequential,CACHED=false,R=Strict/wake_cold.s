asm_wake_cold_asm:
	ldar x9, [x0]
	tbnz w9, #0, .LBB7_2
.LBB7_1:
	ret
.LBB7_2:
	sub x11, x9, #1
	mov x12, x9
	ldr x8, [x0, #8]
	ldr x10, [x0, #16]
	casl x12, x11, [x0]
	cmp x12, x9
	b.ne .LBB7_1
	ldr x1, [x10, #8]
	mov x0, x8
	br x1
