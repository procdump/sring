This repo consists of:

- A Solana Anchor program that implements a frame ring for enqueuing network
frames on chain that can eventually be dequeued.

- A simple tun example to take advantage of the frame ring to enqueue/dequeue packets on chain.
This makes it possible to pipe all the traffic from a tun interface through Solana (using litesvm) and measure bandwidth with tools like iperf3.

