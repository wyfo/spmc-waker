asm_register_asm:
	ldar x2, [x0]
	tbz w2, #1, .LBB3_4
	ldr x9, [x0, #8]
	ldr x8, [x0, #16]
	ldr x10, [x1, #8]
	cmp x9, x10
	b.ne .LBB3_4
	ldr x9, [x1]
	cmp x8, x9
	b.ne .LBB3_4
	add x8, x2, #7
	swpa x8, x8, [x0]
	ret
.LBB3_4:
	b spmc_waker::SpmcWaker<S,_,R>::register_impl_cold
