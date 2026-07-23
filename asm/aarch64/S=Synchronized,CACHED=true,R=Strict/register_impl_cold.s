<spmc_waker::SpmcWaker<spmc_waker::synchronization::Synchronized, true>>::register_impl_cold:
	stp x29, x30, [sp, #-48]!
	stp x22, x21, [sp, #16]
	stp x20, x19, [sp, #32]
	mov x29, sp
	mov x19, x2
	mov x20, x0
	tbnz w19, #0, .LBB1_4
	tbnz w19, #1, .LBB1_9
	ldp x8, x0, [x1]
	ldr x8, [x8]
	blr x8
	str x1, [x20, #8]
	add x19, x19, #9
	str x0, [x20, #16]
	swpal x19, x8, [x20]
	mov x0, x19
	ldp x20, x19, [sp, #32]
	ldp x22, x21, [sp, #16]
	ldp x29, x30, [sp], #48
	ret
.LBB1_4:
	ldr x21, [x20, #8]
	ldr x22, [x20, #16]
	ldp x8, x0, [x1]
	cmp x21, x0
	b.ne .LBB1_7
	cmp x22, x8
	b.ne .LBB1_7
	swpa x19, x8, [x20]
	mov x0, x19
	ldp x20, x19, [sp, #32]
	ldp x22, x21, [sp, #16]
	ldp x29, x30, [sp], #48
	ret
.LBB1_7:
	ldr x8, [x8]
	blr x8
	str x1, [x20, #8]
	add x19, x19, #8
	b .LBB1_11
.LBB1_9:
	ldr x21, [x20, #8]
	ldr x22, [x20, #16]
	ldp x8, x0, [x1]
	ldr x8, [x8]
	blr x8
	add x19, x19, #7
	str x1, [x20, #8]
.LBB1_11:
	str x0, [x20, #16]
	mov x0, x21
	swpal x19, x8, [x20]
	ldr x8, [x22, #24]
	blr x8
	mov x0, x19
	ldp x20, x19, [sp, #32]
	ldp x22, x21, [sp, #16]
	ldp x29, x30, [sp], #48
	ret
	swpal x19, x8, [x20]
	bl _Unwind_Resume
	swpal x19, x8, [x20]
	bl _Unwind_Resume
	swpal x19, x8, [x20]
	bl _Unwind_Resume
