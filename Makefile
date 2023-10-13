
all: build

build:
	cargo clippy
	cargo xtask bundle compressor --release

debug:
	cargo clippy
	cargo xtask bundle compressor --release --features "detailed_debugging"

gui: build
	./target/bundled/Compressor

slap: build
	plugalyzer process --plugin /home/pieter/.vst3/Compressor.vst3 \
	--input=/home/pieter/Coding/rust/compressor/resources/slap.wav \
	--output=out.wav \
	--param=Threshold:-25 \
	--overwrite
	
funk: build
	plugalyzer process --plugin /home/pieter/.vst3/Compressor.vst3 \
	--input=/home/pieter/Coding/rust/compressor/resources/funk.wav \
	--output=out.wav \
	--param=Threshold:-24 \
	--param=Attack:100 \
	--param=Release:100 \
	--param=Ratio:20 \
	--overwrite

sine: debug
	-rm debug.csv
	-plugalyzer process --plugin "/home/pieter/.vst3/Compressor.vst3" \
	--input=/home/pieter/Coding/rust/compressor/resources/sine_40hz_4s.wav \
	--output=out.wav \
	--param=Threshold:-14 \
	--param=Attack:100 \
	--param=Release:100 \
	--param=Ratio:4 \
	--param=LoggerLength:5000 \
	--overwrite
	python tools/plot.py debug.csv


square: debug
	-rm debug.csv
	-plugalyzer process --plugin "/home/pieter/.vst3/Compressor.vst3" \
	--input=/home/pieter/Coding/rust/compressor/resources/square_120hz_4s.wav \
	--output=out.wav \
	--param=Threshold:-14 \
	--param=Attack:1 \
	--param=Release:100 \
	--param=Ratio:4 \
	--param=LoggerLength:1000 \
	--overwrite
	python tools/plot.py debug.csv


triangle: debug
	-rm debug.csv
	-plugalyzer process --plugin "/home/pieter/.vst3/Compressor.vst3" \
	--input=/home/pieter/Coding/rust/compressor/resources/triangle_120hz.wav \
	--output=out.wav \
	--param=Threshold:-14 \
	--param=Attack:0 \
	--param=Release:1000 \
	--param=Ratio:4 \
	--param=Steepness:10 \
	--param=LoggerLength:1000 \
	--overwrite
	python tools/plot.py debug.csv


funkd: debug
	-rm debug.csv
	-plugalyzer process --plugin "/home/pieter/.vst3/Compressor.vst3" \
	--input=/home/pieter/Coding/rust/compressor/resources/funk.wav \
	--output=out.wav \
	--param=Threshold:-24 \
	--param=Attack:100 \
	--param=Release:100 \
	--param=Ratio:20 \
	--param=LoggerLength:50000 \
	--overwrite
	python tools/plot.py debug.csv

clean:
	cargo clean
