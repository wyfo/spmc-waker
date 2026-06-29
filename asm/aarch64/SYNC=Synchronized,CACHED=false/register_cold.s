spmc_waker::SpmcWaker<S,_>::register_cold:
	stp x29, x30, [sp, #-48]!
	str x21, [sp, #16]
	stp x20, x19, [sp, #32]
	mov x29, sp
	ldr x8, [x1]
	cbz w4, .LBB0_3
	add x9, x8, #1
	cmp x9, x2
	b.ne .LBB0_3
	ldr x9, [x1, #8]
	ldr x10, [x0, #8]
	cmp x9, x10
	b.eq .LBB0_16
.LBB0_3:
	tbnz w2, #1, .LBB0_8
	tbnz w2, #0, .LBB0_10
	mov x20, xzr
	tbz w4, #0, .LBB0_14
	mov x21, x0
	ldr x0, [x1, #8]
	ldr x8, [x8]
	blr x8
	mov x8, x0
	str x1, [x21, #8]
	add x8, x8, #1
	swpal x8, x8, [x21]
	cbnz x20, .LBB0_15
	b .LBB0_16
	tbnz w3, #0, .LBB0_11
	mov w0, wzr
	ldp x20, x19, [sp, #32]
	ldr x21, [sp, #16]
	ldp x29, x30, [sp], #48
	ret
	add x9, x2, #4
	cmp x9, #8
	b.hs .LBB0_12
	mov x8, x1
	mov x1, x2
	mov x2, x8
	mov w3, w4
	ldp x20, x19, [sp, #32]
	ldr x21, [sp, #16]
	ldp x29, x30, [sp], #48
	b spmc_waker::SpmcWaker<S,_>::register_fallback
.LBB0_12:
	mov x9, x2
	adrp x10, .Lanon.40775c54d5b0fec933b8e6dc9cbfb8b4.0
	add x10, x10, :lo12:.Lanon.40775c54d5b0fec933b8e6dc9cbfb8b4.0
	casa x9, x10, [x0]
	cmp x9, x2
	b.ne .LBB0_17
	ldr x19, [x0, #8]
	sub x20, x2, #1
	tbnz w4, #0, .LBB0_6
	ldr x1, [x1, #8]
	str x1, [x0, #8]
	add x8, x8, #1
	swpal x8, x8, [x0]
	cbz x20, .LBB0_16
	ldr x8, [x20, #24]
	mov x0, x19
	blr x8
.LBB0_16:
	mov w0, #1
	ldp x20, x19, [sp, #32]
	ldr x21, [sp, #16]
	ldp x29, x30, [sp], #48
	ret
.LBB0_17:
	tbz w3, #0, .LBB0_9
	tbz w9, #1, .LBB0_5
	mov x2, x1
	mov x1, x9
	mov w3, w4
	ldp x20, x19, [sp, #32]
	ldr x21, [sp, #16]
	ldp x29, x30, [sp], #48
	b spmc_waker::SpmcWaker<S,_>::register_fallback
	mov x9, x20
	mov x20, x0
	cbz x9, .LBB0_22
	ldr x8, [x9, #24]
	mov x0, x19
	blr x8
	mov x0, x20
	bl _Unwind_Resume
	bl core::panicking::panic_in_cleanup
