spmc_waker::SpmcWaker<_,_>::wake_sync_cold:
	stp x29, x30, [sp, #-32]!
	stp x20, x19, [sp, #16]
	mov x29, sp
	mov w8, #2
	mov x19, x0
	ldsetal x8, x8, [x0]
	and x9, x8, #0x3
	cmp x9, #1
	b.ne .LBB0_4
	ldr x0, [x19, #8]
	ldur x9, [x8, #15]
	sub x20, x8, #1
	blr x9
	swpl x20, x8, [x19]
	ldp x20, x19, [sp, #16]
	ldp x29, x30, [sp], #32
	ret
.LBB0_4:
	tbnz w8, #1, .LBB0_3
	add x9, x8, #2
	cas x9, x8, [x19]
	ldp x20, x19, [sp, #16]
	ldp x29, x30, [sp], #32
	ret
	swpl x20, x8, [x19]
	bl _Unwind_Resume
