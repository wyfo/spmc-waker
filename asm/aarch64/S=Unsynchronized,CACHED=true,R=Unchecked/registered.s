asm_registered_asm:
	ldr x1, [x0]
	tbnz w1, #0, .LBB4_2
	mov x0, xzr
	ret
.LBB4_2:
	dmb ishld
	ret
