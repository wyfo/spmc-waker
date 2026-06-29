asm_register_asm:
	mov rdx, qword ptr [rdi]
	mov rax, qword ptr [rsi]
	cmp rax, rdx
	jne .LBB11_3
	mov rcx, qword ptr [rsi + 8]
	cmp rcx, qword ptr [rdi + 8]
	jne .LBB11_3
	inc rax
	mov qword ptr [rdi], rax
	ret
.LBB11_3:
	mov ecx, 1
	mov r8d, 1
	jmp spmc_waker::SpmcWaker<S,_>::register_cold
