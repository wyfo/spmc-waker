<spmc_waker::SpmcWaker<spmc_waker::synchronization::Unsynchronized>>::register_impl_cold:
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
	jne .LBB0_1
	mov qword ptr [r14], rbx
	jmp .LBB0_4
.LBB0_1:
	call qword ptr [rax]
	add rbx, 8
	mov qword ptr [r14 + 8], rdx
	mov qword ptr [r14 + 16], rax
	mov qword ptr [r14], rbx
	mov rdi, r15
	call qword ptr [r12 + 24]
.LBB0_4:
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
