.PHONY: all

all:
	@cp ../../common/$(target).json $(target).json
	@RUST_TARGET_PATH="$(shell pwd)" cargo xbuild --target $(target)
	@rm $(target).json
