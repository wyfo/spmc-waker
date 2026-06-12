uc_wake_cold:
	ldar x1, [x0]
	tbnz w1, #0, .LBB1_2
	ret
	b spmc_waker::SpmcWaker<_,_>::wake_unsync_cold
