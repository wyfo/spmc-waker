spmc_waker::SpmcWaker<S,_,R>::register_impl_cold:
	push r15
	push r14
	push r12
	push rbx
	push rax
	mov rbx, rdx
	mov r14, rdi
	test bl, 1
	jne .LBB1_3
	mov r15, qword ptr [r14 + 8]
	mov r12, qword ptr [r14 + 16]
	test bl, 2
	jne .LBB1_2
	add rbx, 9
	mov rax, qword ptr [rsi]
	mov rdi, qword ptr [rsi + 8]
	call qword ptr [rax]
	mov qword ptr [r14 + 8], rdx
	mov qword ptr [r14 + 16], rax
	mov rax, rbx
	xchg qword ptr [r14], rax
	test al, 2
	je .LBB1_10
	#MEMBARRIER
	jmp .LBB1_9
.LBB1_3:
	mov rdi, qword ptr [r14 + 8]
	mov rcx, qword ptr [r14 + 16]
	mov r12, qword ptr [rsi]
	mov r15, qword ptr [rsi + 8]
	mov rax, rdi
	xor rax, r15
	mov rdx, rcx
	xor rdx, r12
	or rdx, rax
	je .LBB1_10
	lea rax, [rbx - 1]
	xchg qword ptr [r14], rax
	test al, 1
	jne .LBB1_12
	test al, 2
	jne .LBB1_6
	mov rdx, rax
	or rdx, 2
	lock cmpxchg	qword ptr [r14], rdx
	jne .LBB1_12
	jmp .LBB1_13
.LBB1_2:
	add rbx, 7
	mov rax, qword ptr [rsi]
	mov rdi, qword ptr [rsi + 8]
	call qword ptr [rax]
	mov qword ptr [r14 + 8], rdx
	mov qword ptr [r14 + 16], rax
	mov qword ptr [r14], rbx
.LBB1_9:
	mov rdi, r15
	call qword ptr [r12 + 24]
	jmp .LBB1_10
.LBB1_6:
	#MEMBARRIER
.LBB1_12:
	call qword ptr [rcx + 24]
.LBB1_13:
	add rbx, 8
	mov rdi, r15
	call qword ptr [r12]
	mov qword ptr [r14 + 8], rdx
	mov qword ptr [r14 + 16], rax
	mov qword ptr [r14], rbx
.LBB1_10:
	mov rax, rbx
	add rsp, 8
	pop rbx
	pop r12
	pop r14
	pop r15
	ret
