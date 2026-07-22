asm_unregister_asm:
	add x8, x1, #1
	cas x1, x8, [x0]
	ret
