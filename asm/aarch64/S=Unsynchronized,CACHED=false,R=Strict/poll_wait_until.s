asm_poll_wait_until_asm:
	stp x29, x30, [sp, #-48]!
	str x21, [sp, #16]
	stp x20, x19, [sp, #32]
	mov x29, sp
	ldrb w8, [x2]
	cbz w8, .LBB1_2
.LBB1_1:
	mov w0, wzr
	ldp x20, x19, [sp, #32]
	ldr x21, [sp, #16]
	ldp x29, x30, [sp], #48
	ret
.LBB1_2:
	ldr x1, [x1]
	ldr x8, [x0]
.LBB1_3:
	mov x19, x8
	tbnz w19, #2, .LBB1_12
	and x8, x19, #0xfffffffffffffff8
	orr x9, x8, #0x4
	mov x8, x19
	casa x8, x9, [x0]
	cmp x8, x19
	b.ne .LBB1_3
	mov x20, x2
	tbnz w19, #0, .LBB1_10
	mov x21, x0
	ldp x8, x0, [x1]
	ldr x8, [x8]
	blr x8
	mov x8, x0
	add x0, x19, #9
	str x1, [x21, #8]
	str x8, [x21, #16]
	stlr x0, [x21]
	ldrb w8, [x20]
	cbz w8, .LBB1_11
.LBB1_8:
	sub x10, x0, #1
	mov x11, x0
	ldr x8, [x21, #8]
	ldr x9, [x21, #16]
	cas x11, x10, [x21]
	cmp x11, x0
	b.ne .LBB1_1
	ldr x9, [x9, #24]
	mov x0, x8
	blr x9
	mov w0, wzr
	ldp x20, x19, [sp, #32]
	ldr x21, [sp, #16]
	ldp x29, x30, [sp], #48
	ret
.LBB1_10:
	mov x2, x19
	str x0, [x29, #24]
	bl <spmc_waker::SpmcWaker<spmc_waker::synchronization::Unsynchronized>>::register_impl_cold
	ldr x21, [x29, #24]
	ldrb w8, [x20]
	cbnz w8, .LBB1_8
.LBB1_11:
	mov w0, #1
	ldp x20, x19, [sp, #32]
	ldr x21, [sp, #16]
	ldp x29, x30, [sp], #48
	ret
.LBB1_12:
	adrp x0, .Lanon.40943f74b53f8fa4249390633cadaabd.0
	add x0, x0, :lo12:.Lanon.40943f74b53f8fa4249390633cadaabd.0
	adrp x2, .Lanon.40943f74b53f8fa4249390633cadaabd.2
	add x2, x2, :lo12:.Lanon.40943f74b53f8fa4249390633cadaabd.2
	mov w1, #47
	bl core::panicking::panic_fmt
	swpal x19, x8, [x21]
	bl _Unwind_Resume
