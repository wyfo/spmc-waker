asm_register_asm:
	push rax
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
	je .LBB3_6
	mov rcx, qword ptr [rdi + 8]
	mov rax, qword ptr [rdi + 16]
	cmp rcx, qword ptr [rsi + 8]
	jne .LBB3_6
	cmp rax, qword ptr [rsi]
	jne .LBB3_6
	add rdx, 7
	mov qword ptr [rdi], rdx
	lock or	dword ptr [rsp - 64], 0
	pop rax
	ret
.LBB3_6:
	pop rax
	jmp <spmc_waker::SpmcWaker<spmc_waker::synchronization::Synchronized, true>>::register_impl_cold
.LBB3_7:
	lea rdi, [rip + .Lanon.40943f74b53f8fa4249390633cadaabd.0]
	lea rdx, [rip + .Lanon.40943f74b53f8fa4249390633cadaabd.2]
	mov esi, 47
	call qword ptr [rip + core::panicking::panic_fmt@GOTPCREL]
