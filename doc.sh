#!/bin/sh
# generate documentation only for partiy and ethcore libraries

cargo doc --no-deps --verbose \
	-p ethkey \
	-p ethstore \
	-p ethash \
	-p ethcore-util \
	-p ethcore \
	-p ethsync \
	-p ethcore-rpc \
	-p ethcore-signer \
	-p ethcore-dapps \
	-p parity \
