#! /bin/bash

time cargo +nightly run --features plot  && cat samples/out.wav > /dev/dsp