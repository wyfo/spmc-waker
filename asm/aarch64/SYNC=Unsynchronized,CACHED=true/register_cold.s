spmc_waker::SpmcWaker<S,_>::register_cold:
	stp x29, x30, [sp, #-32]!
	stp x20, x19, [sp, #16]
	mov x29, sp
	ldr x8, [x1]
	mov x19, x0
	cbz w4, .LBB0_3
	add x9, x8, #1
	cmp x9, x2
	b.ne .LBB0_3
	ldr x9, [x1, #8]
	ldr x10, [x19, #8]
	cmp x9, x10
	b.eq .LBB0_10
.LBB0_3:
	tbnz w2, #1, .LBB0_11
	tbnz w2, #0, .LBB0_13
.LBB0_5:
	ldr x0, [x1, #8]
	tbz w4, #0, .LBB0_8
	ldr x8, [x8]
	mov x20, x2
	blr x8
	mov x8, x0
	mov x0, x1
	mov x2, x20
.LBB0_8:
	ldr x9, [x19, #8]
	ands x10, x2, #0xfffffffffffffffe
	add x8, x8, #1
	str x0, [x19, #8]
	stlr x8, [x19]
	b.eq .LBB0_10
	ldr x8, [x10, #24]
	mov x0, x9
	blr x8
.LBB0_10:
	mov w0, #1
	ldp x20, x19, [sp, #16]
	ldp x29, x30, [sp], #32
	ret
.LBB0_11:
	tbnz w3, #0, .LBB0_14
.LBB0_12:
	mov w0, wzr
	ldp x20, x19, [sp, #16]
	ldp x29, x30, [sp], #32
	ret
.LBB0_13:
	add x9, x2, #4
	cmp x9, #8
	b.hs .LBB0_15
.LBB0_14:
	mov x8, x1
	mov x0, x19
	mov x1, x2
	mov x2, x8
	mov w3, w4
	ldp x20, x19, [sp, #16]
	ldp x29, x30, [sp], #32
	b spmc_waker::SpmcWaker<S,_>::register_fallback
.LBB0_15:
	mov x9, x2
	adrp x10, .Lanon.87f483a65f76ae2ffc6b20dd16c25c5b.0
	add x10, x10, :lo12:.Lanon.87f483a65f76ae2ffc6b20dd16c25c5b.0
	casa x9, x10, [x19]
	cmp x9, x2
	b.eq .LBB0_5
	cbz w3, .LBB0_12
	tbnz w9, #1, .LBB0_19
	mov x2, x9
	b .LBB0_5
.LBB0_19:
	mov x0, x19
	mov x2, x1
	mov x1, x9
	mov w3, w4
	ldp x20, x19, [sp, #16]
	ldp x29, x30, [sp], #32
	b spmc_waker::SpmcWaker<S,_>::register_fallback
	tbz w20, #0, .LBB0_22
	swpal x20, x8, [x19]
.LBB0_22:
	bl _Unwind_Resume
