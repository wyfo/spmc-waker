uc_wake:
	push r14
	push rbx
	push rax
	mov rcx, qword ptr [rdi]
	test cl, 1
	je .LBB1_4
	mov edx, 2
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
