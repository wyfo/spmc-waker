<spmc_waker::SpmcWaker<spmc_waker::synchronization::Sequential, true, spmc_waker::registration::Unchecked>>::register_impl_cold:
	stp x29, x30, [sp, #-48]!
	stp x22, x21, [sp, #16]
	stp x20, x19, [sp, #32]
	mov x29, sp
	mov x19, x2
	mov x20, x0
	tbnz w19, #0, .LBB1_4
	ldr x21, [x20, #8]
	ldr x22, [x20, #16]
	tbnz w19, #1, .LBB1_11
	ldp x8, x0, [x1]
	add x19, x19, #9
	ldr x8, [x8]
	blr x8
	str x1, [x20, #8]
	str x0, [x20, #16]
	swpal x19, x8, [x20]
	tbnz w8, #1, .LBB1_12
.LBB1_3:
	mov x0, x19
	ldp x20, x19, [sp, #32]
	ldp x22, x21, [sp, #16]
	ldp x29, x30, [sp], #48
	ret
.LBB1_4:
	ldr x0, [x20, #8]
	ldr x8, [x20, #16]
	ldp x22, x21, [x1]
	cmp x0, x21
	b.ne .LBB1_6
	cmp x8, x22
	b.eq .LBB1_3
.LBB1_6:
	add x9, x19, #7
	swp x9, x9, [x20]
	tst x9, #0x3
	b.eq .LBB1_10
	tbz w9, #1, .LBB1_9
	dmb ishld
.LBB1_9:
	ldr x8, [x8, #24]
	blr x8
.LBB1_10:
	ldr x8, [x22]
	mov x0, x21
	add x19, x19, #16
	blr x8
	str x1, [x20, #8]
	str x0, [x20, #16]
	stlr x19, [x20]
	mov x0, x19
	ldp x20, x19, [sp, #32]
	ldp x22, x21, [sp, #16]
	ldp x29, x30, [sp], #48
	ret
.LBB1_11:
	ldp x8, x0, [x1]
	add x19, x19, #7
	ldr x8, [x8]
	blr x8
	str x1, [x20, #8]
	str x0, [x20, #16]
	stlr x19, [x20]
.LBB1_12:
	ldr x8, [x22, #24]
	mov x0, x21
	blr x8
	mov x0, x19
	ldp x20, x19, [sp, #32]
	ldp x22, x21, [sp, #16]
	ldp x29, x30, [sp], #48
	ret
