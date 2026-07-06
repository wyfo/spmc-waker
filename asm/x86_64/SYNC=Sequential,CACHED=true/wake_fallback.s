spmc_waker::SpmcWaker<S,_>::wake_fallback:
	push rax
	mov rax, rsi
	mov r9, -1
	mov r10, -3
	jmp .LBB1_1
.LBB1_9:
	mov rsi, qword ptr [rdi + 16]
	mov r11, qword ptr [rdi + 24]
	mov rax, -1
	lock cmpxchg	qword ptr [rdi], rdx
	je .LBB1_10
.LBB1_1:
	cmp rax, -1
	je .LBB1_9
	cmp rax, 1
	je .LBB1_5
	test rax, rax
	jne .LBB1_8
	xor esi, esi
	xor eax, eax
	lock cmpxchg	qword ptr [rdi], rdx
	jne .LBB1_1
	jmp .LBB1_11
.LBB1_5:
	mov eax, 1
	lock cmpxchg	qword ptr [rdi], r10
	jne .LBB1_1
	mov rax, qword ptr [rdi + 16]
	mov rsi, qword ptr [rdi + 8]
	mov qword ptr [rdi + 8], rax
	mov r11, qword ptr [rdi + 24]
	or r11, 1
	mov rax, -3
	lock cmpxchg	qword ptr [rdi], r11
	je .LBB1_12
	mov qword ptr [rdi + 8], rsi
	jmp .LBB1_1
.LBB1_8:
	lock cmpxchg	qword ptr [rdi], r9
	jne .LBB1_1
	jmp .LBB1_9
.LBB1_10:
	mov rdi, rsi
	call qword ptr [r11 + 8]
	mov sil, 1
.LBB1_11:
	mov eax, esi
	pop rcx
	ret
.LBB1_12:
	mov rdi, r8
	call qword ptr [rcx + 24]
	xor esi, esi
	mov eax, esi
	pop rcx
	ret
