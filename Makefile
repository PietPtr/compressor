
all: build

build:
	cargo xtask bundle compressor --release

debug:
	cargo xtask bundle compressor --release --features "detailed_debugging"

debug_gui:
	cargo xtask bundle compressor --release --features "external_stylesheet"

gui: debug_gui
	./target/bundled/Compressor

slap: build
	plugalyzer process --plugin /home/pieter/.vst3/Compressor.vst3 \
	--input=/home/pieter/Coding/rust/compressor/audio/slap.wav \
	--output=out.wav \
	--param=Threshold:-25 \
	--overwrite
	
funk: build
	plugalyzer process --plugin /home/pieter/.vst3/Compressor.vst3 \
	--input=/home/pieter/Coding/rust/compressor/audio/funk.wav \
	--output=out.wav \
	--param=Threshold:-24 \
	--param=Attack:100 \
	--param=Release:100 \
	--param=Ratio:20 \
	--overwrite

sine: debug
	-rm debug.csv
	-plugalyzer process --plugin "/home/pieter/.vst3/Compressor.vst3" \
	--input=/home/pieter/Coding/rust/compressor/audio/sine_40hz_4s.wav \
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
	--input=/home/pieter/Coding/rust/compressor/audio/square_120hz_4s.wav \
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
	--input=/home/pieter/Coding/rust/compressor/audio/triangle_120hz.wav \
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
	--input=/home/pieter/Coding/rust/compressor/audio/funk.wav \
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
