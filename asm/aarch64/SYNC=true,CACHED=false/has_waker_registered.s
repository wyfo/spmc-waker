su_has_waker_registered:
	ldr x8, [x0]
	and x8, x8, #0x3
	cmp x8, #1
	b.ne .LBB0_2
	mov w0, #1
	ret
.LBB0_2:
	ldsetl xzr, x8, [x0]
	and x8, x8, #0x3
	cmp x8, #1
	cset w0, eq
	ret
