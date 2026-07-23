asm_register_asm:
	stp x29, x30, [sp, #-16]!
	mov x29, sp
	ldr x8, [x0]
.LBB3_1:
	mov x2, x8
	tbnz w2, #2, .LBB3_8
	and x8, x2, #0xfffffffffffffff8
	orr x9, x8, #0x4
	mov x8, x2
	casa x8, x9, [x0]
	cmp x8, x2
	b.ne .LBB3_1
	tbz w2, #1, .LBB3_7
	ldr x9, [x0, #8]
	ldr x8, [x0, #16]
	ldr x10, [x1, #8]
	cmp x9, x10
	b.ne .LBB3_7
	ldr x9, [x1]
	cmp x8, x9
	b.ne .LBB3_7
	add x8, x2, #7
	stlr x8, [x0]
	ldp x29, x30, [sp], #16
	ret
.LBB3_7:
	ldp x29, x30, [sp], #16
	b <spmc_waker::SpmcWaker<spmc_waker::synchronization::Unsynchronized, true>>::register_impl_cold
.LBB3_8:
	adrp x0, .Lanon.40943f74b53f8fa4249390633cadaabd.0
	add x0, x0, :lo12:.Lanon.40943f74b53f8fa4249390633cadaabd.0
	adrp x2, .Lanon.40943f74b53f8fa4249390633cadaabd.2
	add x2, x2, :lo12:.Lanon.40943f74b53f8fa4249390633cadaabd.2
	mov w1, #47
	bl core::panicking::panic_fmt
