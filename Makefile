
all: build

build:
	cargo build

build_detailed_debugging:
	cargo xtask bundle compressor --release --features "detailed_debugging"
	
slap: build
	plugalyzer process --plugin /home/pieter/.vst3/Compressor.vst3 \
	--input=/home/pieter/Coding/rust/compressor/audio/slap.wav \
	--output=out.wav \
	--param=Threshold:-25 \
	--overwrite

sine: build_detailed_debugging
	plugalyzer process --plugin "/home/pieter/.vst3/Compressor.vst3" \
	--input=/home/pieter/Coding/rust/compressor/audio/sine_40hz_4s.wav \
	--output=out.wav \
	--param=Threshold:-25 \
	--overwrite

clean:
	cargo clean
