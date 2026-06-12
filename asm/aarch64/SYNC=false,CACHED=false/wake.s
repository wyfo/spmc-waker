uu_wake:
	ldar x9, [x0]
	tbz w9, #0, .LBB1_3
	mov w8, #2
	mov x10, x9
	casa x10, x8, [x0]
	cmp x10, x9
	b.ne .LBB1_3
	sub x10, x9, #1
	ldr x8, [x0, #8]
	stlr x10, [x0]
	ldur x1, [x9, #7]
	mov x0, x8
	br x1
.LBB1_3:
	ret
