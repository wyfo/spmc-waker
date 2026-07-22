asm_unregister_asm:
	ldp x9, x8, [x0]
	add x10, x8, #1
	cas x8, x10, [x9]
	ret
