asm_registered_asm:
	ldar x1, [x0]
	tst w1, #0x1
	csel x0, x0, xzr, ne
	ret
