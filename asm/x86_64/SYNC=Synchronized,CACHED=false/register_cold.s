spmc_waker::SpmcWaker<S,_>::register_cold:
	push r14
	push rbx
	push rax
	mov r9, qword ptr [rsi]
	lea rax, [r9 + 1]
	cmp rax, rdx
	sete al
	lea r10, [rsi + 8]
	test r8b, al
	je .LBB0_2
	mov rax, qword ptr [r10]
	cmp rax, qword ptr [rdi + 8]
	je .LBB0_15
.LBB0_2:
	test dl, 2
	jne .LBB0_9
	test dl, 1
	jne .LBB0_11
.LBB0_4:
	test r8b, r8b
	je .LBB0_7
	mov rbx, rdx
	mov r14, rdi
	mov rdi, qword ptr [r10]
	call qword ptr [r9]
	mov r9, rax
	mov qword ptr [rsp], rdx
	mov r10, rsp
	mov rdi, r14
	mov rdx, rbx
.LBB0_7:
	test dl, 1
	jne .LBB0_13
	mov rax, qword ptr [r10]
	mov qword ptr [rdi + 8], rax
	inc r9
	xchg qword ptr [rdi], r9
	jmp .LBB0_15
.LBB0_9:
	test cl, cl
	jne .LBB0_12
.LBB0_20:
	xor eax, eax
	add rsp, 8
	pop rbx
	pop r14
	ret
.LBB0_11:
	lea rax, [rdx + 4]
	cmp rax, 8
	jae .LBB0_16
.LBB0_12:
	movzx ecx, r8b
	mov rax, rsi
	mov rsi, rdx
	mov rdx, rax
	add rsp, 8
	pop rbx
	pop r14
	jmp spmc_waker::SpmcWaker<S,_>::register_fallback
.LBB0_13:
	mov rax, qword ptr [rdi + 8]
	mov rcx, qword ptr [r10]
	mov qword ptr [rdi + 8], rcx
	inc r9
	xchg qword ptr [rdi], r9
	cmp rdx, 1
	je .LBB0_15
	mov rdi, rax
	call qword ptr [rdx + 23]
.LBB0_15:
	mov al, 1
	add rsp, 8
	pop rbx
	pop r14
	ret
.LBB0_16:
	lea r11, [rip + .Lanon.3906cc28ed211ab7cb59ed8dee7465a5.0]
	mov rax, rdx
	lock cmpxchg	qword ptr [rdi], r11
	je .LBB0_4
	test cl, cl
	je .LBB0_20
	test al, 2
	jne .LBB0_22
	mov rdx, rax
	jmp .LBB0_4
.LBB0_22:
	movzx ecx, r8b
	mov rdx, rsi
	mov rsi, rax
	add rsp, 8
	pop rbx
	pop r14
	jmp spmc_waker::SpmcWaker<S,_>::register_fallback
	test bl, 1
	je .LBB0_25
	xchg qword ptr [r14], rbx
.LBB0_25:
	mov rdi, rax
	call _Unwind_Resume@PLT
