asm_try_register_asm:
	ldar x2, [x0]
	ldr x9, [x1]
	mov x8, x0
	cmp x9, x2
	b.ne .LBB12_3
	ldr x10, [x1, #8]
	ldr x11, [x8, #8]
	cmp x10, x11
	b.ne .LBB12_3
	add x9, x9, #1
	mov w0, #1
	stlr x9, [x8]
	ret
.LBB12_3:
	mov x0, x8
	mov w3, wzr
	mov w4, #1
	b spmc_waker::SpmcWaker<S,_>::register_cold
