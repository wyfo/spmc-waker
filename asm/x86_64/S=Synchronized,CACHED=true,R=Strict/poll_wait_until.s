asm_poll_wait_until_asm:
	push r14
	push rbx
	push rax
	movzx eax, byte ptr [rdx]
	test al, al
	jne .LBB2_9
	mov rsi, qword ptr [rsi]
	mov rax, qword ptr [rdi]
.LBB2_2:
	mov rcx, rax
	test cl, 4
	jne .LBB2_13
	mov r8, rcx
	and r8, -8
	or r8, 4
	mov rax, rcx
	lock cmpxchg	qword ptr [rdi], r8
	jne .LBB2_2
	test cl, 2
	je .LBB2_10
	mov r8, qword ptr [rdi + 8]
	mov rax, qword ptr [rdi + 16]
	cmp r8, qword ptr [rsi + 8]
	jne .LBB2_10
	cmp rax, qword ptr [rsi]
	jne .LBB2_10
	add rcx, 7
	mov rax, rcx
	xchg qword ptr [rdi], rax
	movzx eax, byte ptr [rdx]
	test al, al
	je .LBB2_11
.LBB2_8:
	lea rdx, [rcx + 1]
	mov rax, rcx
	lock cmpxchg	qword ptr [rdi], rdx
.LBB2_9:
	xor eax, eax
	add rsp, 8
	pop rbx
	pop r14
	ret
.LBB2_10:
	mov rbx, rdi
	mov r14, rdx
	mov rdx, rcx
	call spmc_waker::SpmcWaker<S,_,R>::register_impl_cold
	mov rdx, r14
	mov rdi, rbx
	mov rcx, rax
	movzx eax, byte ptr [rdx]
	test al, al
	jne .LBB2_8
.LBB2_11:
	mov al, 1
	add rsp, 8
	pop rbx
	pop r14
	ret
.LBB2_13:
	lea rdi, [rip + .Lanon.40943f74b53f8fa4249390633cadaabd.0]
	lea rdx, [rip + .Lanon.40943f74b53f8fa4249390633cadaabd.2]
	mov esi, 47
	call qword ptr [rip + core::panicking::panic_fmt@GOTPCREL]
