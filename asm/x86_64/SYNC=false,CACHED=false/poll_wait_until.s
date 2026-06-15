asm_poll_wait_until_asm:
	push r14
	push rbx
	push rax
	movzx eax, byte ptr [rdx]
	test al, al
	je .LBB10_2
.LBB10_1:
	xor eax, eax
	add rsp, 8
	pop rbx
	pop r14
	ret
.LBB10_2:
	mov rbx, rdx
	mov rsi, qword ptr [rsi]
	mov rdx, qword ptr [rdi]
	test dl, 3
	jne .LBB10_8
	mov rcx, qword ptr [rsi]
	mov rax, qword ptr [rsi + 8]
	mov r14, rdi
	mov rdi, rax
	call qword ptr [rcx]
	mov qword ptr [r14 + 8], rdx
	inc rax
	mov rdx, rax
	xchg qword ptr [r14], rdx
	movzx edx, byte ptr [rbx]
	test dl, dl
	je .LBB10_6
	mov rcx, r14
	mov rdx, rax
	and rdx, -2
	lock cmpxchg	qword ptr [r14], rdx
	jne .LBB10_1
	mov rdi, qword ptr [rcx + 8]
	call qword ptr [rdx + 24]
	jmp .LBB10_1
.LBB10_6:
	mov al, 1
	add rsp, 8
	pop rbx
	pop r14
	ret
.LBB10_8:
	mov rcx, rbx
	add rsp, 8
	pop rbx
	pop r14
	jmp spmc_waker::SpmcWaker<_,_>::poll_wait_until_cold
