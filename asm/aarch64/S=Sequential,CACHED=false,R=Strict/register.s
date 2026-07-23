asm_register_asm:
	stp x29, x30, [sp, #-32]!
	stp x20, x19, [sp, #16]
	mov x29, sp
	ldr x8, [x0]
	mov x19, x0
.LBB2_1:
	mov x20, x8
	tbnz w20, #2, .LBB2_7
	and x8, x20, #0xfffffffffffffff8
	orr x9, x8, #0x4
	mov x8, x20
	casa x8, x9, [x19]
	cmp x8, x20
	b.ne .LBB2_1
	tbnz w20, #0, .LBB2_6
	ldp x8, x0, [x1]
	ldr x8, [x8]
	blr x8
	add x8, x20, #9
	str x1, [x19, #8]
	str x0, [x19, #16]
	stlr x8, [x19]
	ldp x20, x19, [sp, #16]
	ldp x29, x30, [sp], #32
	ret
.LBB2_6:
	mov x0, x19
	mov x2, x20
	ldp x20, x19, [sp, #16]
	ldp x29, x30, [sp], #32
	b <spmc_waker::SpmcWaker<spmc_waker::synchronization::Sequential>>::register_impl_cold
.LBB2_7:
	adrp x0, .Lanon.40943f74b53f8fa4249390633cadaabd.0
	add x0, x0, :lo12:.Lanon.40943f74b53f8fa4249390633cadaabd.0
	adrp x2, .Lanon.40943f74b53f8fa4249390633cadaabd.2
	add x2, x2, :lo12:.Lanon.40943f74b53f8fa4249390633cadaabd.2
	mov w1, #47
	bl core::panicking::panic_fmt
	swpal x20, x8, [x19]
	bl _Unwind_Resume
