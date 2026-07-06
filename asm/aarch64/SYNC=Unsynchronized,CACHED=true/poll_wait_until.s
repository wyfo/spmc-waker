asm_poll_wait_until_asm:
	ldrb w8, [x2]
	cbz w8, .LBB10_2
	mov w0, wzr
	ret
.LBB10_2:
	ldr x1, [x1]
	mov x3, x2
	ldar x2, [x0]
	ldr x8, [x1]
	cmp x8, x2
	b.ne .LBB10_7
	ldr x9, [x1, #8]
	ldr x10, [x0, #8]
	cmp x9, x10
	b.ne .LBB10_7
	add x9, x8, #1
	stlr x9, [x0]
	ldrb w10, [x3]
	cbz w10, .LBB10_6
	cas x9, x8, [x0]
	mov w0, wzr
	ret
.LBB10_6:
	mov w0, #1
	ret
.LBB10_7:
	b spmc_waker::SpmcWaker<S,_>::poll_wait_until_cold
