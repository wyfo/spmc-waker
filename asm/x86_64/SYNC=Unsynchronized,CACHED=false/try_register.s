asm_try_register_asm:
	push rbx
	mov rbx, rdi
	mov rdx, qword ptr [rdi]
	test dl, 3
	jne .LBB12_2
	mov rax, qword ptr [rsi]
	mov rdi, qword ptr [rsi + 8]
	call qword ptr [rax]
	mov qword ptr [rbx + 8], rdx
	inc rax
	mov qword ptr [rbx], rax
	mov al, 1
	pop rbx
	ret
.LBB12_2:
	mov rdi, rbx
	xor ecx, ecx
	mov r8d, 1
	pop rbx
	jmp spmc_waker::SpmcWaker<S,_>::register_cold
