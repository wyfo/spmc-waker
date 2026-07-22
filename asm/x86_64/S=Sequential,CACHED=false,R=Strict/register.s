asm_register_asm:
	push r14
	push rbx
	push rax
	mov rbx, rdi
	mov rax, qword ptr [rdi]
.LBB2_1:
	mov r14, rax
	test r14b, 4
	jne .LBB2_7
	mov rcx, r14
	and rcx, -8
	or rcx, 4
	mov rax, r14
	lock cmpxchg	qword ptr [rbx], rcx
	jne .LBB2_1
	test r14b, 1
	jne .LBB2_6
	mov rax, qword ptr [rsi]
	mov rdi, qword ptr [rsi + 8]
	call qword ptr [rax]
	add r14, 9
	mov qword ptr [rbx + 8], rdx
	mov qword ptr [rbx + 16], rax
	xchg qword ptr [rbx], r14
	add rsp, 8
	pop rbx
	pop r14
	ret
.LBB2_6:
	mov rdi, rbx
	mov rdx, r14
	add rsp, 8
	pop rbx
	pop r14
	jmp spmc_waker::SpmcWaker<S,_,R>::register_impl_cold
.LBB2_7:
	lea rdi, [rip + .Lanon.8d71c8851bc03468f628dced9b2f7f7b.0]
	lea rdx, [rip + .Lanon.8d71c8851bc03468f628dced9b2f7f7b.2]
	mov esi, 47
	call qword ptr [rip + core::panicking::panic_fmt@GOTPCREL]
	xchg qword ptr [rbx], r14
	mov rdi, rax
	call _Unwind_Resume@PLT
