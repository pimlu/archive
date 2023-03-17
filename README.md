## Archive

This is a browser game written in Rust which is left here in a WIP state. It uses WebRTC for UDP-like server-client communication, and WebGPU for realtime rendering. It was originally intended to be a reimplementation of a web game called surviv.io that would vanquish bots.

It was maybe too ambitious. I was trying to delta encode the server state and then send it via [entropy coding](https://docs.rs/constriction/latest/constriction/) to squeeze the last bits out, but I didn't actually have a working game yet lol.

But what is here is a nifty communication layer over WebRTC which supports alternative transport like WebSockets as a fallback. And some nifty WebGPU rendering/browser builds, like I always do.

### Why is it called Archive?
surviv.io is all about shooting and dodging bullets flying all over the place. The band Archive has a song called Bullets.
