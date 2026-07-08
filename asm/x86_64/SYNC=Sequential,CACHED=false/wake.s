asm_wake_asm:
	mov rsi, qword ptr [rdi]
	test sil, 1
	jne .LBB18_1
.LBB18_5:
	ret
.LBB18_1:
	test sil, 2
	jne .LBB18_5
	lea rdx, [rsi + 2]
	mov rax, rsi
	lock cmpxchg	qword ptr [rdi], rdx
	je .LBB18_6
	add rsi, -4
	cmp rsi, -8
	jb .LBB18_5
	mov ecx, eax
	mov rsi, rax
	and ecx, 1
	jne .LBB18_1
	jmp .LBB18_5
.LBB18_6:
	lea rax, [rsi + 4]
	cmp rax, 8
	jb .LBB18_5
	lea rcx, [rsi - 1]
	mov r8, qword ptr [rdi + 8]
	mov rax, rdx
	lock cmpxchg	qword ptr [rdi], rcx
	jne .LBB18_9
	mov rdi, r8
	jmp qword ptr [rsi + 7]
.LBB18_9:
	mov rsi, rax
	mov rdx, rcx
	jmp spmc_waker::SpmcWaker<S,_>::wake_fallback
