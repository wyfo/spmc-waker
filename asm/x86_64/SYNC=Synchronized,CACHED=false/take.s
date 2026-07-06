asm_take_asm:
	push rax
	mov rsi, qword ptr [rdi]
	mov rcx, rsi
	test sil, 1
	jne .LBB14_3
	lock or	dword ptr [rsp - 64], 0
	mov rcx, qword ptr [rdi]
	test cl, 1
	je .LBB14_2
.LBB14_3:
	xor sil, 1
.LBB14_4:
	test cl, 2
	jne .LBB14_8
	lea rdx, [rcx + 2]
	mov rax, rcx
	lock cmpxchg	qword ptr [rdi], rdx
	je .LBB14_11
	add rcx, -4
	cmp rcx, -8
	jb .LBB14_8
	mov edx, eax
	mov rcx, rax
	and edx, 1
	jne .LBB14_4
.LBB14_8:
	test sil, 1
	jne .LBB14_10
	lock or	dword ptr [rsp - 64], 0
	mov rcx, qword ptr [rdi]
	mov sil, 1
	test cl, 1
	jne .LBB14_4
.LBB14_10:
	xor ecx, ecx
.LBB14_14:
	mov rax, rcx
	mov rdx, r8
	pop rcx
	ret
.LBB14_11:
	lea rax, [rcx + 4]
	cmp rax, 8
	jae .LBB14_12
.LBB14_2:
	xor ecx, ecx
	mov rax, rcx
	mov rdx, r8
	pop rcx
	ret
.LBB14_12:
	dec rcx
	mov r8, qword ptr [rdi + 8]
	mov rax, rdx
	lock cmpxchg	qword ptr [rdi], rcx
	je .LBB14_14
	mov rsi, rax
	mov rdx, rcx
	call spmc_waker::SpmcWaker<S,_>::wake_fallback
	mov rcx, rax
	mov r8, rdx
	mov rax, rcx
	mov rdx, r8
	pop rcx
	ret
