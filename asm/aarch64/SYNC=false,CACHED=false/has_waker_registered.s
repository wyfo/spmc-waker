asm_has_waker_registered_asm:
	ldar x8, [x0]
	and w0, w8, #0x1
	ret
