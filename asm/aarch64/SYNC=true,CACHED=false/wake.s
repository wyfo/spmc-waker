su_wake:
	mov w8, #2
	ldsetl x8, x9, [x0]
	tbnz w9, #1, .LBB1_3
	tbnz w9, #0, .LBB1_4
	add x8, x9, #2
	cas x8, x9, [x0]
	ret
	dmb ishld
	sub x10, x9, #1
	ldr x8, [x0, #8]
	swpl x10, x10, [x0]
	ldur x1, [x9, #7]
	mov x0, x8
	br x1
