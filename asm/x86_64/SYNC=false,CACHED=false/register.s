uu_register:
	push rbx
	mov rax, rsi
	mov rbx, rdi
	mov rcx, qword ptr [rdi]
	mov rsi, qword ptr [rsi]
	test cl, 3
	jne .LBB1_2
	mov rdi, qword ptr [rax + 8]
	call qword ptr [rsi]
	mov qword ptr [rbx + 8], rdx
	or rax, 1
	xchg qword ptr [rbx], rax
	mov al, 1
	pop rbx
	ret
.LBB1_2:
	mov rdx, qword ptr [rax + 8]
	mov rdi, rbx
	pop rbx
	jmp spmc_waker::SpmcWaker<_,_>::overwrite
