-- HUMAN RESOURCE MACHINE PROGRAM --

a:
    INBOX   
    COPYTO   12
b:
    COPYFROM [12]
    OUTBOX  
    BUMPUP   12
    COPYFROM [12]
    JUMPN    a
    COPYTO   12
    JUMP     b


