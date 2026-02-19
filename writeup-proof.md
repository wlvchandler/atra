### Is your design deterministic, if not why did you choose that design? How do you think about determinism, what adjustments you would make? 

No. The matching algorithm itself is deterministic given the same input sequence by the system around it is non-dereministic because of wall-clock timestamps in a couple places. This wasn't a "final" design decision, rather a p0 MVP decision.

There are cases where non determinism can be used (or at least where it's defensible) e.g. some places use it to prevent queue gaming and MM fairness for RFQ systems (like: some venues intentionally randomize which MM gets "last look" under certain conditoins to prevent bad faith frontrunning) but to be clear - if this were heading to production, I would NOT deploy until determinism was guaranteed. Those special cases are like, explicit product features, not side effects of MVP naive implementations.

Specific locations are in `Order::new()` (timestamp is used as a naive tiebreaker for `Order::Ord` which means PTP across equal price orders is technically tied to when the code happened to run rather than a stable input ordering) and in the Elixir gateway for similar reasons: (not as immediately obvious but `MatchingEngine` pulls orders from ETS and sorts by `monotonic_time()`, then fans out chunks to async tasks... those chunks hit the Rust engine thru the grpc pool in whatever order the tasks happen to complete).

P1 fixes: The instrument queue (inq.rs) is actually already more of the direction of the 'classic exchange' answer of an input sequencer. It's a single writer ring buffer per instrument so that's a natural sequencing primitive, but the missing piece is that the gateway needs to write THAT into a queue (in order of course) rather than firing concurrent grpc calls.

I believe there also is technically non-determinism in mutex acquisition order in service.rs. I think under concurrent grpc requests, the OS scheduler is strictly responsible for choosing lock acquisition (which to be fair is expected) so two orders that arrive in the same microsecond can be processed in either order. Again this is fixed with an upstream sequencer.


### If we needed to reconstruct the exact book state from raw input, how would your system guarantee correctness? Again, any adjustments you would make to the existing design? 

Currently not possible because there's no durable event log. The OB and trade history are both purely in memory so if the process crashes everything's gone. Actually even without a crash, we can't currently replay because there's no input journal to replay from as well as the internal timestamping ordering that makes the system currently nondeterministic. 

P1 Fixes: a WAL at the sequencer is the obvious next step and in general the engine needs to be a pure state machine. Again, similar root cause to the first question. Order::new and Trade::new should accept timestamp from the sequencer as a param. Oh and idempotency keys. The engine doesn't currently reject duplicate seq numbers. 

Lower priority fixes would be periodic snapshots to disk + WAL truncating. Recovery would then be loading the latest snap and replaying the WAL entries after it. That's the standard approach at least in systems using e.g. redis AOF + RDB or Kafka log compaction. Neither of which I'd use for a truly performant trade engine.

> Note: I just realized the "good use case for kafka/redis" comment I left in the matching engine code is a bit misleading due to where it's placed... or rather only tells half the story. It might mislead/imply that the trade output (post-engine logging) is needed without any attention to pre-engine path i.e. an input journal. 

### Where is the true hot path in your engine, and what would you optimize first to reach production-level throughput? Just reasoning here is fine

gRPC request -> `place_order()` -> `match_order()` -> [price level walk + fills] -> trade recording -> gRPC response

#### What I'd optimize

**P0:** Biggest bottleneck is the Mutex prior to `place_order()` I believe. Every operation currently contends on a single one. Under actual load this is really bad. The lock hold time spans the entire matching operation (incl. all fills + trade recording) which means the system is serialized to 1 operation at a time regardless of core count. Fix is to just remove the mutex from the hot path entirely and utilize the Instrument Queue to dedicate a thread per instrument. Then the matching thread drains the queue and can process sequentially and no locks are needed because it's the sole owner of the instrument's state. Actually this would unbind grpc from the engine and allow it to be just an ingestion layer (as intended).

**P1:** Reduce all the cloning overhead in the matching loop. Way too many allocations currently. 

**P2:** I did not know this at time of writing but reviewing today I realized that `Decimal` is 128 bit *software* (!!!) arithmetic so the arithmetic is way more expensive than native integer ops. Fix: update the fill loop to use u64 fixed the same way the InstrumentQueue does. Even without profiling I think this would be a ~5-10x speedup of the inner loop. 

**P?:** If this were going to be a production level OB/engine I'd use a specialized data structure and not rely on the Rust provided ones. BTreeMap is logarithmic lookup and for hot pathing "get the best price", in practice an OB RARELY needs more than a few hundred active price levels so the "right" way is to take some care and optimize a structure for cache locality. This is only relevant at really high throughput, but worth mentioning

**P?:** Get rid of timestamping altogether as already discussed BUT a quick win prior to that would just be to batch the clock read syscall because we only need to make that call 1x per match cycle. Then we could stamp the trades from that cycle with the same value which is actually more semantically correct anyways sincetrades from one aggressor order logically happen at the same instant


Fortunately most of this is cleanup + overhead removal from MVP state since the matching algorithm logic itself & the PTP semantics are right. 

> (PS: Rust is interesting but I am not convinced it's is the best choice for a high performance engine. The ownership model eliminates a class of bugs but I haven't yet seen it outperform C++ (where a well-designed memory pool/slab allocator like openACR erases ownership leaks anyways). Happy to dig into that conversation)

Open issues tracking the above and other known gaps are on https://github.com/wlvchandler/atra/issues