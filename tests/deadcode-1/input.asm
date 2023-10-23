-- HUMAN RESOURCE MACHINE PROGRAM --

a:
    INBOX   
    JUMP     b
    OUTBOX  
    JUMP     a
    OUTBOX  
b:
    OUTBOX  
    JUMP     c
c:

