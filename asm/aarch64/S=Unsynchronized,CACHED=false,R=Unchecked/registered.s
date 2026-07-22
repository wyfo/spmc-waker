asm_registered_asm:
	ldr x1, [x0]
	tbnz w1, #0, .LBB3_2
	mov x0, xzr
	ret
.LBB3_2:
	dmb ishld
	ret
