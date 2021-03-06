global interrupt_handlers

section .text
bits 64

%macro pushAll 0
        push rax
        push rcx
        push rdx
        push r8
        push r9
        push r10
        push r11
        ;; These two are caller-saved on x86_64!
        push rdi
        push rsi
%endmacro

%macro popAll 0
        pop rsi
        pop rdi
        pop r11
        pop r10
        pop r9
        pop r8
        pop rdx
        pop rcx
        pop rax
%endmacro

%macro ISR_NOERRCODE 1
	[global isr%1]
	isr%1:
		push 0		; dummy error code
		push %1		; interrupt number
		jmp isr_common
%endmacro

%macro ISR_ERRCODE 1
	[global isr%1]
	isr%1:
		push %1		; interrupt number
		jmp isr_common
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
ISR_ERRCODE 10
ISR_ERRCODE 11
ISR_ERRCODE 12
ISR_ERRCODE 13
ISR_ERRCODE 14
ISR_NOERRCODE 16
ISR_ERRCODE 17
ISR_NOERRCODE 18
ISR_NOERRCODE 19


ISR_NOERRCODE 32
%assign i 33
%rep    224
ISR_NOERRCODE i
%assign i i+1
%endrep

extern isr_handler

isr_common:
        pushAll

        mov rdi, rsp            ; Pass pointer to interrupt data.

        call isr_handler

        popAll
        add rsp, 16             ; Remove err code & interrupt ID.
        iretq

        section .rodata
        
section .data
interrupt_handlers:
        dq isr0
        dq isr1
        dq isr2
        dq isr3
        dq isr4
        dq isr5
        dq isr6
        dq isr7
        dq isr8
        dq 0
        dq isr10
        dq isr11
        dq isr12
        dq isr13
        dq isr14
        dq 0
        dq isr16
        dq isr17
        dq isr18
        dq isr19
        dq 0
        dq 0
        dq 0
        dq 0
        dq 0
        dq 0
        dq 0
        dq 0
        dq 0
        dq 0
        dq 0                    ; int_entry_30
        dq 0
%assign i 32
%rep    224
        dq isr%+i
%assign i i+1
%endrep