CC = gcc
CFLAGS = -Wall -Wextra -std=c11 -D_POSIX_C_SOURCE=200809L -Isrc
LDFLAGS = -lpthread -lcurl -lm -lcjson
 
SRC_DIR = src
BUILD_DIR = build
TARGET = $(BUILD_DIR)/health_checker
TEST_TARGET = $(BUILD_DIR)/health_evaluator_test
 
SOURCES = $(SRC_DIR)/main.c \
          $(SRC_DIR)/config.c \
          $(SRC_DIR)/db_client.c \
          $(SRC_DIR)/tcp_check.c \
          $(SRC_DIR)/http_check.c \
		  $(SRC_DIR)/target_task.c \
          $(SRC_DIR)/health_evaluator.c \
          $(SRC_DIR)/location.c \
          $(SRC_DIR)/http_client.c
 
OBJECTS = $(SOURCES:$(SRC_DIR)/%.c=$(BUILD_DIR)/%.o)
 
all: $(TARGET)
 
$(TARGET): $(OBJECTS)
	@mkdir -p $(BUILD_DIR)
	$(CC) $(CFLAGS) -o $@ $^ $(LDFLAGS)
 
$(BUILD_DIR)/%.o: $(SRC_DIR)/%.c
	@mkdir -p $(BUILD_DIR)
	$(CC) $(CFLAGS) -c $< -o $@
 
clean:
	rm -rf $(BUILD_DIR)

$(TEST_TARGET): tests/health_evaluator_test.c src/health_evaluator.c
	@mkdir -p $(BUILD_DIR)
	$(CC) $(CFLAGS) -o $@ $^

test: $(TEST_TARGET)
	./$(TEST_TARGET)

.PHONY: all clean test
 
