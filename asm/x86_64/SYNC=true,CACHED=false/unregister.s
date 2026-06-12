su_unregister:
	mov rcx, qword ptr [rdi]
	mov eax, ecx
	and eax, 3
	cmp eax, 1
	jne .LBB0_4
	lea rdx, [rcx - 1]
	mov rax, rcx
	lock cmpxchg	qword ptr [rdi], rdx
	jne .LBB0_4
	push rax
	mov rdi, qword ptr [rdi + 8]
	call qword ptr [rcx + 23]
	mov al, 1
	add rsp, 8
	ret
.LBB0_4:
	xor eax, eax
	ret
