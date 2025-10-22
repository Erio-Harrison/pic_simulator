LIST    P=12F629
    #include <p12f629.inc>

    ORG 0x000
    GOTO    INIT

    ORG 0x004
    RETFIE

INIT:
    BSF     STATUS, RP0
    MOVLW   0x00
    MOVWF   TRISIO
    BCF     STATUS, RP0
    CLRF    GPIO
    CLRWDT

MAIN_LOOP:
    MOVLW   0x01
    MOVWF   GPIO
    CALL    DELAY
    
    MOVLW   0x02
    MOVWF   GPIO
    CALL    DELAY
    
    MOVLW   0x04
    MOVWF   GPIO
    CALL    DELAY
    
    MOVLW   0x10
    MOVWF   GPIO
    CALL    DELAY
    
    MOVLW   0x20
    MOVWF   GPIO
    CALL    DELAY
    
    MOVLW   0x37
    MOVWF   GPIO
    CALL    DELAY
    
    MOVLW   0x00
    MOVWF   GPIO
    CALL    DELAY
    
    GOTO    MAIN_LOOP

DELAY:
    MOVLW   0xFF
    MOVWF   0x20
DELAY_LOOP:
    CLRWDT
    DECFSZ  0x20, F
    GOTO    DELAY_LOOP
    RETURN

    END