asm_poll_wait_until_asm:
	push r15
	push r14
	push rbx
	movzx eax, byte ptr [rdx]
	test al, al
	je .LBB2_2
.LBB2_1:
	xor eax, eax
	pop rbx
	pop r14
	pop r15
	ret
.LBB2_2:
	mov rsi, qword ptr [rsi]
	mov rax, qword ptr [rdi]
.LBB2_3:
	mov rbx, rax
	test bl, 4
	jne .LBB2_13
	mov rcx, rbx
	and rcx, -8
	or rcx, 4
	mov rax, rbx
	lock cmpxchg	qword ptr [rdi], rcx
	jne .LBB2_3
	mov r15, rdx
	test bl, 1
	jne .LBB2_10
	mov r14, rdi
	mov rax, qword ptr [rsi]
	mov rdi, qword ptr [rsi + 8]
	call qword ptr [rax]
	add rbx, 9
	mov qword ptr [r14 + 8], rdx
	mov qword ptr [r14 + 16], rax
	mov rax, rbx
	xchg qword ptr [r14], rax
	movzx eax, byte ptr [r15]
	test al, al
	je .LBB2_11
.LBB2_8:
	lea rdx, [rbx - 1]
	mov rdi, qword ptr [r14 + 8]
	mov rcx, qword ptr [r14 + 16]
	mov rax, rbx
	lock cmpxchg	qword ptr [r14], rdx
	jne .LBB2_1
	call qword ptr [rcx + 24]
	jmp .LBB2_1
.LBB2_10:
	mov r14, rdi
	mov rdx, rbx
	call spmc_waker::SpmcWaker<S,_,R>::register_impl_cold
	mov rbx, rax
	movzx eax, byte ptr [r15]
	test al, al
	jne .LBB2_8
.LBB2_11:
	mov al, 1
	pop rbx
	pop r14
	pop r15
	ret
.LBB2_13:
	lea rdi, [rip + .Lanon.40943f74b53f8fa4249390633cadaabd.0]
	lea rdx, [rip + .Lanon.40943f74b53f8fa4249390633cadaabd.2]
	mov esi, 47
	call qword ptr [rip + core::panicking::panic_fmt@GOTPCREL]
	xchg qword ptr [r14], rbx
	mov rdi, rax
	call _Unwind_Resume@PLT
