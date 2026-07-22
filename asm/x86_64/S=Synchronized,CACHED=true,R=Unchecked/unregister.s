asm_unregister_asm:
	mov rcx, qword ptr [rdi]
	mov rax, qword ptr [rdi + 8]
	lea rdx, [rax + 1]
	lock cmpxchg	qword ptr [rcx], rdx
	ret
