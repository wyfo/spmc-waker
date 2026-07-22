asm_registered_asm:
	mov rdx, qword ptr [rdi]
	test dl, 1
	jne .LBB4_2
	xor eax, eax
	ret
.LBB4_2:
	mov rax, rdi
	#MEMBARRIER
	ret
