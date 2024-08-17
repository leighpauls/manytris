constant constexpr size_t W = 10;
constant constexpr size_t H = 22;
constant constexpr size_t NUM_BLOCKS = W*H;
constant constexpr size_t FIELD_BYTES = NUM_BLOCKS / 8 + ((NUM_BLOCKS % 8) ? 1 : 0);

struct Field {
  uint8_t bytes[FIELD_BYTES];
};

struct TetrominoPositions {
  uint8_t pos[4][2];
};


struct DropConfig {
  uint32_t tetromino_idx;
  uint32_t initial_field_idx;
  uint32_t dest_field_idx;
};

void set_pos(device Field* f, uint8_t x, uint8_t y) {
  size_t bit_index = y * W + x;
  size_t byte_index = bit_index / 8;
  size_t offset = bit_index % 8;
  uint8_t mask = 1 << offset;
  f->bytes[byte_index] |= mask;
}

[[kernel]] void drop_tetromino(
    device const TetrominoPositions* tp,
    device Field* fields,
    device const DropConfig* configs,
    uint config_index [[thread_position_in_grid]]) {
  auto config = &configs[config_index];
  auto source_field = &fields[config->initial_field_idx];
  auto dest_field = &fields[config->dest_field_idx];

  for (size_t i = 0; i < FIELD_BYTES; i++ ) {
    dest_field->bytes[i] = source_field->bytes[i];
  }

  for (size_t i = 0; i < 4; i++) {
    set_pos(dest_field, tp->pos[i][0], tp->pos[i][1]);
  }
}
