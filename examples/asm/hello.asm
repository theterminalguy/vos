; Hello World Program
; Writes "Hello" to the display device

; Display base address: 0x80000000
; Each character is 2 bytes (char + attribute)

start:
    ; Load display base address
    LUI R1, 0x8000      ; R1 = 0x80000000 (display base)

    ; Write 'H' (0x48)
    ADDI R2, R0, 72     ; R2 = 'H'
    SB R2, R1, 0        ; Store byte at display[0]

    ; Write 'e' (0x65)
    ADDI R2, R0, 101    ; R2 = 'e'
    SB R2, R1, 2        ; Store byte at display[2]

    ; Write 'l' (0x6C)
    ADDI R2, R0, 108    ; R2 = 'l'
    SB R2, R1, 4        ; Store byte at display[4]

    ; Write 'l' (0x6C)
    SB R2, R1, 6        ; Store byte at display[6]

    ; Write 'o' (0x6F)
    ADDI R2, R0, 111    ; R2 = 'o'
    SB R2, R1, 8        ; Store byte at display[8]

    HALT                ; Stop execution
