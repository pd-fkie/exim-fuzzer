# Fuzzing Exim with AFLNet

This directory contains a setup for fuzzing Exim with AFLNet inside a docker container.
We used this as a baseline for a comparison to our own fuzzer in `../`.

## How to build
```
sudo docker build -t exim-aflnet .
```

## How to fuzz
```
mkdir shared
sudo docker run --security-opt=seccomp:unconfined -v "$PWD/shared:/shared" -d --entrypoint /bin/bash exim-aflnet /fuzz.sh
```

The results of the fuzzing campaign can be found in `shared/output/`.
