su_has_waker_registered:
	mov rcx, qword ptr [rdi]
	and ecx, 3
	mov al, 1
	cmp ecx, 1
	je .LBB0_2
	lock or	dword ptr [rsp - 64], 0
	mov rax, qword ptr [rdi]
	and eax, 3
	cmp eax, 1
	sete al
.LBB0_2:
	ret
