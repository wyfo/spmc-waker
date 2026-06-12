uc_has_waker_registered:
	ldar x8, [x0]
	and w0, w8, #0x1
	ret
