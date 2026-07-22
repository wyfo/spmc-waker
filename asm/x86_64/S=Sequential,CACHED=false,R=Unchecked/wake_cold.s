asm_wake_cold_asm:
	mov rax, qword ptr [rdi]
	test al, 1
	jne .LBB7_1
.LBB7_2:
	ret
.LBB7_1:
	mov rcx, qword ptr [rdi + 8]
	mov rdx, qword ptr [rdi + 16]
	lea rsi, [rax - 1]
	lock cmpxchg	qword ptr [rdi], rsi
	jne .LBB7_2
	mov rdi, rcx
	jmp qword ptr [rdx + 8]
