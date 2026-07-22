asm_registered_asm:
	mov rdx, qword ptr [rdi]
	test dl, 1
	jne .LBB3_2
	xor eax, eax
	ret
.LBB3_2:
	mov rax, rdi
	#MEMBARRIER
	ret
