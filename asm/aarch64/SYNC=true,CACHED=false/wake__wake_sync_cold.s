spmc_waker::SpmcWaker<_,_>::wake_sync_cold:
	mov w8, #2
	ldsetal x8, x9, [x0]
	and x8, x9, #0x3
	cmp x8, #1
	b.ne .LBB0_2
	sub x10, x9, #1
	ldr x8, [x0, #8]
	swpl x10, x10, [x0]
	ldur x1, [x9, #7]
	mov x0, x8
	br x1
.LBB0_2:
	tbnz w9, #1, .LBB0_4
	add x8, x9, #2
	cas x8, x9, [x0]
	ret
