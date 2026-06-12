sc_has_waker_registered:
	mov rcx, qword ptr [rdi]
	mov al, 1
	test cl, 1
	jne .LBB0_2
	lock or	dword ptr [rsp - 64], 0
	mov rax, qword ptr [rdi]
.LBB0_2:
	and al, 1
	ret
