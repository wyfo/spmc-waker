asm_take_asm:
	mov rsi, qword ptr [rdi]
	mov rax, rsi
	test sil, 1
	jne .LBB5_3
	lock or	dword ptr [rsp - 64], 0
	mov rax, qword ptr [rdi]
	test al, 1
	je .LBB5_2
.LBB5_3:
	#MEMBARRIER
	mov rdx, qword ptr [rdi + 8]
	mov rcx, qword ptr [rdi + 16]
	lea r8, [rax - 1]
	lock cmpxchg	qword ptr [rdi], r8
	jne .LBB5_4
.LBB5_7:
	mov rax, rcx
	ret
.LBB5_4:
	test sil, 1
	je .LBB5_2
	lock or	dword ptr [rsp - 64], 0
	mov rax, qword ptr [rdi]
	test al, 1
	je .LBB5_2
	#MEMBARRIER
	mov rdx, qword ptr [rdi + 8]
	mov rcx, qword ptr [rdi + 16]
	lea rsi, [rax - 1]
	lock cmpxchg	qword ptr [rdi], rsi
	je .LBB5_7
.LBB5_2:
	xor ecx, ecx
	mov rax, rcx
	ret
