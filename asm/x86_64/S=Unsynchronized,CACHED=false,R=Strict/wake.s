asm_wake_asm:
	mov rax, qword ptr [rdi]
	test al, 1
	je .LBB5_2
	#MEMBARRIER
	mov rcx, qword ptr [rdi + 8]
	mov rdx, qword ptr [rdi + 16]
	lea rsi, [rax - 1]
	lock cmpxchg	qword ptr [rdi], rsi
	jne .LBB5_2
	mov rdi, rcx
	jmp qword ptr [rdx + 8]
.LBB5_2:
	ret
