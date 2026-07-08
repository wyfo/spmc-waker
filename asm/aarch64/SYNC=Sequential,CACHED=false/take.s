asm_take_asm:
	stp x29, x30, [sp, #-16]!
	mov x29, sp
	ldar x8, [x0]
	tbnz w8, #0, .LBB14_3
.LBB14_1:
	mov x2, xzr
.LBB14_2:
	mov x0, x2
	mov x1, x4
	ldp x29, x30, [sp], #16
	ret
.LBB14_3:
.LBB14_4:
	tbnz w8, #1, .LBB14_1
	add x9, x8, #2
	mov x10, x8
	casa x10, x9, [x0]
	cmp x10, x8
	b.eq .LBB14_8
	sub x8, x8, #4
	mov x2, xzr
	cmn x8, #8
	b.lo .LBB14_11
	mov x8, x10
	tbnz w10, #0, .LBB14_4
	b .LBB14_2
.LBB14_8:
	add x10, x8, #4
	cmp x10, #8
	b.lo .LBB14_1
	sub x2, x8, #1
	mov x1, x9
	ldr x4, [x0, #8]
	casl x1, x2, [x0]
	cmp x1, x9
	b.eq .LBB14_2
	mov x3, x2
	bl spmc_waker::SpmcWaker<S,_>::wake_fallback
	ldp x29, x30, [sp], #16
	ret
.LBB14_11:
	mov x0, x2
	mov x1, x4
	ldp x29, x30, [sp], #16
	ret
