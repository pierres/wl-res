CFLAGS  ?= -O2 -Wall -Wextra -std=c11
CFLAGS  += $(shell pkg-config --cflags sdl3)
LDLIBS  += $(shell pkg-config --libs sdl3)
PREFIX  ?= $(HOME)/.local

screen-res: screen-res.c
	$(CC) $(CFLAGS) -o $@ $< $(LDLIBS)

install: screen-res
	install -Dm755 screen-res $(DESTDIR)$(PREFIX)/bin/screen-res

clean:
	rm -f screen-res

.PHONY: install clean
