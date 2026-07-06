spmc_waker::SpmcWaker<S,_>::wake_registered_cold:
	stp x29, x30, [sp, #-48]!
	stp x22, x21, [sp, #16]
	stp x20, x19, [sp, #32]
	mov x29, sp
.LBB4_1:
	tbnz w1, #1, .LBB4_8
	add x21, x1, #2
	mov x8, x1
	casa x8, x21, [x0]
	cmp x8, x1
	b.eq .LBB4_5
	sub x9, x1, #4
	cmn x9, #8
	b.lo .LBB4_8
	mov x1, x8
	tbnz w8, #0, .LBB4_1
	b .LBB4_8
.LBB4_5:
	add x8, x1, #4
	cmp x8, #8
	b.lo .LBB4_8
	ldr x20, [x0, #8]
	mov x22, x0
	ldur x8, [x1, #15]
	sub x19, x1, #1
	mov x0, x20
	blr x8
	mov x1, x21
	casl x1, x19, [x22]
	cmp x1, x21
	b.ne .LBB4_9
.LBB4_8:
	mov w0, wzr
	ldp x20, x19, [sp, #32]
	ldp x22, x21, [sp, #16]
	ldp x29, x30, [sp], #48
	ret
.LBB4_9:
	mov x0, x22
	mov x2, x19
	mov x3, x19
	mov x4, x20
	ldp x20, x19, [sp, #32]
	ldp x22, x21, [sp, #16]
	ldp x29, x30, [sp], #48
	b spmc_waker::SpmcWaker<S,_>::wake_fallback
	swpal x19, x8, [x22]
	bl _Unwind_Resume
