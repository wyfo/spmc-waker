<spmc_waker::SpmcWaker<spmc_waker::synchronization::Synchronized, true, spmc_waker::registration::Unchecked>>::wake_impl_cold:
	push r15
	push r14
	push r13
	push r12
	push rbx
	mov r15, rsi
	mov rbx, rdi
	mov r14, qword ptr [rdi + 8]
	mov rcx, qword ptr [rdi + 16]
	lea r12, [rsi - 1]
	mov rax, rsi
	lock cmpxchg	qword ptr [rdi], r12
	jne .LBB0_1
.LBB0_4:
	mov rdi, r14
	mov r13, rcx
	call qword ptr [rcx + 16]
	inc r15
	mov rax, r12
	lock cmpxchg	qword ptr [rbx], r15
	je .LBB0_10
	mov rax, r13
	mov rdi, r14
	pop rbx
	pop r12
	pop r13
	pop r14
	pop r15
	jmp qword ptr [rax + 24]
.LBB0_1:
	test dl, dl
	jne .LBB0_10
	lock or	dword ptr [rsp - 64], 0
	mov r15, qword ptr [rbx]
	test r15b, 1
	je .LBB0_10
	#MEMBARRIER
	mov r14, qword ptr [rbx + 8]
	mov rcx, qword ptr [rbx + 16]
	lea r12, [r15 - 1]
	mov rax, r15
	lock cmpxchg	qword ptr [rbx], r12
	je .LBB0_4
.LBB0_10:
	pop rbx
	pop r12
	pop r13
	pop r14
	pop r15
	ret
	mov rbx, rax
	mov rdi, r14
	call qword ptr [r13 + 24]
	mov rdi, rbx
	call _Unwind_Resume@PLT
	call qword ptr [rip + core::panicking::panic_in_cleanup@GOTPCREL]
