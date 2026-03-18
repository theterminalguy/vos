; Fibonacci Sequence Calculator
; Calculates first 10 Fibonacci numbers
; Result stored in registers R3-R12

start:
    ; Initialize first two Fibonacci numbers
    ADDI R1, R0, 0      ; F(0) = 0
    ADDI R2, R0, 1      ; F(1) = 1

    ; Store F(0) and F(1)
    ADD R3, R0, R1      ; R3 = F(0)
    ADD R4, R0, R2      ; R4 = F(1)

    ; Calculate F(2)
    ADD R5, R1, R2      ; R5 = F(2) = F(0) + F(1)
    ADD R1, R0, R2      ; Shift: R1 = F(1)
    ADD R2, R0, R5      ; Shift: R2 = F(2)

    ; Calculate F(3)
    ADD R6, R1, R2      ; R6 = F(3)
    ADD R1, R0, R2
    ADD R2, R0, R6

    ; Calculate F(4)
    ADD R7, R1, R2      ; R7 = F(4)
    ADD R1, R0, R2
    ADD R2, R0, R7

    ; Calculate F(5)
    ADD R8, R1, R2      ; R8 = F(5)
    ADD R1, R0, R2
    ADD R2, R0, R8

    ; Calculate F(6)
    ADD R9, R1, R2      ; R9 = F(6)
    ADD R1, R0, R2
    ADD R2, R0, R9

    ; Calculate F(7)
    ADD R10, R1, R2     ; R10 = F(7)
    ADD R1, R0, R2
    ADD R2, R0, R10

    ; Calculate F(8)
    ADD R11, R1, R2     ; R11 = F(8)
    ADD R1, R0, R2
    ADD R2, R0, R11

    ; Calculate F(9)
    ADD R12, R1, R2     ; R12 = F(9)

    HALT                ; Done
