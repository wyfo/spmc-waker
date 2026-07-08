asm_unregister_asm:
	mov rcx, qword ptr [rdi]
	mov eax, ecx
	and eax, 3
	cmp eax, 1
	setne al
	lea rdx, [rcx - 4]
	cmp rdx, -8
	setae dl
	or dl, al
	jne .LBB17_4
	lea rdx, [rcx - 1]
	mov rax, rcx
	lock cmpxchg	qword ptr [rdi], rdx
	jne .LBB17_4
	push rax
	mov rdi, qword ptr [rdi + 8]
	call qword ptr [rcx + 23]
	mov al, 1
	add rsp, 8
	ret
.LBB17_4:
	xor eax, eax
	ret
