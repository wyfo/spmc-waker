uu_wake:
	mov rcx, qword ptr [rdi]
	mov eax, ecx
	and eax, 3
	cmp eax, 1
	jne .LBB1_2
	lea rdx, [rcx + 2]
	mov rax, rcx
	lock cmpxchg	qword ptr [rdi], rdx
	jne .LBB1_2
	lea rdx, [rcx - 1]
	mov rax, qword ptr [rdi + 8]
	mov qword ptr [rdi], rdx
	mov rdi, rax
	jmp qword ptr [rcx + 7]
.LBB1_2:
	ret
