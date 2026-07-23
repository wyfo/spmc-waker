<spmc_waker::SpmcWaker<spmc_waker::synchronization::Sequential, true, spmc_waker::registration::Unchecked>>::register_impl_cold:
	push r15
	push r14
	push r12
	push rbx
	push rax
	mov rbx, rdx
	mov r14, rdi
	test bl, 1
	jne .LBB1_3
	mov r15, qword ptr [r14 + 8]
	mov r12, qword ptr [r14 + 16]
	test bl, 2
	jne .LBB1_2
	add rbx, 9
	mov rax, qword ptr [rsi]
	mov rdi, qword ptr [rsi + 8]
	call qword ptr [rax]
	mov qword ptr [r14 + 8], rdx
	mov qword ptr [r14 + 16], rax
	mov rax, rbx
	xchg qword ptr [r14], rax
	test al, 2
	jne .LBB1_7
	jmp .LBB1_8
.LBB1_3:
	mov rdi, qword ptr [r14 + 8]
	mov rax, qword ptr [r14 + 16]
	mov r12, qword ptr [rsi]
	mov r15, qword ptr [rsi + 8]
	mov rcx, rdi
	xor rcx, r15
	mov rdx, rax
	xor rdx, r12
	or rdx, rcx
	je .LBB1_8
	lea rcx, [rbx + 7]
	xchg qword ptr [r14], rcx
	test cl, 3
	je .LBB1_5
	test cl, 2
	je .LBB1_11
	#MEMBARRIER
.LBB1_11:
	call qword ptr [rax + 24]
.LBB1_5:
	add rbx, 16
	mov rdi, r15
	call qword ptr [r12]
	mov qword ptr [r14 + 8], rdx
	mov qword ptr [r14 + 16], rax
	mov rax, rbx
	xchg qword ptr [r14], rax
	jmp .LBB1_8
.LBB1_2:
	add rbx, 7
	mov rax, qword ptr [rsi]
	mov rdi, qword ptr [rsi + 8]
	call qword ptr [rax]
	mov qword ptr [r14 + 8], rdx
	mov qword ptr [r14 + 16], rax
	mov rax, rbx
	xchg qword ptr [r14], rax
.LBB1_7:
	mov rdi, r15
	call qword ptr [r12 + 24]
.LBB1_8:
	mov rax, rbx
	add rsp, 8
	pop rbx
	pop r12
	pop r14
	pop r15
	ret
