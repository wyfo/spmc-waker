asm_unregister_asm:
	mov rax, rsi
	lea rcx, [rsi + 1]
	lock cmpxchg	qword ptr [rdi], rcx
	ret
