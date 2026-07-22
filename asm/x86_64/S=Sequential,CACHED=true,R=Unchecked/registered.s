asm_registered_asm:
	mov rdx, qword ptr [rdi]
	xor eax, eax
	test dl, 1
	cmovne rax, rdi
	ret
