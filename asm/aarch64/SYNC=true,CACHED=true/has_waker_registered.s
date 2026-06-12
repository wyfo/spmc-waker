sc_has_waker_registered:
	ldr x8, [x0]
	tbnz w8, #0, .LBB0_2
	ldsetl xzr, x8, [x0]
	and w0, w8, #0x1
	ret
	mov w8, #1
	and w0, w8, #0x1
	ret
