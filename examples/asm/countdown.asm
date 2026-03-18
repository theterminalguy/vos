; Countdown Loop
; Counts down from 10 to 0

start:
    ADDI R1, R0, 10     ; R1 = 10 (counter)

loop:
    SUBI R1, R1, 1      ; Decrement counter
    BNE R1, R0, loop    ; Loop if R1 != 0

    ; Counter reached 0
    HALT                ; Done
