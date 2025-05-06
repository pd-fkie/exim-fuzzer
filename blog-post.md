# How to build a high-performance network fuzzer with LibAFL and libdesock

- existing network fuzzing solutions struggle on all fronts
- fuzzing speed is a big problem because they use either real network connections or
  emulation/virtualization for snapshot-based fuzzing
- both come with a huge overhead
- and they struggle with deeply exploring the target, since most of the tools out there are
  built on top of AFL
- AFL is a fuzzer for binary file formats while most network protocols are text-based
- for the vulnerability research we do at work we built a high-performance network fuzzer
  that tackles these problems and would like to detail its setup in this post

- the first thing we addressed was the problem of input generation
    - we had to come up with our own mutators and input representation that works with text-based protocols
    - for that we used LibAFL, a library made for building custom fuzzers, which made this very easy
- the second problem we approached was how to actually feed inputs to network applications
    - for this we chose to "desocket" the applications with libdesock and serve the individual
      packets over a shared memory channel
- we compared our tool to AFLNet, arguably the most popular network fuzzer at the time of writing this
- found that our setup gave us 42x performance boost and enabled us to get a lot more coverage (?)
- we were able to uncover one new vulnerability in already heavily fuzzed software
- if you'd like to check the source code out yourself, you can find it [here]() on Github

## Writing a custom fuzzer
- If we want our fuzzer to find bugs we need to emancipate ourselves from AFL
- Let's have a look at this message exchange in the FTP protocol that is used to establish
  a connection for data transmission:
  ```
  > PORT 192,168,1,178,12,34
  < 200 Okay
  ```
- What could be sensible ways to mutate this message?
    - Perhaps we could replace the numbers in the command with other numbers
      like `-1`, `127`, `4294967295`, etc.
    - or we could replace the `PORT` command with another command
    - or we could try if `PORT` takes other arguments by inserting random text separated by spaces
- Either way, we need meaningful text-based mutations and an input representation that enables them
- Our approach was to represent individual messages of a protocol as a stream of tokens, i.e. a `TokenStream`
- Where a `Token` is either a number, whitespace or normal text
- The message above is represented as
  ```
  Text("PORT"), Whitespace(" "), Number("192"), Text(","), Number("168"), [...], Whitespace("\r\n")
  ```
- This enables mutators to have some sense of "awareness", i.e. the ability to operate on actually meaningful, semantic units of a message
  instead of random bytes
    - We can mutate the individual numbers of the PORT command
    - We can duplicate/delete/crossover entire arguments to commands
- and much more while still being low-level enough to just flip some bits
- Then we can go to the next level of our input representation
- network protocols are a back and forth of multiple messages, so our input needs to be a sequence of `TokenStream`s, not just a single one
- in rust this is very easy to implement
- we simply define our data types...
  ```rs
    enum TextToken {
        Number(Vec<u8>),
        Whitespace(Vec<u8>),
        Text(Vec<u8>),
        Constant(Vec<u8>), // for text-tokens that come straight from the dictionary
    }

    struct TokenStream(Vec<TextToken>);

    struct PacketBasedInput(Vec<TokenStream>);
  ```
  ...and plug the `PacketBasedInput` into our fuzzer without hassle, thanks to LibAFL

## Implementing fast message passing
- now we have a good method for input generation but we don't want to sacrifice efficiency for effectiveness
- so we need a fast method of transmitting fuzz input to the application
- this is where our desocketing library [libdesock]() comes into play
- libdesock enables us to customize what happens when a network application sends or receives messages over the network
    - it hooks all network operations that the application normally would delegate to the kernel and
      emulates them in user-space
    - when the application tries to read data from a network connection, it redirects the reads to some
      other input channel, e.g. stdin
- in our case, the fuzzer sets up a shared memory channel
- in each iteration libdesock reads the input from the shm channel
- this is possible due to the "hooks" feature of libdesock
- we can simply implement our own "input hook" and "output hook" that determine what happens on a network
  send/recv
- in less than 50 lines of C code we attach to the shared memory channel and read from it:
  ```c
    // Set by the fuzzer in each iteration:
    typedef struct {
        size_t cursor;
        size_t size; // length of fuzz input
        char data[]; // fuzz input
    } PacketBuffer;
    
    PacketBuffer* packet_buffer = /* points to shm */;
    
    // Called whenever a read on a network connection occurs.
    // We place `size` bytes from the shm channel into `buf`.
    size_t hook_input (char* buf, size_t size) {
        size_t cursor = packet_buffer->cursor;
        size_t rem_bytes = packet_buffer->size - cursor;
        
        size = (size < rem_bytes) ? size : rem_bytes;
        
        memcpy(buf, &packet_buffer->data[cursor], size);
        packet_buffer->cursor += size;
        
        return size;
    }
  ```
- You might ask yourself how multiple messages are handled since we are just dealing with one
  flat memory buffer in shm
- the TokenStream's from the `PacketBasedInput` got concatenated, separated by the string `--------`
- libdesock automatically detects this separator and feeds this packets individually to the application

## fuzzing actually
- the rest of the fuzzer is kept very simple: no powerschedules, no mutation scheduling, no
  compare coverage, no extra feedback about the protocol state of the target, etc.
- results
    - compare our fuzzer to AFLNet
    - with AFLNet we got around ~30 exec/s on one core and were not able to utilize
      multiple cores
    - with our fuzzer we got around ~1200 exec/s pro core and were able to utilize
      multicore-fuzzing with linear scaling
    - concrete stats: we were fuzzing @ 15k exec/s on 12 cores
    - TODO: coverage evaluation
    - enabled us to squeeze multiple bugs out of heavily vetted code
- lesson
    - as more and more peolple are fuzzing, stock-solutions like
      AFL become less and less effective
    - if you want to find bugs don't just rely on existing fuzzers
    - fuzzing solutions that give you an edge over the competition are not
      that far away
    - putting a little bit of effort into writing custom fuzzers can
      give a big payoff
    - in our opinion _the_ way to go for future vulnerability research
