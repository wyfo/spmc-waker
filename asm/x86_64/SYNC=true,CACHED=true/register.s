sc_register:
	mov rax, rsi
	mov rcx, qword ptr [rdi]
	mov rsi, qword ptr [rsi]
	mov rdx, qword ptr [rax + 8]
	cmp rsi, rcx
	jne spmc_waker::SpmcWaker<_,_>::overwrite
	cmp rdx, qword ptr [rdi + 8]
	jne spmc_waker::SpmcWaker<_,_>::overwrite
	or rcx, 1
	xchg qword ptr [rdi], rcx
	mov al, 1
	ret
