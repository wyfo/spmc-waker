asm_register_asm:
	mov rax, qword ptr [rdi]
.LBB3_1:
	mov rdx, rax
	test dl, 4
	jne .LBB3_7
	mov rcx, rdx
	and rcx, -8
	or rcx, 4
	mov rax, rdx
	lock cmpxchg	qword ptr [rdi], rcx
	jne .LBB3_1
	test dl, 2
	je spmc_waker::SpmcWaker<S,_,R>::register_impl_cold
	mov rcx, qword ptr [rdi + 8]
	mov rax, qword ptr [rdi + 16]
	cmp rcx, qword ptr [rsi + 8]
	jne spmc_waker::SpmcWaker<S,_,R>::register_impl_cold
	cmp rax, qword ptr [rsi]
	jne .LBB3_6
	add rdx, 7
	xchg qword ptr [rdi], rdx
	ret
.LBB3_6:
	jmp spmc_waker::SpmcWaker<S,_,R>::register_impl_cold
.LBB3_7:
	push rax
	lea rdi, [rip + .Lanon.8d71c8851bc03468f628dced9b2f7f7b.0]
	lea rdx, [rip + .Lanon.8d71c8851bc03468f628dced9b2f7f7b.2]
	mov esi, 47
	call qword ptr [rip + core::panicking::panic_fmt@GOTPCREL]
