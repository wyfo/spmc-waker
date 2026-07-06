asm_take_asm:
	mov rdx, qword ptr [rdi]
	test dl, 1
	jne .LBB14_3
.LBB14_1:
	xor ecx, ecx
.LBB14_2:
	mov rax, rcx
	mov rdx, r8
	ret
.LBB14_3:
	xor ecx, ecx
.LBB14_4:
	test dl, 2
	jne .LBB14_8
	lea rsi, [rdx + 2]
	mov rax, rdx
	lock cmpxchg	qword ptr [rdi], rsi
	je .LBB14_9
	add rdx, -4
	cmp rdx, -8
	jb .LBB14_8
	mov esi, eax
	mov rdx, rax
	and esi, 1
	jne .LBB14_4
	jmp .LBB14_2
.LBB14_8:
	mov rax, rcx
	mov rdx, r8
	ret
.LBB14_9:
	lea rax, [rdx + 4]
	cmp rax, 8
	jb .LBB14_1
	dec rdx
	mov r8, qword ptr [rdi + 8]
	mov rax, rsi
	lock cmpxchg	qword ptr [rdi], rdx
	jne .LBB14_13
	mov rcx, rdx
	mov rax, rcx
	mov rdx, r8
	ret
.LBB14_13:
	push rax
	mov rsi, rax
	mov rcx, rdx
	call spmc_waker::SpmcWaker<S,_>::wake_fallback
	mov rcx, rax
	mov r8, rdx
	add rsp, 8
	mov rax, rcx
	mov rdx, r8
	ret
