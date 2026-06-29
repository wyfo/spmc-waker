asm_has_waker_registered_asm:
	ldr x8, [x0]
	and w0, w8, #0x1
	ret
