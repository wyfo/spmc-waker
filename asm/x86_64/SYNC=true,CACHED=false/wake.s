asm_wake_asm:
	mov rcx, qword ptr [rdi]
	mov rsi, rcx
	test cl, 1
	jne .LBB14_2
	lock or	dword ptr [rsp - 64], 0
	mov rsi, qword ptr [rdi]
	test sil, 1
	je .LBB14_9
.LBB14_2:
	xor cl, 1
.LBB14_3:
	test sil, 2
	jne .LBB14_7
	lea rdx, [rsi + 2]
	mov rax, rsi
	lock cmpxchg	qword ptr [rdi], rdx
	je .LBB14_10
	add rsi, -4
	cmp rsi, -8
	jb .LBB14_7
	mov edx, eax
	mov rsi, rax
	and edx, 1
	jne .LBB14_3
.LBB14_7:
	test cl, 1
	jne .LBB14_9
	lock or	dword ptr [rsp - 64], 0
	mov rsi, qword ptr [rdi]
	mov cl, 1
	test sil, 1
	jne .LBB14_3
	jmp .LBB14_9
.LBB14_10:
	lea rax, [rsi + 4]
	cmp rax, 8
	jae .LBB14_11
.LBB14_9:
	ret
.LBB14_11:
	lea rcx, [rsi - 1]
	mov r8, qword ptr [rdi + 8]
	mov rax, rdx
	lock cmpxchg	qword ptr [rdi], rcx
	jne .LBB14_13
	mov rdi, r8
	jmp qword ptr [rsi + 7]
.LBB14_13:
	mov rsi, rax
	mov rdx, rcx
	jmp spmc_waker::SpmcWaker<_,_>::wake_fallback
