sc_register:
	mov x8, x1
	ldr x3, [x0]
	ldr x1, [x1]
	ldr x2, [x8, #8]
	cmp x1, x3
	b.ne .LBB1_3
	ldr x8, [x0, #8]
	cmp x2, x8
	b.ne .LBB1_3
	orr x8, x3, #0x1
	swpa x8, x8, [x0]
	mov w0, #1
	ret
.LBB1_3:
	b spmc_waker::SpmcWaker<_,_>::overwrite
