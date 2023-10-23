-- HUMAN RESOURCE MACHINE PROGRAM --

a:
b:
    COPYFROM 14
    COPYTO   13
    INBOX    
    COPYTO   [14]
c:
    BUMPDN   13
    JUMPN    d
    COPYFROM [14]
    SUB      [13]
    JUMPZ    a
    JUMP     c
d:
    COPYFROM [14]
    OUTBOX
    BUMPUP   14
    JUMP     b

