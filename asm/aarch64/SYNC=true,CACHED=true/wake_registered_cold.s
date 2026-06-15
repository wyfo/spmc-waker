spmc_waker::SpmcWaker<_,_>::wake_registered_cold:
	stp x29, x30, [sp, #-48]!
	stp x22, x21, [sp, #16]
	stp x20, x19, [sp, #32]
	mov x29, sp
	mov x19, x0
	tbnz w1, #1, .LBB4_5
	add x22, x1, #2
	mov x8, x1
	casal x8, x22, [x19]
	cmp x8, x1
	b.eq .LBB4_7
	sub x9, x1, #4
	cmn x9, #8
	b.lo .LBB4_5
	mov x1, x8
	tbnz w8, #0, .LBB4_1
.LBB4_5:
	tbnz w2, #0, .LBB4_10
	ldsetl xzr, x1, [x19]
	mov w2, #1
	tbnz w1, #0, .LBB4_1
	b .LBB4_10
.LBB4_7:
	add x8, x1, #4
	cmp x8, #8
	b.lo .LBB4_10
	ldr x21, [x19, #8]
	ldur x8, [x1, #15]
	sub x20, x1, #1
	mov x0, x21
	blr x8
	mov x1, x22
	casl x1, x20, [x19]
	cmp x1, x22
	b.ne .LBB4_11
.LBB4_10:
	ldp x20, x19, [sp, #32]
	ldp x22, x21, [sp, #16]
	ldp x29, x30, [sp], #48
	ret
.LBB4_11:
	mov x0, x19
	mov x2, x20
	mov x3, x20
	mov x4, x21
	ldp x20, x19, [sp, #32]
	ldp x22, x21, [sp, #16]
	ldp x29, x30, [sp], #48
	b spmc_waker::SpmcWaker<_,_>::wake_fallback
	swpal x20, x8, [x19]
	bl _Unwind_Resume
