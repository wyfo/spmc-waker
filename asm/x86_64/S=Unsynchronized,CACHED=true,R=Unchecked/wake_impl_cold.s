<spmc_waker::SpmcWaker<spmc_waker::synchronization::Unsynchronized, true, spmc_waker::registration::Unchecked>>::wake_impl_cold:
	push r15
	push r14
	push r13
	push r12
	push rbx
	mov rbx, qword ptr [rdi + 8]
	mov rcx, qword ptr [rdi + 16]
	lea r12, [rsi - 1]
	mov rax, rsi
	lock cmpxchg	qword ptr [rdi], r12
	jne .LBB0_7
	mov r14, rsi
	mov r15, rdi
	mov rdi, rbx
	mov r13, rcx
	call qword ptr [rcx + 16]
	inc r14
	mov rax, r12
	lock cmpxchg	qword ptr [r15], r14
	jne .LBB0_3
.LBB0_7:
	pop rbx
	pop r12
	pop r13
	pop r14
	pop r15
	ret
.LBB0_3:
	mov rax, r13
	mov rdi, rbx
	pop rbx
	pop r12
	pop r13
	pop r14
	pop r15
	jmp qword ptr [rax + 24]
	mov r14, rax
	mov rdi, rbx
	call qword ptr [r13 + 24]
	mov rdi, r14
	call _Unwind_Resume@PLT
	call qword ptr [rip + core::panicking::panic_in_cleanup@GOTPCREL]
