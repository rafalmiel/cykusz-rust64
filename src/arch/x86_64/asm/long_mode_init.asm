global long_mode_start

section .text
bits 64
long_mode_start:
	extern rust_main
	call setup_SSE
	mov rsp, stack_top
	mov rax, rust_main
	jmp rax

	mov rax, 0x2f592f412f4b2f4f
	mov qword [0xb8000], rax
	hlt

; Check for SSE and enable it. If it's not supported throw error "a".
setup_SSE:
	; check for SSE
	mov rax, 0x1
	cpuid
	test edx, 1<<25
	jz .no_SSE

	; enable SSE
	mov rax, cr0
	and ax, 0xFFFB      ; clear coprocessor emulation CR0.EM
	or ax, 0x2          ; set coprocessor monitoring  CR0.MP
	mov cr0, rax
	mov rax, cr4
	or ax, 3 << 9       ; set CR4.OSFXSR and CR4.OSXMMEXCPT at the same time
	mov cr4, rax

	ret
.no_SSE:
	mov al, "a"
	jmp error

error:
	mov rbx, 0x4f4f4f524f524f45
	mov [0xb8000], rbx
        mov rbx, 0x4f204f204f3a4f52
	mov [0xb8008], rbx
	mov byte [0xb800e], al
	hlt
	jmp error

	section .stack
	stack_bottom:
		resb 4096*2
	stack_top:
