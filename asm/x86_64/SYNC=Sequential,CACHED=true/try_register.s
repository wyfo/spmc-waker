asm_try_register_asm:
	mov rdx, qword ptr [rdi]
	mov rax, qword ptr [rsi]
	cmp rax, rdx
	jne .LBB12_3
	mov rcx, qword ptr [rsi + 8]
	cmp rcx, qword ptr [rdi + 8]
	jne .LBB12_3
	inc rax
	xchg qword ptr [rdi], rax
	mov al, 1
	ret
.LBB12_3:
	xor ecx, ecx
	mov r8d, 1
	jmp spmc_waker::SpmcWaker<S,_>::register_cold
