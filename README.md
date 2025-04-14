# Project for learning Rust and the epoll way of handling sockets

This is a workspace containing client, server and library code making chatting over a network possible. We are basically reinventing the wheel here :)
The server part of chatter is a server that handles multiple connections/clients using epoll. The protocol is JOW (JSON Over the Wire).
