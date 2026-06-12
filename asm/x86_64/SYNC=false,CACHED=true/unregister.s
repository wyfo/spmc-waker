uc_unregister:
	mov rax, qword ptr [rdi]
	mov ecx, eax
	and ecx, 3
	cmp ecx, 1
	jne .LBB0_1
	lea rcx, [rax - 1]
	lock cmpxchg	qword ptr [rdi], rcx
	sete al
	ret
.LBB0_1:
	xor eax, eax
	ret
