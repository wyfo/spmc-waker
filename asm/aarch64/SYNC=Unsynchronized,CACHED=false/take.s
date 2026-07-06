asm_take_asm:
	ldr x8, [x0]
	tbnz w8, #0, .LBB14_2
	mov x2, xzr
	mov x0, x2
	mov x1, x4
	ret
.LBB14_2:
	stp x29, x30, [sp, #-16]!
	mov x29, sp
.LBB14_3:
	tbnz w8, #1, .LBB14_8
	add x9, x8, #2
	mov x10, x8
	casa x10, x9, [x0]
	cmp x10, x8
	b.eq .LBB14_7
	sub x8, x8, #4
	mov x2, xzr
	cmn x8, #8
	b.lo .LBB14_11
	mov x8, x10
	tbnz w10, #0, .LBB14_3
	b .LBB14_10
.LBB14_7:
	add x10, x8, #4
	cmp x10, #8
	b.hs .LBB14_9
.LBB14_8:
	mov x2, xzr
	ldp x29, x30, [sp], #16
	mov x0, x2
	mov x1, x4
	ret
.LBB14_9:
	sub x2, x8, #1
	mov x1, x9
	ldr x4, [x0, #8]
	casl x1, x2, [x0]
	cmp x1, x9
	b.ne .LBB14_12
.LBB14_10:
	ldp x29, x30, [sp], #16
	mov x0, x2
	mov x1, x4
	ret
.LBB14_11:
	ldp x29, x30, [sp], #16
	mov x0, x2
	mov x1, x4
	ret
.LBB14_12:
	mov x3, x2
	bl spmc_waker::SpmcWaker<S,_>::wake_fallback
	ldp x29, x30, [sp], #16
	ret
