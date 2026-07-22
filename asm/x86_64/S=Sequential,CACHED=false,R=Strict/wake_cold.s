asm_wake_cold_asm:
	mov rax, qword ptr [rdi]
	test al, 1
	jne .LBB6_1
.LBB6_2:
	ret
.LBB6_1:
	mov rcx, qword ptr [rdi + 8]
	mov rdx, qword ptr [rdi + 16]
	lea rsi, [rax - 1]
	lock cmpxchg	qword ptr [rdi], rsi
	jne .LBB6_2
	mov rdi, rcx
	jmp qword ptr [rdx + 8]
