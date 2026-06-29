asm_wake_asm:
	mov rsi, qword ptr [rdi]
	test sil, 1
	jne .LBB14_1
.LBB14_5:
	ret
.LBB14_1:
	test sil, 2
	jne .LBB14_5
	lea rdx, [rsi + 2]
	mov rax, rsi
	lock cmpxchg	qword ptr [rdi], rdx
	je .LBB14_6
	add rsi, -4
	cmp rsi, -8
	jb .LBB14_5
	mov ecx, eax
	mov rsi, rax
	and ecx, 1
	jne .LBB14_1
	jmp .LBB14_5
.LBB14_6:
	lea rax, [rsi + 4]
	cmp rax, 8
	jb .LBB14_5
	lea rcx, [rsi - 1]
	mov r8, qword ptr [rdi + 8]
	mov rax, rdx
	lock cmpxchg	qword ptr [rdi], rcx
	jne .LBB14_9
	mov rdi, r8
	jmp qword ptr [rsi + 7]
.LBB14_9:
	mov rsi, rax
	mov rdx, rcx
	jmp spmc_waker::SpmcWaker<S,_>::wake_fallback
