asm_try_register_asm:
	ldar x2, [x0]
	ldr x8, [x1]
	cmp x8, x2
	b.ne .LBB12_3
	ldr x9, [x1, #8]
	ldr x10, [x0, #8]
	cmp x9, x10
	b.ne .LBB12_3
	add x8, x8, #1
	swpa x8, x8, [x0]
	mov w0, #1
	ret
.LBB12_3:
	mov w3, wzr
	mov w4, #1
	b spmc_waker::SpmcWaker<S,_>::register_cold
