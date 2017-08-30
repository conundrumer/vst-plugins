# vst
OUT := liboscify.dylib
PLUGIN := Oscify
PLUGIN_DEBUG := OscifyDebug

VST_BUNDLER :=../osx_vst_bundler.sh
SRC_FILES := $(wildcard src/*.rs)
VST_DIR := $(HOME)/Library/Audio/Plug-Ins/VST

all: $(VST_DIR)/$(PLUGIN).vst config

debug: $(VST_DIR)/$(PLUGIN_DEBUG).vst config

$(PLUGIN_DEBUG).vst: $(SRC_FILES)
	cargo build
	$(VST_BUNDLER) $(PLUGIN_DEBUG) target/debug/$(OUT)

$(PLUGIN).vst: $(SRC_FILES)
	cargo build --release
	$(VST_BUNDLER) $(PLUGIN) target/release/$(OUT)

$(VST_DIR)/%.vst: %.vst
	rm -rf $@
	cp -r $< $@

# config

CONFIG := oscify-config.json
CONFIG_EXAMPLE := oscify-config-example.json

DAW_DIR := $(HOME)/Library/Application\ Support/Bitwig/Bitwig\ Studio

config: $(DAW_DIR)/$(CONFIG)

$(CONFIG): | $(CONFIG_EXAMPLE)
	cp -n $(CONFIG_EXAMPLE) $(CONFIG)

$(DAW_DIR)/$(CONFIG): $(CONFIG)
	cp $< "$@"
