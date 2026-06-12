uc_has_waker_registered:
	mov rax, qword ptr [rdi]
	and al, 1
	ret
