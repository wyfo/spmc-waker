spmc_waker::SpmcWaker<S,_>::wake_registered_cold:
.LBB4_1:
	test sil, 2
	jne .LBB4_5
	lea rcx, [rsi + 2]
	mov rax, rsi
	lock cmpxchg	qword ptr [rdi], rcx
	je .LBB4_8
	add rsi, -4
	cmp rsi, -8
	jb .LBB4_5
	mov ecx, eax
	mov rsi, rax
	and ecx, 1
	jne .LBB4_1
.LBB4_5:
	test dl, 1
	jne .LBB4_7
	lock or	dword ptr [rsp - 64], 0
	mov rsi, qword ptr [rdi]
	mov dl, 1
	test sil, 1
	jne .LBB4_1
	jmp .LBB4_7
.LBB4_8:
	lea rax, [rsi + 4]
	cmp rax, 8
	jae .LBB4_9
.LBB4_7:
	ret
.LBB4_9:
	lea rdx, [rsi - 1]
	mov r8, qword ptr [rdi + 8]
	mov rax, rcx
	lock cmpxchg	qword ptr [rdi], rdx
	jne .LBB4_11
	mov rdi, r8
	jmp qword ptr [rsi + 7]
.LBB4_11:
	mov rsi, rax
	mov rcx, rdx
	jmp spmc_waker::SpmcWaker<S,_>::wake_fallback
