sc_wake:
	push r14
	push rbx
	push rax
	mov rax, qword ptr [rdi]
.LBB1_1:
	mov rcx, rax
	or rcx, 2
	lock cmpxchg	qword ptr [rdi], rcx
	jne .LBB1_1
	test al, 2
	jne .LBB1_8
	test al, 1
	jne .LBB1_6
	mov rcx, rax
	add rcx, 2
	mov rdx, rax
	mov rax, rcx
	lock cmpxchg	qword ptr [rdi], rdx
.LBB1_8:
	add rsp, 8
	pop rbx
	pop r14
	ret
.LBB1_6:
	#MEMBARRIER
	mov rbx, rax
	dec rbx
	mov r14, rdi
	mov rdi, qword ptr [rdi + 8]
	call qword ptr [rax + 15]
	xchg qword ptr [r14], rbx
	add rsp, 8
	pop rbx
	pop r14
	ret
	xchg qword ptr [r14], rbx
	mov rdi, rax
	call _Unwind_Resume@PLT
