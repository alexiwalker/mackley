# Mackley Message Queue

a small message queue that uses a custom protocol on top of TCP to send and receive messages, and monitor the server

## Protocol

see the "String encoding" section below for details on ```%string``` encoding

MMQP|versionMajor.versionMinor|commandEnumChar|%username:%password|...message type dependant

example message


send a message
MMQP|0.1|M|%username:%password|%queue|%messageGroupId|%message

poll for messages
MMQP|0.1|P|%username:%password|%queue|%messageGroupId|%messageGroupId or 0|

----



### String encoding

strings in message formats descriptions are preceded by a % (percent) character. this character is not in the message itself and denotes the encoding scheme.

The scheme is as follows:

The length of the string is read and precedes the string, but the length itself has additional encoding.

The length is a usize (64bit unsigned integer). However, most messages will not require all 8 bytes of the usize.  To save space,
the length itself is preceded by a single byte that indicates the number of bytes used to encode the length.

for example, a string length of 63456 is encoded as:

1111_0111___1110_0000 (spacing added for readability)

requires 2 bytes, so the leading character is

0000_0010 (2)
followed by
1111_0111 (high byte)
1110_0000 (low byte)

upon reconstructing, fill a u8 array with zeroes, then read with the following code:

where size is the number of bytes used to encode the length (where the cursor starts the function)
cursor is the current position in the bytes of the message/string (assumes zero in this case)

    let size_for_size = message_binary[*cursor];
    *cursor += 1;
    let mut bytes: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
    for i in 0..size_for_size {
       bytes[8 - (size_for_size - i) as usize] = message_binary[*cursor];
       *cursor += 1
    }
    let actual_size = usize::from_be_bytes(bytes);

starting at the final byte of the array, subtract the index from the size to get the actual usize byte index to set, which is the current message binary cursor
increment the cursor, and loop again until the end.

Once the string length has been determined, read that number of bytes from the provided binary. This is the string itself.


all ```%string```s are encoded in this way and will be treated as utf8. Empty strings are encoded as a single byte with value 0. They cannot be omitted.

