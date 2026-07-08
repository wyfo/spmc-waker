asm_take_asm:
	stp x29, x30, [sp, #-16]!
	mov x29, sp
	ldr x8, [x0]
	tst x8, #0x1
	cset w9, eq
	tbnz w8, #0, .LBB14_2
	ldsetl xzr, x8, [x0]
	tbz w8, #0, .LBB14_9
.LBB14_2:
	tbnz w8, #1, .LBB14_6
	add x10, x8, #2
	mov x11, x8
	casal x11, x10, [x0]
	cmp x11, x8
	b.eq .LBB14_8
	sub x8, x8, #4
	cmn x8, #8
	b.lo .LBB14_6
	mov x8, x11
	tbnz w11, #0, .LBB14_2
.LBB14_6:
	tbnz w9, #0, .LBB14_9
	ldsetl xzr, x8, [x0]
	mov w9, #1
	tbnz w8, #0, .LBB14_2
	b .LBB14_9
.LBB14_8:
	add x9, x8, #4
	cmp x9, #8
	b.hs .LBB14_11
.LBB14_9:
	mov x2, xzr
.LBB14_10:
	mov x0, x2
	mov x1, x4
	ldp x29, x30, [sp], #16
	ret
.LBB14_11:
	sub x2, x8, #1
	mov x1, x10
	ldr x4, [x0, #8]
	casl x1, x2, [x0]
	cmp x1, x10
	b.eq .LBB14_10
	mov x3, x2
	bl spmc_waker::SpmcWaker<S,_>::wake_fallback
	ldp x29, x30, [sp], #16
	ret
