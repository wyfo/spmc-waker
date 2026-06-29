asm_poll_wait_until_asm:
	movzx eax, byte ptr [rdx]
	test al, al
	je .LBB10_1
	xor eax, eax
	ret
.LBB10_1:
	mov rcx, rdx
	mov rsi, qword ptr [rsi]
	mov rdx, qword ptr [rdi]
	mov r8, qword ptr [rsi]
	cmp r8, rdx
	jne spmc_waker::SpmcWaker<S,_>::poll_wait_until_cold
	mov rax, qword ptr [rsi + 8]
	cmp rax, qword ptr [rdi + 8]
	jne spmc_waker::SpmcWaker<S,_>::poll_wait_until_cold
	lea rax, [r8 + 1]
	mov qword ptr [rdi], rax
	movzx ecx, byte ptr [rcx]
	test cl, cl
	je .LBB10_4
	lock cmpxchg	qword ptr [rdi], r8
	xor eax, eax
	ret
.LBB10_4:
	mov al, 1
	ret
