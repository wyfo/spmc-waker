asm_unregister_asm:
	mov rdx, qword ptr [rdi]
	mov rax, qword ptr [rdi + 8]
	lea rsi, [rax - 1]
	mov rdi, qword ptr [rdx + 8]
	mov rcx, qword ptr [rdx + 16]
	lock cmpxchg	qword ptr [rdx], rsi
	jne .LBB6_1
	jmp qword ptr [rcx + 24]
.LBB6_1:
	ret
