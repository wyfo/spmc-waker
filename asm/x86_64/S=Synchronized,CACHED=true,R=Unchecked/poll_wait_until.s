asm_poll_wait_until_asm:
	movzx eax, byte ptr [rdx]
	test al, al
	je .LBB2_1
	xor eax, eax
	ret
.LBB2_1:
	push r14
	push rbx
	push rax
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
	mov qword ptr [rdi], rax
	lock or	dword ptr [rsp - 64], 0
.LBB2_6:
	movzx ecx, byte ptr [rdx]
	test cl, cl
	lea rsp, [rsp + 8]
	pop rbx
	pop r14
	je .LBB2_7
	lea rcx, [rax + 1]
	lock cmpxchg	qword ptr [rdi], rcx
	xor eax, eax
	ret
.LBB2_7:
	mov al, 1
	ret
.LBB2_5:
	mov rbx, rdi
	mov r14, rdx
	mov rdx, rax
	call <spmc_waker::SpmcWaker<spmc_waker::synchronization::Synchronized, true, spmc_waker::registration::Unchecked>>::register_impl_cold
	mov rdx, r14
	mov rdi, rbx
	jmp .LBB2_6
