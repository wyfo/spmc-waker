asm_wake_asm:
	push r15
	push r14
	push r13
	push r12
	push rbx
	mov rbx, rdi
	mov rcx, qword ptr [rdi]
	mov r14, rcx
	test cl, 1
	jne .LBB6_2
	lock or	dword ptr [rsp - 64], 0
	mov r14, qword ptr [rbx]
	test r14b, 1
	je .LBB6_12
.LBB6_2:
	#MEMBARRIER
	mov r15, qword ptr [rbx + 8]
	mov rdx, qword ptr [rbx + 16]
	lea r12, [r14 - 1]
	mov rax, r14
	lock cmpxchg	qword ptr [rbx], r12
	jne .LBB6_3
.LBB6_6:
	mov rdi, r15
	mov r13, rdx
	call qword ptr [rdx + 16]
	inc r14
	mov rax, r12
	lock cmpxchg	qword ptr [rbx], r14
	je .LBB6_12
	mov rax, r13
	mov rdi, r15
	pop rbx
	pop r12
	pop r13
	pop r14
	pop r15
	jmp qword ptr [rax + 24]
.LBB6_3:
	test cl, 1
	je .LBB6_12
	lock or	dword ptr [rsp - 64], 0
	mov r14, qword ptr [rbx]
	test r14b, 1
	je .LBB6_12
	#MEMBARRIER
	mov r15, qword ptr [rbx + 8]
	mov rdx, qword ptr [rbx + 16]
	lea r12, [r14 - 1]
	mov rax, r14
	lock cmpxchg	qword ptr [rbx], r12
	je .LBB6_6
.LBB6_12:
	pop rbx
	pop r12
	pop r13
	pop r14
	pop r15
	ret
	mov rbx, rax
	mov rdi, r15
	call qword ptr [r13 + 24]
	mov rdi, rbx
	call _Unwind_Resume@PLT
	call qword ptr [rip + core::panicking::panic_in_cleanup@GOTPCREL]
