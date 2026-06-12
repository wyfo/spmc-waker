uu_wake_cold:
	mov rsi, qword ptr [rdi]
	test sil, 1
	jne .LBB1_2
	ret
.LBB1_2:
	jmp qword ptr [rip + spmc_waker::SpmcWaker<_,_>::wake_unsync_cold@GOTPCREL]
