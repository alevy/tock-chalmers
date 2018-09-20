# Makefile for user application

# Specify this directory relative to the current application.
TOCK_USERLAND_BASE_DIR = ../../../../../userland

# Which files to compile.
C_SRCS := $(wildcard *.c)

# Include userland master makefile. Contains rules and flags for actually
# building the application.
include $(TOCK_USERLAND_BASE_DIR)/AppMakefile.mk

app.tbf: build/app.tab
	cp build/cortex-m4/cortex-m4.tbf $@

.PHONY: all
all: app.tbf
