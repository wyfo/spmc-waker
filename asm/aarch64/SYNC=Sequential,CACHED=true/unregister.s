asm_unregister_asm:
	ldr x8, [x0]
	and x9, x8, #0x3
	sub x10, x8, #4
	cmp x9, #1
	ccmn x10, #9, #2, eq
	b.ls .LBB13_2
	mov w0, wzr
	ret
.LBB13_2:
	sub x9, x8, #1
	mov x10, x8
	cas x10, x9, [x0]
	cmp x10, x8
	cset w0, eq
	ret
