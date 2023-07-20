# The Handshake
This is the first step of the puzzle and requires a series of confirmation of receiving  a predetermined data. 

## Client Part 1

The client sends the following data:

### *C0* 
	1 byte
This is The Version (in our case always "3") and looks as the following: 
#Packet-Diagram
```
 0 1 2 3 4 5 6 7
 +-+-+-+-+-+-+-+-+
 |    version    |
 +-+-+-+-+-+-+-+-+
		C0 
```

### *C1*
	1536 octets long
This is the timestamp packet of the client, it also includes some extra fields that are as follows:
#Packet-Diagram
```
  0                   1                   2                   3
  0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
 +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
 |                        time (4 bytes)                         |
 +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
 |                        zero (4 bytes)                         |
 +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
 |                        random bytes                           |
 +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
 |                        random bytes                           |
 |                           (cont)                              |
 |                            ....                               |
 +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+

	                         C1 bits
```

- The timestamp itself is made of 4 bytes
- This is followed by 4 bytes that are all Zero
- After that we fill the rest of the packet with random data that does not matter.

## Server Part 1

The Server starts by reading the two packet from [Client Part 1](#Client%20Part%201). Then it makes these packets :

### *S0* 
	1 byte
The S0 packet should be the exact same as the [C0 Packet](#C0). *However if it does not look the same the server should not accept the connection!* 
#Packet-Diagram
```
 0 1 2 3 4 5 6 7
 +-+-+-+-+-+-+-+-+
 |    version    |
 +-+-+-+-+-+-+-+-+
		S0 
```

### *S1*
	1536 octets long
This should be similar to the [C1 packet](#C1) in structure, but contain the timestamp of the server instead.
#Packet-Diagram
```
  0                   1                   2                   3
  0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
 +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
 |                        time (4 bytes)                         |
 +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
 |                        zero (4 bytes)                         |
 +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
 |                        random bytes                           |
 +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
 |                        random bytes                           |
 |                           (cont)                              |
 |                            ....                               |
 +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+

	                         S1 bits
```

## *S2 and C2*
	1536 octets long
The Server **MUST** have the [C1 Packet](#C1) from the Client in order to make this packet and the Client **MUST** have the [S1 Packet](#S1) from the Server. The Server receives the **C2 Packet** and checks its content if it matches the content of the [S1 Packet](#S1)
#Packet-Diagram
```
  0                   1                   2                   3
  0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
 +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
 |                        time (4 bytes)                         |
 +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
 |                       time2 (4 bytes)                         |
 +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
 |                         random echo                           |
 +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
 |                         random echo                           |
 |                            (cont)                             |
 |                             ....                              |
 +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+

                          C2 and S2 bits
```

Back to [[RTMP]] 