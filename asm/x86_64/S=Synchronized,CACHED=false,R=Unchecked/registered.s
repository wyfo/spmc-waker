asm_registered_asm:
	mov rax, rdi
	mov rcx, qword ptr [rsi]
	mov rdx, rcx
	test cl, 1
	jne .LBB4_2
	lock or	dword ptr [rsp - 64], 0
	mov rdx, qword ptr [rsi]
	mov dil, 2
	test dl, 1
	je .LBB4_3
.LBB4_2:
	and ecx, 1
	#MEMBARRIER
	xor cl, 1
	mov qword ptr [rax], rsi
	mov qword ptr [rax + 8], rdx
	mov edi, ecx
.LBB4_3:
	mov byte ptr [rax + 16], dil
	ret
