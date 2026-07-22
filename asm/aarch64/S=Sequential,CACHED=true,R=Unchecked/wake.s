asm_wake_asm:
	stp x29, x30, [sp, #-64]!
	str x23, [sp, #16]
	stp x22, x21, [sp, #32]
	stp x20, x19, [sp, #48]
	mov x29, sp
	ldar x20, [x0]
	tbz w20, #0, .LBB6_4
	sub x22, x20, #1
	mov x8, x20
	ldr x19, [x0, #8]
	ldr x21, [x0, #16]
	casl x8, x22, [x0]
	cmp x8, x20
	b.ne .LBB6_4
	mov x23, x0
	ldr x8, [x21, #16]
	mov x0, x19
	blr x8
	add x8, x20, #1
	mov x9, x22
	casl x9, x8, [x23]
	cmp x9, x22
	b.ne .LBB6_5
.LBB6_4:
	ldp x20, x19, [sp, #48]
	ldr x23, [sp, #16]
	ldp x22, x21, [sp, #32]
	ldp x29, x30, [sp], #64
	ret
.LBB6_5:
	ldr x1, [x21, #24]
	mov x0, x19
	ldp x20, x19, [sp, #48]
	ldr x23, [sp, #16]
	ldp x22, x21, [sp, #32]
	ldp x29, x30, [sp], #64
	br x1
	ldr x8, [x21, #24]
	mov x20, x0
	mov x0, x19
	blr x8
	mov x0, x20
	bl _Unwind_Resume
	bl core::panicking::panic_in_cleanup
