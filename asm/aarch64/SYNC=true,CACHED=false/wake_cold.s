su_wake_cold:
	ldr x8, [x0]
	and x8, x8, #0x3
	cmp x8, #1
	b.eq .LBB1_3
	ldsetl xzr, x8, [x0]
	and x8, x8, #0x3
	cmp x8, #1
	b.eq .LBB1_3
	ret
.LBB1_3:
	b spmc_waker::SpmcWaker<_,_>::wake_sync_cold
