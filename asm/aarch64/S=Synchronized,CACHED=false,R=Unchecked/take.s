asm_take_asm:
	ldr x9, [x0]
	mov x8, x0
	mov x10, x9
	tbnz w9, #0, .LBB5_2
	ldsetl xzr, x10, [x8]
	tbz w10, #0, .LBB5_7
.LBB5_2:
	dmb ishld
	sub x11, x10, #1
	mov x12, x10
	ldr x1, [x8, #8]
	ldr x0, [x8, #16]
	casl x12, x11, [x8]
	cmp x12, x10
	b.ne .LBB5_4
.LBB5_3:
	ret
.LBB5_4:
	tbz w9, #0, .LBB5_7
	ldsetl xzr, x9, [x8]
	tbz w9, #0, .LBB5_7
	dmb ishld
	sub x10, x9, #1
	mov x11, x9
	ldr x1, [x8, #8]
	ldr x0, [x8, #16]
	casl x11, x10, [x8]
	cmp x11, x9
	b.eq .LBB5_3
.LBB5_7:
	mov x0, xzr
	ret
