asm_poll_wait_until_asm:
	movzx eax, byte ptr [rdx]
	test al, al
	je .LBB1_2
	xor eax, eax
	ret
.LBB1_2:
	push r15
	push r14
	push rbx
	mov r14, rdx
	mov rbx, rdi
	mov rsi, qword ptr [rsi]
	mov r15, qword ptr [rdi]
	test r15b, 1
	jne .LBB1_7
	add r15, 9
	mov rax, qword ptr [rsi]
	mov rdi, qword ptr [rsi + 8]
	call qword ptr [rax]
	mov qword ptr [rbx + 8], rdx
	mov qword ptr [rbx + 16], rax
	mov rax, r15
	xchg qword ptr [rbx], rax
	movzx eax, byte ptr [r14]
	test al, al
	je .LBB1_8
.LBB1_4:
	lea rdx, [r15 - 1]
	mov rdi, qword ptr [rbx + 8]
	mov rcx, qword ptr [rbx + 16]
	mov rax, r15
	lock cmpxchg	qword ptr [rbx], rdx
	jne .LBB1_6
	call qword ptr [rcx + 24]
.LBB1_6:
	xor eax, eax
	pop rbx
	pop r14
	pop r15
	ret
.LBB1_7:
	mov rdi, rbx
	mov rdx, r15
	call spmc_waker::SpmcWaker<S,_,R>::register_impl_cold
	mov r15, rax
	movzx eax, byte ptr [r14]
	test al, al
	jne .LBB1_4
.LBB1_8:
	mov al, 1
	pop rbx
	pop r14
	pop r15
	ret
