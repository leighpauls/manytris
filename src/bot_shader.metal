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
  uint8_t left_shifts;
  uint8_t right_shifts;
};

void set_pos(device Field* f, uint8_t x, uint8_t y) {
  size_t bit_index = y * W + x;
  size_t byte_index = bit_index / 8;
  size_t offset = bit_index % 8;
  uint8_t mask = 1 << offset;
  f->bytes[byte_index] |= mask;
}

bool is_occupied(device Field* f, uint8_t x, uint8_t y) {
  size_t bit_index = y * W + x;
  size_t byte_index = bit_index / 8;
  size_t offset = bit_index % 8;
  uint8_t mask = 1 << offset;
  return (f->bytes[byte_index] & mask) != 0;
}

enum ShiftDir {Down, Left, Right};

bool try_shift(device Field* f, thread TetrominoPositions* tp, ShiftDir d) {
  for (auto i = 0; i < 4; i++) {
    thread uint8_t* p = tp->pos[i];
    switch (d) {
    case Down:
      if (p[1] == 0 || is_occupied(f, p[0], p[1]-1)) {
        return false;
      }
      break;
    case Left:
      if (p[0] == 0 || is_occupied(f, p[0]-1, p[1])) {
        return false;
      }
      break;
    case Right:
      if (p[0] == W-1 || is_occupied(f, p[0]+1, p[1])) {
        return false;
      }
      break;
    }
  }
  for (auto i = 0; i < 4; i++) {
    thread uint8_t* p = tp->pos[i];
    switch (d) {
    case Down:
      p[1] -= 1;
      break;
    case Left:
      p[0] -= 1;
      break;
    case Right:
      p[0] += 1;
      break;
    }
  }
  return true;
}

[[kernel]] void drop_tetromino(
    device const TetrominoPositions* tp,
    device Field* fields,
    device const DropConfig* configs,
    uint config_index [[thread_position_in_grid]]) {
  auto config = &configs[config_index];
  auto source_field = &fields[config->initial_field_idx];
  auto dest_field = &fields[config->dest_field_idx];

  *dest_field = *source_field;

  auto p = tp[config->tetromino_idx];

  for (auto i = 0; i < config->left_shifts; i++) {
    try_shift(dest_field, &p, ShiftDir::Left);
  }
  for (auto i = 0; i < config->right_shifts; i++) {
    try_shift(dest_field, &p, ShiftDir::Right);
  }

  while (try_shift(dest_field, &p, ShiftDir::Down)) {}

  for (size_t i = 0; i < 4; i++) {
    set_pos(dest_field, p.pos[i][0], p.pos[i][1]);
  }
}
