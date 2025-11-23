# CyDnA Core Protocol - Attribution

## Primary Creator & Designer

**Shayan Golmezerji**

All credit for the design, specification, and implementation of the CyDnA protocol belongs to Shayan Golmezerji.

## About CyDnA

CyDnA (Cyber-Physical Data Network Architecture) is an ultra-low latency UDP messaging core protocol designed for time-critical communication between Sensor Layer (S-Layer) and Gateway Layer (G-Layer) hardware systems.

## Engineering Specification

The protocol was developed according to the final engineering specification for Phase 1, which includes:

- Ultra-low latency custom UDP messaging core
- Zero-copy serialization using rkyv framework
- Minimal external dependencies (std + tokio only)
- Production-grade security with Ed25519 signatures and Blake2b hashing
- Custom lightweight ACK/NACK reliability mechanism
- Exponential backoff for retransmission strategy

## License

CyDnA Core is released under the Creative Commons Attribution 4.0 International (CC BY 4.0) license.

This means you are free to:
- Use this work for any purpose (commercial or non-commercial)
- Modify and adapt the code
- Distribute and share the software

As long as you:
- Provide attribution to Shayan Golmezerji
- Link to this license
- Indicate what changes were made

## Usage Attribution

When using or referencing CyDnA Core, please include:

```
CyDnA Core Protocol
Created by Shayan Golmezerji
Licensed under CC BY 4.0 (Creative Commons Attribution 4.0 International)
```

## Contact

For inquiries about the CyDnA protocol, reach out to Shayan Golmezerji.

---

**Last Updated:** November 23, 2025
