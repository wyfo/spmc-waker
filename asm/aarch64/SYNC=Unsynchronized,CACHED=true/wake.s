asm_wake_asm:
	stp x29, x30, [sp, #-48]!
	stp x22, x21, [sp, #16]
	stp x20, x19, [sp, #32]
	mov x29, sp
	ldr x8, [x0]
	tbnz w8, #0, .LBB14_2
.LBB14_1:
	ldp x20, x19, [sp, #32]
	ldp x22, x21, [sp, #16]
	ldp x29, x30, [sp], #48
	ret
	tbnz w8, #1, .LBB14_1
	add x21, x8, #2
	mov x9, x8
	casa x9, x21, [x0]
	cmp x9, x8
	b.eq .LBB14_6
	sub x8, x8, #4
	cmn x8, #8
	b.lo .LBB14_1
	mov x8, x9
	tbnz w9, #0, .LBB14_2
	b .LBB14_1
.LBB14_6:
	add x9, x8, #4
	cmp x9, #8
	b.lo .LBB14_1
	ldr x20, [x0, #8]
	mov x22, x0
	ldur x9, [x8, #15]
	sub x19, x8, #1
	mov x0, x20
	blr x9
	mov x1, x21
	casl x1, x19, [x22]
	cmp x1, x21
	b.eq .LBB14_1
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
