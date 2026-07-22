asm_registered_asm:
	ldr x10, [x0]
	mov x9, x10
	tbnz w10, #0, .LBB4_3
	ldsetl xzr, x9, [x0]
	tbnz w9, #0, .LBB4_3
	mov w10, #2
	strb w10, [x8, #16]
	ret
.LBB4_3:
	and x10, x10, #0x1
	dmb ishld
	stp x0, x9, [x8]
	eor w10, w10, #0x1
	strb w10, [x8, #16]
	ret
