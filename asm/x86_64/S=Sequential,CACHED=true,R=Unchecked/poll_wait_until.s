asm_poll_wait_until_asm:
	movzx eax, byte ptr [rdx]
	test al, al
	je .LBB2_1
	xor eax, eax
	ret
.LBB2_1:
	mov rsi, qword ptr [rsi]
	mov rax, qword ptr [rdi]
	test al, 2
	je .LBB2_5
	mov r8, qword ptr [rdi + 8]
	mov rcx, qword ptr [rdi + 16]
	cmp r8, qword ptr [rsi + 8]
	jne .LBB2_5
	cmp rcx, qword ptr [rsi]
	jne .LBB2_5
	add rax, 7
	mov rcx, rax
	xchg qword ptr [rdi], rcx
	movzx ecx, byte ptr [rdx]
	test cl, cl
	je .LBB2_7
.LBB2_8:
	lea rcx, [rax + 1]
	lock cmpxchg	qword ptr [rdi], rcx
	xor eax, eax
	ret
.LBB2_5:
	push r14
	push rbx
	push rax
	mov rbx, rdi
	mov r14, rdx
	mov rdx, rax
	call spmc_waker::SpmcWaker<S,_,R>::register_impl_cold
	mov rdx, r14
	mov rdi, rbx
	add rsp, 8
	pop rbx
	pop r14
	movzx ecx, byte ptr [rdx]
	test cl, cl
	jne .LBB2_8
.LBB2_7:
	mov al, 1
	ret
