-- HUMAN RESOURCE MACHINE PROGRAM --

    JUMP     b
a:
    COPYFROM [14]
    OUTBOX   
    BUMPUP   14
b:
c:
    COPYFROM 14
    COPYTO   13
    INBOX    
    COPYTO   [14]
d:
    BUMPDN   13
    JUMPN    a
    COPYFROM [14]
    SUB      [13]
    JUMPZ    c
    JUMP     d

