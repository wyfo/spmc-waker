uc_unregister:
	ldr x8, [x0]
	and x9, x8, #0x3
	cmp x9, #1
	b.ne .LBB0_2
	sub x9, x8, #1
	mov x10, x8
	cas x10, x9, [x0]
	cmp x10, x8
	cset w0, eq
	ret
.LBB0_2:
	mov w0, wzr
	ret
