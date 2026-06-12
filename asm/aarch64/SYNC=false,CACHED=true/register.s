uc_register:
	mov x9, x1
	ldr x3, [x0]
	ldr x1, [x1]
	ldr x2, [x9, #8]
	mov x8, x0
	cmp x1, x3
	b.ne .LBB1_3
	ldr x9, [x8, #8]
	cmp x2, x9
	b.ne .LBB1_3
	orr x9, x3, #0x1
	mov w0, #1
	stlr x9, [x8]
	ret
.LBB1_3:
	mov x0, x8
	b spmc_waker::SpmcWaker<_,_>::overwrite
