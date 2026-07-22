asm_wake_asm:
	mov rdx, qword ptr [rdi]
	mov rax, rdx
	test dl, 1
	jne .LBB7_2
	lock or	dword ptr [rsp - 64], 0
	mov rax, qword ptr [rdi]
	test al, 1
	je .LBB7_6
.LBB7_2:
	#MEMBARRIER
	mov rcx, qword ptr [rdi + 8]
	mov rsi, qword ptr [rdi + 16]
	lea r8, [rax - 1]
	lock cmpxchg	qword ptr [rdi], r8
	jne .LBB7_3
.LBB7_7:
	mov rdi, rcx
	jmp qword ptr [rsi + 8]
.LBB7_3:
	test dl, 1
	je .LBB7_6
	lock or	dword ptr [rsp - 64], 0
	mov rax, qword ptr [rdi]
	test al, 1
	je .LBB7_6
	#MEMBARRIER
	mov rcx, qword ptr [rdi + 8]
	mov rsi, qword ptr [rdi + 16]
	lea rdx, [rax - 1]
	lock cmpxchg	qword ptr [rdi], rdx
	je .LBB7_7
.LBB7_6:
	ret
