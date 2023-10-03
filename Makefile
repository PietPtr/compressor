
all: build

build:
	cargo build

debug:
	cargo xtask bundle compressor --release --features "detailed_debugging"
	
slap: build
	plugalyzer process --plugin /home/pieter/.vst3/Compressor.vst3 \
	--input=/home/pieter/Coding/rust/compressor/audio/slap.wav \
	--output=out.wav \
	--param=Threshold:-25 \
	--overwrite

sine: debug
	plugalyzer process --plugin "/home/pieter/.vst3/Compressor.vst3" \
	--input=/home/pieter/Coding/rust/compressor/audio/sine_40hz_4s.wav \
	--output=out.wav \
	--param=Threshold:-14 \
	--param=Attack:10 \
	--param=Release:5 \
	--overwrite
	python tools/plot.py debug.csv

clean:
	cargo clean
