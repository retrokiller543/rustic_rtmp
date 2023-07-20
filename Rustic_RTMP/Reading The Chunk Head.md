# Chunk Basic Head

The Chunk Basic Head has four different ways it can look like, they are [[#Small]], [[#Medium]] and [[#Large]]. Each type will contain the same kind of information but their sizes vary drastically. The things included in the Chunk Basic Head is the format *(or 'fmt')* and the Chunk Stream ID

## Small
	1 byte long
This is the smallest of the bunch being only 1 byte long and looks like the following:
#Packet-Diagram 
```
  0 1 2 3 4 5 6 7
 +-+-+-+-+-+-+-+-+
 |fmt|   cs id   |
 +-+-+-+-+-+-+-+-+

 Chunk basic header 1
```

## Medium
	2 byte long

#Packet-Diagram 
```
  0                   1
  0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5
 +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
 |fmt|     0     |   cs id - 64  |
 +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+

 Chunk basic header 2
```

## Large
	3 byte long

#Packet-Diagram 
```
  0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3
 +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
 |fmt|     1     |          cs id - 64           |
 +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+

 Chunk basic header 3
```