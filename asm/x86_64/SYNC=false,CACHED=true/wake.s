uc_wake:
	push r14
	push rbx
	push rax
	mov rcx, qword ptr [rdi]
	mov eax, ecx
	and eax, 3
	cmp eax, 1
	jne .LBB1_4
	lea rdx, [rcx + 2]
	mov rax, rcx
	lock cmpxchg	qword ptr [rdi], rdx
	jne .LBB1_4
	lea rbx, [rcx - 1]
	mov r14, rdi
	mov rdi, qword ptr [rdi + 8]
	call qword ptr [rcx + 15]
	mov qword ptr [r14], rbx
.LBB1_4:
	add rsp, 8
	pop rbx
	pop r14
	ret
	mov qword ptr [r14], rbx
	mov rdi, rax
	call _Unwind_Resume@PLT
