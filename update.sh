#!/usr/bin/env bash

cargo update
cd runtime && cargo update
cd wasm && cargo update
cd ../../
