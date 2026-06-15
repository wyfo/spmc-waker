asm_has_waker_registered_asm:
	ldr x8, [x0]
	tbnz w8, #0, .LBB9_2
	ldsetl xzr, x8, [x0]
	and w0, w8, #0x1
	ret
	mov w0, #1
	ret
