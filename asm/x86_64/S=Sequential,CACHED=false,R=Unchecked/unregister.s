asm_unregister_asm:
	mov rax, rsi
	mov rcx, rdi
	dec rsi
	mov rdi, qword ptr [rdi + 8]
	mov rdx, qword ptr [rcx + 16]
	lock cmpxchg	qword ptr [rcx], rsi
	jne .LBB5_1
	jmp qword ptr [rdx + 24]
.LBB5_1:
	ret
