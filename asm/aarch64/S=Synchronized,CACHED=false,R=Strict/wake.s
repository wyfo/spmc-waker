asm_wake_asm:
	ldr x9, [x0]
	mov x11, x9
	tbnz w9, #0, .LBB6_2
	ldsetl xzr, x11, [x0]
	tbz w11, #0, .LBB6_7
.LBB6_2:
	dmb ishld
	sub x12, x11, #1
	mov x13, x11
	ldr x8, [x0, #8]
	ldr x10, [x0, #16]
	casl x13, x12, [x0]
	cmp x13, x11
	b.ne .LBB6_4
.LBB6_3:
	ldr x1, [x10, #8]
	mov x0, x8
	br x1
.LBB6_4:
	tbz w9, #0, .LBB6_7
	ldsetl xzr, x9, [x0]
	tbz w9, #0, .LBB6_7
	dmb ishld
	sub x11, x9, #1
	mov x12, x9
	ldr x8, [x0, #8]
	ldr x10, [x0, #16]
	casl x12, x11, [x0]
	cmp x12, x9
	b.eq .LBB6_3
.LBB6_7:
	ret
