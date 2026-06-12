su_wake:
	mov rax, qword ptr [rdi]
.LBB1_1:
	mov rcx, rax
	or rcx, 2
	lock cmpxchg	qword ptr [rdi], rcx
	jne .LBB1_1
	test al, 2
	jne .LBB1_5
	test al, 1
	jne .LBB1_6
	mov rcx, rax
	add rcx, 2
	mov rdx, rax
	mov rax, rcx
	lock cmpxchg	qword ptr [rdi], rdx
.LBB1_5:
	ret
.LBB1_6:
	#MEMBARRIER
	mov rdx, rax
	dec rdx
	mov rcx, qword ptr [rdi + 8]
	xchg qword ptr [rdi], rdx
	mov rdi, rcx
	jmp qword ptr [rax + 7]
