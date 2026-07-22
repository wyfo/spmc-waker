spmc_waker::SpmcWaker<S,_,R>::register_impl_cold:
	stp x29, x30, [sp, #-48]!
	stp x22, x21, [sp, #16]
	stp x20, x19, [sp, #32]
	mov x29, sp
	mov x19, x2
	mov x20, x0
	tbnz w19, #0, .LBB1_4
	ldr x21, [x20, #8]
	ldr x22, [x20, #16]
	tbnz w19, #1, .LBB1_10
	ldp x8, x0, [x1]
	add x19, x19, #9
	ldr x8, [x8]
	blr x8
	str x1, [x20, #8]
	str x0, [x20, #16]
	swpl x19, x8, [x20]
	tbz w8, #1, .LBB1_6
	dmb ishld
	b .LBB1_11
.LBB1_4:
	ldr x0, [x20, #8]
	ldr x8, [x20, #16]
	ldp x22, x21, [x1]
	cmp x0, x21
	b.ne .LBB1_7
	cmp x8, x22
	b.ne .LBB1_7
.LBB1_6:
	mov x0, x19
	ldp x20, x19, [sp, #32]
	ldp x22, x21, [sp, #16]
	ldp x29, x30, [sp], #48
	ret
.LBB1_7:
	sub x9, x19, #1
	swp x9, x9, [x20]
	tbnz w9, #0, .LBB1_13
	tbnz w9, #1, .LBB1_12
	orr x10, x9, #0x2
	mov x11, x9
	casa x11, x10, [x20]
	cmp x11, x9
	b.ne .LBB1_13
	b .LBB1_14
.LBB1_10:
	ldp x8, x0, [x1]
	add x19, x19, #7
	ldr x8, [x8]
	blr x8
	str x1, [x20, #8]
	str x0, [x20, #16]
	stlr x19, [x20]
.LBB1_11:
	ldr x8, [x22, #24]
	mov x0, x21
	blr x8
	mov x0, x19
	ldp x20, x19, [sp, #32]
	ldp x22, x21, [sp, #16]
	ldp x29, x30, [sp], #48
	ret
.LBB1_12:
	dmb ishld
.LBB1_13:
	ldr x8, [x8, #24]
	blr x8
.LBB1_14:
	ldr x8, [x22]
	mov x0, x21
	add x19, x19, #8
	blr x8
	str x1, [x20, #8]
	str x0, [x20, #16]
	stlr x19, [x20]
	mov x0, x19
	ldp x20, x19, [sp, #32]
	ldp x22, x21, [sp, #16]
	ldp x29, x30, [sp], #48
	ret
