asm_take_asm:
	mov rax, qword ptr [rdi]
	test al, 1
	jne .LBB4_2
	xor eax, eax
	ret
.LBB4_2:
	#MEMBARRIER
	mov rdx, qword ptr [rdi + 8]
	mov rcx, qword ptr [rdi + 16]
	lea rsi, [rax - 1]
	lock cmpxchg	qword ptr [rdi], rsi
	mov eax, 0
	cmove rax, rcx
	ret
