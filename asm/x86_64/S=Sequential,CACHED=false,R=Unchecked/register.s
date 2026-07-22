asm_register_asm:
	push r14
	push rbx
	push rax
	mov r14, rdi
	mov rbx, qword ptr [rdi]
	test bl, 1
	jne .LBB2_2
	add rbx, 9
	mov rax, qword ptr [rsi]
	mov rdi, qword ptr [rsi + 8]
	call qword ptr [rax]
	mov qword ptr [r14 + 8], rdx
	mov qword ptr [r14 + 16], rax
	xchg qword ptr [r14], rbx
	add rsp, 8
	pop rbx
	pop r14
	ret
.LBB2_2:
	mov rdi, r14
	mov rdx, rbx
	add rsp, 8
	pop rbx
	pop r14
	jmp spmc_waker::SpmcWaker<S,_,R>::register_impl_cold
