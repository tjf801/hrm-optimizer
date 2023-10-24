-- HUMAN RESOURCE MACHINE PROGRAM --

    JUMP b
a:
    OUTBOX
    INBOX
    OUTBOX
b:
    INBOX
    JUMP c
    INBOX
    JUMP a
    JUMP d
c:
    OUTBOX
    INBOX
    JUMP e
d:
    OUTBOX
e:
    OUTBOX
    JUMP f
f:

