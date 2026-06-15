asm_has_waker_registered_asm:
	mov rcx, qword ptr [rdi]
	mov al, 1
	test cl, 1
	jne .LBB9_2
	lock or	dword ptr [rsp - 64], 0
	mov rax, qword ptr [rdi]
	and al, 1
.LBB9_2:
	ret
