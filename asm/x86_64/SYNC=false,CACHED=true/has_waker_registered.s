asm_has_waker_registered_asm:
	mov rax, qword ptr [rdi]
	and al, 1
	ret
