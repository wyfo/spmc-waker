sc_wake_cold:
	ldr x8, [x0]
	tbnz w8, #0, .LBB1_3
	ldsetl xzr, x8, [x0]
	tbnz w8, #0, .LBB1_3
	ret
	b spmc_waker::SpmcWaker<_,_>::wake_sync_cold
