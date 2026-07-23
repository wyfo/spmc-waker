<spmc_waker::SpmcWaker<spmc_waker::synchronization::Synchronized, false, spmc_waker::registration::Unchecked>>::wake_impl_cold:
	sub x10, x1, #1
	mov x11, x1
	ldr x8, [x0, #8]
	ldr x9, [x0, #16]
	casl x11, x10, [x0]
	cmp x11, x1
	b.ne .LBB0_2
.LBB0_1:
	ldr x1, [x9, #8]
	mov x0, x8
	br x1
.LBB0_2:
	tbnz w2, #0, .LBB0_5
	ldsetl xzr, x10, [x0]
	tbz w10, #0, .LBB0_5
	dmb ishld
	sub x11, x10, #1
	mov x12, x10
	ldr x8, [x0, #8]
	ldr x9, [x0, #16]
	casl x12, x11, [x0]
	cmp x12, x10
	b.eq .LBB0_1
.LBB0_5:
	ret
