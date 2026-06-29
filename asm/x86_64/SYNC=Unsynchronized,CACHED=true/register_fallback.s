spmc_waker::SpmcWaker<S,_>::register_fallback:
	push r15
	push r14
	push rbx
	sub rsp, 16
	mov rax, rsi
	mov rbx, rdi
	test ecx, ecx
	je .LBB2_2
	mov rcx, qword ptr [rdx]
	mov rdi, qword ptr [rdx + 8]
	mov r14, rax
	call qword ptr [rcx]
	mov rcx, rax
	mov rax, r14
	mov qword ptr [rsp], rcx
	mov qword ptr [rsp + 8], rdx
	mov rdx, rsp
.LBB2_2:
	lea rcx, [rax + 4]
	cmp rcx, 7
	ja .LBB2_13
	xor ecx, ecx
.LBB2_4:
	lock cmpxchg	qword ptr [rbx], rcx
	je .LBB2_10
	mov rsi, rax
	add rsi, 4
	cmp rsi, 8
	jb .LBB2_4
	mov rdi, rbx
	mov rsi, rdx
	mov rdx, rax
	mov ecx, 1
	xor r8d, r8d
	call spmc_waker::SpmcWaker<S,_>::register_cold
	jmp .LBB2_18
.LBB2_13:
	mov rcx, qword ptr [rdx + 8]
	mov qword ptr [rbx + 16], rcx
	mov rcx, qword ptr [rdx]
	mov qword ptr [rbx + 24], rcx
	mov ecx, 1
	lock cmpxchg	qword ptr [rbx], rcx
	mov rcx, rax
	mov al, 1
	je .LBB2_18
	xor r15d, r15d
	mov rax, rcx
	jmp .LBB2_15
.LBB2_10:
	mov r14, qword ptr [rbx + 16]
	mov r15, qword ptr [rbx + 24]
	mov rax, qword ptr [rdx + 8]
	mov qword ptr [rbx + 16], rax
	mov rax, qword ptr [rdx]
	mov qword ptr [rbx + 24], rax
	mov ecx, 1
	xor eax, eax
	lock cmpxchg	qword ptr [rbx], rcx
	jne .LBB2_15
	mov al, 1
	test r15, r15
	je .LBB2_18
	mov rdi, r14
	call qword ptr [r15 + 24]
	mov al, 1
.LBB2_18:
	add rsp, 16
	pop rbx
	pop r14
	pop r15
	ret
.LBB2_15:
	mov rdi, rbx
	mov rsi, rdx
	mov rdx, rax
	mov ecx, 1
	xor r8d, r8d
	call spmc_waker::SpmcWaker<S,_>::register_cold
	test r15, r15
	je .LBB2_18
	mov rdi, r14
	mov ebx, eax
	call qword ptr [r15 + 24]
	mov eax, ebx
	jmp .LBB2_18
	mov rbx, rax
	test r15, r15
	je .LBB2_9
	mov rdi, r14
	call qword ptr [r15 + 24]
.LBB2_9:
	mov rdi, rbx
	call _Unwind_Resume@PLT
	call qword ptr [rip + core::panicking::panic_in_cleanup@GOTPCREL]
