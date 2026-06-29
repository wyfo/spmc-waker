spmc_waker::SpmcWaker<S,_>::register_cold:
	push r15
	push r14
	push rbx
	mov r9, qword ptr [rsi]
	lea rax, [r9 + 1]
	cmp rax, rdx
	sete al
	test r8b, al
	je .LBB0_2
	mov rax, qword ptr [rsi + 8]
	cmp rax, qword ptr [rdi + 8]
	je .LBB0_14
.LBB0_2:
	test dl, 2
	jne .LBB0_7
	test dl, 1
	jne .LBB0_10
.LBB0_4:
	mov rbx, qword ptr [rdi + 8]
	test r8b, r8b
	je .LBB0_12
	mov r14, rdi
	mov r15, rdx
	mov rdi, qword ptr [rsi + 8]
	call qword ptr [r9]
	mov r9, rax
	mov rax, rdx
	mov rdx, r15
	mov rdi, r14
	jmp .LBB0_13
.LBB0_7:
	test cl, cl
	jne .LBB0_11
.LBB0_8:
	xor eax, eax
	pop rbx
	pop r14
	pop r15
	ret
.LBB0_10:
	lea rax, [rdx + 4]
	cmp rax, 8
	jae .LBB0_15
.LBB0_11:
	movzx ecx, r8b
	mov rax, rsi
	mov rsi, rdx
	mov rdx, rax
	pop rbx
	pop r14
	pop r15
	jmp spmc_waker::SpmcWaker<S,_>::register_fallback
.LBB0_12:
	mov rax, qword ptr [rsi + 8]
.LBB0_13:
	mov qword ptr [rdi + 8], rax
	inc r9
	mov qword ptr [rdi], r9
	mov rdi, rbx
	call qword ptr [rdx + 24]
.LBB0_14:
	mov al, 1
	pop rbx
	pop r14
	pop r15
	ret
.LBB0_15:
	lea r10, [rip + .Lanon.098797136eefd0e6b84892bc51c6e462.0]
	mov rax, rdx
	lock cmpxchg	qword ptr [rdi], r10
	jne .LBB0_17
	dec rdx
	jmp .LBB0_4
.LBB0_17:
	test cl, cl
	je .LBB0_8
	test al, 2
	jne .LBB0_21
	mov rdx, rax
	jmp .LBB0_4
.LBB0_21:
	movzx ecx, r8b
	mov rdx, rsi
	mov rsi, rax
	pop rbx
	pop r14
	pop r15
	jmp spmc_waker::SpmcWaker<S,_>::register_fallback
	mov r14, rax
	mov rdi, rbx
	call qword ptr [r15 + 24]
	mov rdi, r14
	call _Unwind_Resume@PLT
	call qword ptr [rip + core::panicking::panic_in_cleanup@GOTPCREL]
