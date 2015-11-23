%macro pushAll 0
        push rbp
	push r15
	push r14
	push r13
	push r12
	push r11
	push r10
	push r9
	push r8
	push rsi
	push rdi
	push rdx
	push rcx
	push rbx
	push rax
%endmacro

%macro popAll 0
        pop rax
	pop rbx
	pop rcx
	pop rdx
	pop rdi
	pop rsi
	pop r8
	pop r9
	pop r10
	pop r11
	pop r12
	pop r13
	pop r14
	pop r15
	pop rbp
%endmacro

%macro ISR_NOERRCODE 1
	[global isr%1]
	isr%1:
		push 0		; dummy error code
		push %1		; interrupt number
		jmp isr_common_stub
%endmacro

%macro ISR_ERRCODE 1
	[global isr%1]
	isr%1:
		push %1		; interrupt number
		jmp isr_common_stub
%endmacro

%macro IRQ 2
	[global irq%1]
	irq%1:
		push 0		; interrupt number
		push %2		; interrupt number
		jmp irq_common_stub
%endmacro

ISR_NOERRCODE 0
ISR_NOERRCODE 1
ISR_NOERRCODE 2
ISR_NOERRCODE 3
ISR_NOERRCODE 4
ISR_NOERRCODE 5
ISR_NOERRCODE 6
ISR_NOERRCODE 7
ISR_ERRCODE 8
ISR_NOERRCODE 9
ISR_ERRCODE 10
ISR_ERRCODE 11
ISR_ERRCODE 12
ISR_ERRCODE 13
ISR_ERRCODE 14
ISR_NOERRCODE 15
ISR_NOERRCODE 16
ISR_NOERRCODE 17
ISR_NOERRCODE 18
ISR_NOERRCODE 19
ISR_NOERRCODE 20
ISR_NOERRCODE 21
ISR_NOERRCODE 22
ISR_NOERRCODE 23
ISR_NOERRCODE 24
ISR_NOERRCODE 25
ISR_NOERRCODE 26
ISR_NOERRCODE 27
ISR_NOERRCODE 28
ISR_NOERRCODE 29
ISR_NOERRCODE 30
ISR_NOERRCODE 31

IRQ 0, 32
IRQ 1, 33
IRQ 2, 34
IRQ 3, 35
IRQ 4, 36
IRQ 5, 37
IRQ 6, 38
IRQ 7, 39
IRQ 8, 40
IRQ 9, 41
IRQ 10, 42
IRQ 11, 43
IRQ 12, 44
IRQ 13, 45
IRQ 14, 46
IRQ 15, 47

global irq_spurious
irq_spurious:
	iretq

extern isr_handler

isr_common_stub:
	pushAll

	mov rax, ds
	push rax		; save data segment descriptor

	mov rax, 0x10
	mov ds, rax
	mov es, rax
	mov fs, rax
	mov gs, rax

	push rsp		; pointer to stack frame

	call isr_handler

	add rsp, 8		; remove stack pointer from stack

	pop rbx			; restore data segment descriptor
	mov ds, rbx
	mov es, rbx
	mov fs, rbx
	mov gs, rbx

	popAll
	
	add rsp, 16		; remove int num and error code from stack
	iretq

extern irq_handler

irq_common_stub:
	pushAll

	mov ax, ds
	push rax		; save data segment descriptor

	mov rax, 0x10
	mov ds, rax
	mov es, rax
	mov fs, rax
	mov gs, rax

	push rsp		; pointer to stack frame

	call irq_handler

	add rsp, 8		; remove stack pointer from stack

	pop rbx			; restore data segment descriptor
	mov ds, rbx
	mov es, rbx
	mov fs, rbx
	mov gs, rbx

	popAll
	
	add rsp, 16		; remove int num and error code from stack
	iretq

	global idt_flush

global idt_flush
idt_flush:
	mov rax, [rsp + 8]
	lidt [rax]
	ret