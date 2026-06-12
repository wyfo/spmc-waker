sc_wake:
	push r14
	push rbx
	push rax
	mov rbx, rdi
	mov rax, qword ptr [rdi]
.LBB1_1:
	mov rcx, rax
	or rcx, 2
	lock cmpxchg	qword ptr [rbx], rcx
	jne .LBB1_1
	mov ecx, eax
	and ecx, 3
	cmp ecx, 1
	jne .LBB1_5
	#MEMBARRIER
	mov r14, rax
	dec r14
	mov rdi, qword ptr [rbx + 8]
	call qword ptr [rax + 15]
	xchg qword ptr [rbx], r14
.LBB1_7:
	add rsp, 8
	pop rbx
	pop r14
	ret
.LBB1_5:
	test al, 2
	jne .LBB1_7
	mov rcx, rax
	add rcx, 2
	mov rdx, rax
	mov rax, rcx
	lock cmpxchg	qword ptr [rbx], rdx
	add rsp, 8
	pop rbx
	pop r14
	ret
	xchg qword ptr [rbx], r14
	mov rdi, rax
	call _Unwind_Resume@PLT
