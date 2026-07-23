<spmc_waker::SpmcWaker<spmc_waker::synchronization::Synchronized, false, spmc_waker::registration::Unchecked>>::register_impl_cold:
	push r15
	push r14
	push r12
	push rbx
	push rax
	mov rbx, rdx
	mov r14, rdi
	mov rdi, qword ptr [rdi + 8]
	mov rax, qword ptr [r14 + 16]
	mov r12, qword ptr [rsi]
	mov r15, qword ptr [rsi + 8]
	mov rcx, rdi
	xor rcx, r15
	mov rdx, rax
	xor rdx, r12
	or rdx, rcx
	je .LBB1_3
	lea rcx, [rbx - 1]
	xchg qword ptr [r14], rcx
	test cl, 1
	je .LBB1_2
	call qword ptr [rax + 24]
.LBB1_2:
	add rbx, 8
	mov rdi, r15
	call qword ptr [r12]
	mov qword ptr [r14 + 8], rdx
	mov qword ptr [r14 + 16], rax
	mov rax, rbx
	xchg qword ptr [r14], rax
.LBB1_3:
	mov rax, rbx
	add rsp, 8
	pop rbx
	pop r12
	pop r14
	pop r15
	ret
