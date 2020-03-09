# Waffle
Fast runtime for programming languages. Includes multiple GCs for different uses, concurrency support through lightweight green processes.

# Features
- Lightweight processes instead of native threads or green threads.
- Message based communication through processes (similar to Erlang)
- Bytecode optimizer and register allocator included
- Baseline JIT and Fusion JIT included (W.I.P)
- Multiple GC algorithms supported.

Read more there: [Waffle architecture overview](https://github.com/jazz-lang/Waffle/wiki/Waffle-architecture-overview)

# Concurrency
Waffle uses thing called lightweight processes, you may think of it like green threads,fibers or coroutines except that lighweight processes 
does not share heap and when message from one process sent to another it's deep copied into another process heap.
When runtime initialized it spawns process scheduler with one pool for primary processes scheduling and another pool for blocking processes. 
And another thing is timeout worker that periodically checks that process received message or sleept enough time and scheduled it back to primary pool.



# Garbage collection
Waffle provides multiple GCs for your choise:
- Incremental mark&sweep  

    Simple mark&sweep GC that uses freelist allocation and incremental collection, recommended for simple programs.
- Incremental generational mark&sweep

    Same as above, but also supports generations, recommended for almost every programs that doesn't make heap too fragmented.
 - Ieiunium GC 
  
    Generational GC with scavenging of nursery,copying intermediate space and mark&sweep of old space. Recommended for evey kind of programs.

    NOTE: Ieiunium may be slower than incremental mark&sweep but gives you fast allocation and heap defragmentation.
- Cheney's Semispace
  
    Copying GC without generations support, does very fast collection if heap not that big.
- Mark&Sweep and maybe compact
    
    This algorithm doesn't defragment heap until fragmentation is ~60%. Recommended for every kind of programs.
- On the fly GC (Concurrent Mark&Sweep)

    GC that does small pauses to get snapshot of the roots and does all work in the background.
    

# TODO
- Priority based process scheduler
- Implement JIT
- ~~Finalization support in moving collectors~~ (Done!)
