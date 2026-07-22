spmc_waker::SpmcWaker<S,_,R>::register_impl_cold:
	push r15
	push r14
	push r12
	push rbx
	push rax
	mov rbx, rdx
	mov r14, rdi
	mov r15, qword ptr [rdi + 8]
	mov r12, qword ptr [rdi + 16]
	mov rax, qword ptr [rsi]
	mov rdi, qword ptr [rsi + 8]
	mov rcx, r15
	xor rcx, rdi
	mov rdx, r12
	xor rdx, rax
	or rdx, rcx
	jne .LBB1_1
	mov rax, rbx
	xchg qword ptr [r14], rax
	jmp .LBB1_4
.LBB1_1:
	call qword ptr [rax]
	add rbx, 8
	mov qword ptr [r14 + 8], rdx
	mov qword ptr [r14 + 16], rax
	mov rax, rbx
	xchg qword ptr [r14], rax
	mov rdi, r15
	call qword ptr [r12 + 24]
.LBB1_4:
	mov rax, rbx
	add rsp, 8
	pop rbx
	pop r12
	pop r14
	pop r15
	ret
	xchg qword ptr [r14], rbx
	mov rdi, rax
	call _Unwind_Resume@PLT
