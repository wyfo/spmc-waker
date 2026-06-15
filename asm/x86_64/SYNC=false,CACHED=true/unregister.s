asm_unregister_asm:
	mov rax, qword ptr [rdi]
	mov ecx, eax
	and ecx, 3
	cmp ecx, 1
	setne cl
	lea rdx, [rax - 4]
	cmp rdx, -8
	setae dl
	or dl, cl
	jne .LBB13_1
	lea rcx, [rax - 1]
	lock cmpxchg	qword ptr [rdi], rcx
	sete al
	ret
.LBB13_1:
	xor eax, eax
	ret
