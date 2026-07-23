asm_wake_cold_asm:
	ldr x8, [x0]
	mov x1, x8
	tbnz w8, #0, .LBB7_3
	ldsetl xzr, x1, [x0]
	tbnz w1, #0, .LBB7_3
	ret
.LBB7_3:
	mov w9, #1
	dmb ishld
	bic w2, w9, w8
	b <spmc_waker::SpmcWaker<spmc_waker::synchronization::Synchronized, false, spmc_waker::registration::Unchecked>>::wake_impl_cold
