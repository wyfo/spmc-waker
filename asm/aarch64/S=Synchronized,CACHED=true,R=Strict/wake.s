asm_wake_asm:
	stp x29, x30, [sp, #-64]!
	str x23, [sp, #16]
	stp x22, x21, [sp, #32]
	stp x20, x19, [sp, #48]
	mov x29, sp
	ldr x8, [x0]
	mov x19, x0
	mov x21, x8
	tbnz w8, #0, .LBB6_2
	ldsetl xzr, x21, [x19]
	tbz w21, #0, .LBB6_9
.LBB6_2:
	dmb ishld
	sub x23, x21, #1
	mov x9, x21
	ldr x20, [x19, #8]
	ldr x22, [x19, #16]
	casl x9, x23, [x19]
	cmp x9, x21
	b.ne .LBB6_6
.LBB6_3:
	ldr x8, [x22, #16]
	mov x0, x20
	blr x8
	add x8, x21, #1
	mov x9, x23
	casl x9, x8, [x19]
	cmp x9, x23
	b.eq .LBB6_9
	ldr x1, [x22, #24]
	mov x0, x20
	ldp x20, x19, [sp, #48]
	ldr x23, [sp, #16]
	ldp x22, x21, [sp, #32]
	ldp x29, x30, [sp], #64
	br x1
.LBB6_6:
	tbz w8, #0, .LBB6_9
	ldsetl xzr, x21, [x19]
	tbz w21, #0, .LBB6_9
	dmb ishld
	sub x23, x21, #1
	mov x8, x21
	ldr x20, [x19, #8]
	ldr x22, [x19, #16]
	casl x8, x23, [x19]
	cmp x8, x21
	b.eq .LBB6_3
.LBB6_9:
	ldp x20, x19, [sp, #48]
	ldr x23, [sp, #16]
	ldp x22, x21, [sp, #32]
	ldp x29, x30, [sp], #64
	ret
	ldr x8, [x22, #24]
	mov x19, x0
	mov x0, x20
	blr x8
	mov x0, x19
	bl _Unwind_Resume
	bl core::panicking::panic_in_cleanup
