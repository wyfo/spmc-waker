spmc_waker::SpmcWaker<S,_>::register_cold:
	sub sp, sp, #48
	stp x29, x30, [sp, #16]
	stp x20, x19, [sp, #32]
	add x29, sp, #16
	mov x10, x1
	ldr x8, [x10], #8
	cbz w4, .LBB0_3
	add x9, x8, #1
	cmp x9, x2
	b.ne .LBB0_3
	ldr x9, [x10]
	ldr x11, [x0, #8]
	cmp x9, x11
	b.eq .LBB0_16
.LBB0_3:
	tbnz w2, #1, .LBB0_10
	tbnz w2, #0, .LBB0_12
.LBB0_5:
	tbz w4, #0, .LBB0_8
	mov x19, x0
	ldr x0, [x10]
	mov x20, x2
	ldr x8, [x8]
	blr x8
	mov x8, x0
	add x10, sp, #8
	mov x2, x20
	mov x0, x19
	str x1, [sp, #8]
.LBB0_8:
	tbnz w2, #0, .LBB0_14
	ldr x9, [x10]
	add x8, x8, #1
	str x9, [x0, #8]
	swpal x8, x8, [x0]
	mov w0, #1
	ldp x20, x19, [sp, #32]
	ldp x29, x30, [sp, #16]
	add sp, sp, #48
	ret
.LBB0_10:
	tbnz w3, #0, .LBB0_13
.LBB0_11:
	mov w0, wzr
	ldp x20, x19, [sp, #32]
	ldp x29, x30, [sp, #16]
	add sp, sp, #48
	ret
.LBB0_12:
	add x9, x2, #4
	cmp x9, #8
	b.hs .LBB0_17
.LBB0_13:
	mov x8, x1
	mov x1, x2
	mov x2, x8
	mov w3, w4
	ldp x20, x19, [sp, #32]
	ldp x29, x30, [sp, #16]
	add sp, sp, #48
	b spmc_waker::SpmcWaker<S,_>::register_fallback
.LBB0_14:
	ldr x10, [x10]
	ldr x9, [x0, #8]
	add x8, x8, #1
	cmp x2, #1
	str x10, [x0, #8]
	swpal x8, x8, [x0]
	b.eq .LBB0_16
	ldur x8, [x2, #23]
	mov x0, x9
	blr x8
.LBB0_16:
	mov w0, #1
	ldp x20, x19, [sp, #32]
	ldp x29, x30, [sp, #16]
	add sp, sp, #48
	ret
.LBB0_17:
	mov x9, x2
	adrp x11, .Lanon.ad1dd373b0eeb19e0c56a132ea2d2936.0
	add x11, x11, :lo12:.Lanon.ad1dd373b0eeb19e0c56a132ea2d2936.0
	casa x9, x11, [x0]
	cmp x9, x2
	b.eq .LBB0_5
	cbz w3, .LBB0_11
	tbnz w9, #1, .LBB0_21
	mov x2, x9
	b .LBB0_5
.LBB0_21:
	mov x2, x1
	mov x1, x9
	mov w3, w4
	ldp x20, x19, [sp, #32]
	ldp x29, x30, [sp, #16]
	add sp, sp, #48
	b spmc_waker::SpmcWaker<S,_>::register_fallback
	tbz w20, #0, .LBB0_24
	swpal x20, x8, [x19]
.LBB0_24:
	bl _Unwind_Resume
