# CruxC2
> If any man will come after me, let him deny himself, and take up his **cross**, and follow me\
> -- Matthew 16:24

CruxC2 is a lightweight C2 Framework for use with Penetration Testing, CTFs, and Red Team engagements. It consists of these two components:
1. CruxAgent: Executed on the target system and connects back to the CruxServer.
2. CruxServer: Listener on which operators can issue commands to CruxAgents

## Implemented & Planned Features:
The following features are currently implemented:
- [x] Interactive shell session

The following features are currently planned, in the order that it will be implemented:
- [ ] Setting Variables
- [ ] TLS encrypted transport
- [ ] File upload/download
- [ ] Ability to edit remote files
- [ ] Establish Persistence to avoid re-running exploit
- [ ] Support for Linux and Windows Targets

## CruxAgent
CruxAgent is the component of the C2 that provides server control over the target. It should be transferred onto the target machine and executed like following:
```
$ CruxAgent --help
A simple Command & Control Agent inside the CruxC2 framework.

Usage: CruxAgent [OPTIONS] <RHOST>

Arguments:
  <RHOST>  Remote host to connect to (mandatory)

Options:
  -p, --port <RPORT>  Remote port to connect to (short -p) [default: 1337]
  -h, --help          Print help
  -V, --version       Print version
```

## CruxServer
CruxServer will listen for incoming connections (on port 1337 by default) much like a reverse shell listener when launched.
```
$ CruxServer --help
A simple Command & Control Server inside the CruxC2 framework.

Usage: CruxServer [OPTIONS]

Options:
  -p, --port <PORT>  The port to listen on (short -c) [default: 1337]
  -h, --help         Print help
  -V, --version      Print version
```

After the agent connects, the operator will be presented with with a clean interactive shell session with readline features such as up/down arrows to cycle through command history and moving the cursor with left/right arrow to edit commands.

Operator can execute most shell commands with different shell operators (`\`, `>`, etc.). Although setting shell variables are not currently implemented, reading existing ones are.

```
$ CruxServer

   ______                             ______   _____
 .' ___  |                          .' ___  | / ___ `.
/ .'   \_| _ .--.  __   _   _   __ / .'   \_||_/___) |
| |       [ `/'`\][  | | | [ \ [  ]| |        .'____.'
\ `.___.'\ | |     | \_/ |, > '  < \ `.___.'\/ /_____
 `.____ .'[___]    '.__.'_/[__]`\_] `.____ .'|_______|


CruxServer is listening on port 1337
Agent 0 Connected from 127.0.0.1:37122!
CRUX|brian@rx-93-nu|127.0.0.1:37122|$ echo "Hello World"
Hello World

CRUX|brian@rx-93-nu|127.0.0.1:37122|$ echo "Hello World" | base64
SGVsbG8gV29ybGQK

CRUX|brian@rx-93-nu|127.0.0.1:37122|$ echo "Hello World" > file.txt

CRUX|brian@rx-93-nu|127.0.0.1:37122|$ cat file.txt
Hello World

CRUX|brian@rx-93-nu|127.0.0.1:37122|$ echo $HOME
/home/brian

CRUX|brian@rx-93-nu|127.0.0.1:37122|$
```

## Disclaimer
This project is for educational or red teaming use in controlled, authorized environments only. Any misuse against unauthorized systems is illegal and strictly discouraged.

## License
This project is distributed under GPLv3 license.
